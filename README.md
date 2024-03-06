# run-in-roblox
run-in-roblox is a tool to run a place, a model, or an individual script inside Roblox Studio.

run-in-roblox pipes output from inside Roblox Studio back to stdout/stderr, which enables traditional automation tools to work alongside Roblox.

## Installation

### From GitHub Releases
You can download pre-built binaries from [run-in-roblox's GitHub Releases page](https://github.com/azul-rbx/run-in-roblox/releases).

## Usage
The recommended way to use `run-in-roblox` is with a place file and a script to run:

```bash
run-in-roblox --place MyPlace.rbxlx --script starter-script.lua
```

This will open `MyPlace.rbxlx` in Roblox Studio, run `starter-script.lua` until it completes, and then exit.

`--place` is optional, but `--script` is required.

## License
run-in-roblox is available under the terms of the MPL-2.0 License. See [LICENSE](LICENSE) or <https://opensource.org/licenses/mpl-2-0> for details.