use std::{
    io::{Read, Write}, process::{self, Command, Stdio}, rc::Rc, sync::Arc, time::Duration
};

use anyhow::{anyhow, bail, Context, Result};
use fs_err::File;
use log::{error, info};
use roblox_install::RobloxStudio;
use tokio::time::sleep;

use crate::{
    message_receiver::{self, Message, RobloxMessage},
    RunOptions,
};

/// A wrapper for `process::Child` that force-kills the process on drop.
struct KillOnDrop(process::Child, bool);

impl Drop for KillOnDrop {
    fn drop(&mut self) {
        if self.1 {
            return;
        }
        let _ignored = self.0.kill();
    }
}

pub struct PlaceRunnerArgs {
    pub port: u16,
    pub script: String,
    pub opts: RunOptions,
}

pub struct PlaceRunner {
    port: u16,
    script: String,
    opts: RunOptions,
    api_svc: Option<Arc<message_receiver::Svc>>,
    studio_process: Option<Arc<KillOnDrop>>,
    studio_install: Option<Arc<RobloxStudio>>,
    exit_receiver: Option<async_channel::Receiver<()>>,
    message_recv_tx: Option<async_channel::Sender<Option<RobloxMessage>>>
}

impl PlaceRunner {
    pub fn new(args: PlaceRunnerArgs) -> Self {
        Self {
            port: args.port,
            script: args.script,
            opts: args.opts,
            api_svc: None,
            studio_process: None,
            studio_install: None,
            exit_receiver: None,
            message_recv_tx: None,
        }
    }

    pub fn get_studio_install(&mut self) -> Result<Option<Arc<RobloxStudio>>> {
        if self.studio_install.is_none() { 
            let studio_install =
                RobloxStudio::locate().context("Could not locate a Roblox Studio installation.")?;
            self.studio_install = Some(studio_install.into());
        }
        Ok(self.studio_install.clone())
    }

    pub fn install_plugin(&mut self) -> Result<()> {
        let studio_install = &self.get_studio_install()?.unwrap();

        let mut local_plugin = match File::open("./plugin/plugin.rbxm") {
            Err(_) => {
                bail!("could not open plugin file - did you build it with `lune`?");
            }
            Ok(file) => file,
        };
        let mut local_plugin_data = vec![];
        local_plugin.read_to_end(&mut local_plugin_data)?;

        if std::fs::create_dir_all(studio_install.plugins_path()).is_err() {
            bail!("could not create plugins directory - are you missing permissions to write to `{:?}`?", studio_install.plugins_path());
        }

        let plugin_file_path = studio_install.plugins_path().join("run_in_roblox.rbxm");
        let mut plugin_file = File::create(plugin_file_path)?;
        plugin_file.write_all(&local_plugin_data)?;
        Ok(())
    }

    pub fn remove_plugin(&mut self) -> Result<()> {
        let studio_install = self.get_studio_install()?.unwrap();
        let plugin_file_path = studio_install.plugins_path().join("run_in_roblox.rbxm");

        std::fs::remove_file(plugin_file_path)?;
        Ok(())
    }

    pub fn get_studio_args(&mut self) -> Result<Vec<String>> {
        let studio_install = self.get_studio_install()?.unwrap();
        let result = match &self.opts.team_test {
            true => {
                vec![
                    "-task".to_string(),
                    "StartTeamTest".to_string(),
                    "-placeId".to_string(),
                    format!("{:}", self.opts.place_id.unwrap()),
                    "-universeId".to_string(),
                    format!("{:}", self.opts.universe_id.unwrap()),
                ]
            }
            false => {
                let place_file = self.opts.place_file.as_ref().unwrap();
                std::fs::copy(
                    place_file,
                    dbg!(studio_install.plugins_path().join("../server.rbxl")),
                )?;
                vec![
                    "-task".to_string(),
                    "StartServer".to_string(),
                    "-placeId".to_string(),
                    format!("{:}", self.opts.place_id.unwrap()),
                    "-universeId".to_string(),
                    format!("{:}", self.opts.universe_id.unwrap()),
                    "-creatorId".to_string(),
                    format!("{:}", self.opts.creator_id.unwrap()),
                    "-creatorType".to_string(),
                    format!("{:}", self.opts.creator_type.unwrap()),
                    "-numtestserverplayersuponstartup".to_string(),
                    format!("{:}", self.opts.num_clients),
                ]
            }
        };

        Ok(result)
    }

    pub async fn stop(
        &mut self
    ) -> Result<()> {
        if let Some(api_svc) = &self.api_svc {
            api_svc.stop().await;
        }
        
        if let Some(sender) = &self.message_recv_tx {
            sender.close();
        }

        self.remove_plugin()?;

        Ok(())
    }

    pub async fn handle_server_start(
        &self,
        server: String
    ) {
        let api_svc = self.api_svc.as_ref().unwrap();
        info!("studio server {server:} started");
        api_svc
            .queue_event(
                server.clone(),
                message_receiver::RobloxEvent::RunScript {
                    script: self.script.clone(),
                    oneshot: self.opts.oneshot
                },
            )
            .await;
        // By default, if "oneshot" and "no_exit" mode is specified,
        // we don't have control over the Studio executable's lifecycle,
        // so we'll send a "Deregister" message to Studio so that this application
        // can exit cleanly, allowing you to re-run it again in a sort-of-"watch mode".
        if self.opts.oneshot {
            api_svc
                .queue_event(server.clone(), message_receiver::RobloxEvent::Deregister {
                    no_exit: self.opts.no_exit
                }).await;
        }
    }

    pub async fn run(
        &mut self,
        sender: async_channel::Sender<Option<RobloxMessage>>,
        exit_receiver: async_channel::Receiver<()>,
    ) -> Result<(), anyhow::Error> {
        self.message_recv_tx = Some(sender);
        self.exit_receiver = Some(exit_receiver);

        self.install_plugin()?;

        let studio_install = self.get_studio_install()?.unwrap();
        let studio_args = self.get_studio_args()?;

        self.api_svc = Some(
            message_receiver::Svc::start()
                .await
                .expect("api service to start"),
        );

        self.studio_process = if self.opts.no_launch {
            None
        } else {
            info!("launching roblox studio with args {studio_args:?}");
            Some(KillOnDrop(
                Command::new(studio_install.application_path())
                    .args(studio_args)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()?,
                self.opts.no_exit,
            ).into())
        };

        if !self.opts.no_launch {
            let api_svc = self.api_svc.as_ref().unwrap();
            let exit_receiver = self.exit_receiver.as_ref().unwrap();
            let timeout_task = async {
                sleep(Duration::from_secs(30)).await;
            };

            let progress_bar = indicatif::ProgressBar::new_spinner().with_message("Waiting for an instance of Roblox Studio to come alive...");
            progress_bar.enable_steady_tick(Duration::from_millis(50));
            tokio::select! {
                msg = api_svc.recv() => {
                    let Message::Start { server } = msg else {
                        return Err(anyhow!("expected first message to be received to be a server starting"))
                    };
                    progress_bar.finish_and_clear();
                    self.handle_server_start(server).await;
                },
                _ = exit_receiver.recv() => {
                    progress_bar.finish_and_clear();
                    info!("ctrl-c caught, exiting now");
                    self.stop().await?;
                    return Ok(())
                },
                () = timeout_task => {
                    error!("caught a timeout while waiting for a studio instance to start - do you need to login?");
                    self.stop().await?;
                    return Ok(())
                }
            }
        }

        loop {
            let api_svc = self.api_svc.as_ref().unwrap();
            let exit_receiver = self.exit_receiver.as_ref().unwrap();
            let sender = self.message_recv_tx.as_ref().unwrap();

            tokio::select! {
                msg = api_svc.recv() => {
                    match msg {
                        Message::Start { server } => {
                            self.handle_server_start(server).await;
                        }
                        Message::Stop { server } => {
                            info!("studio server {server:} stopped");
                            if self.opts.oneshot {
                                info!("now exiting as --oneshot was set to true.");
                                break;
                            }
                        }
                        Message::Messages(roblox_messages) => {
                            for message in roblox_messages {
                                sender.send(Some(message)).await?;
                            }
                        }
                    }
                }
                _ = exit_receiver.recv() => {
                    break;
                }
            }
        }

        self.stop().await?;
        Ok(())
    }
}