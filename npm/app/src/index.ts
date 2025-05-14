#!/usr/bin/env node

import { spawn } from 'child_process';
import * as path from 'path';
import * as os from 'os';
import * as fs from 'fs';

const platform = os.platform();
const arch = os.arch();
let extension = '';

if (platform === 'win32') {
  extension = '.exe';
}

function getBinaryPath(): string {
  // Mapping OS and architecture to package names
  const platformMap: Record<string, string> = {
    'win32': 'windows',
    'darwin': 'darwin',
    'linux': 'linux'
  };

  const archMap: Record<string, string> = {
    'x64': 'x64',
    'arm64': 'arm64'
  };

  const npmPlatform = platformMap[platform] || 'unsupported';
  const npmArch = archMap[arch] || 'unsupported';

  if (npmPlatform === 'unsupported' || npmArch === 'unsupported') {
    throw new Error(`Unsupported platform: ${platform} ${arch}`);
  }

  // Format: app-{platform}-{arch}
  const packageName = `run-in-roblox-${npmPlatform}-${npmArch}`;

  try {
    // Try to find the binary in the platform-specific package
    const packagePath = path.dirname(require.resolve(`${packageName}/package.json`));
    const binaryPath = path.join(packagePath, `run-in-roblox${extension}`);
    
    if (fs.existsSync(binaryPath)) {
      // Make sure it's executable (for non-Windows platforms)
      if (platform !== 'win32') {
        fs.chmodSync(binaryPath, '755');
      }
      return binaryPath;
    } else {
      throw new Error(`Binary not found at ${binaryPath}`);
    }
  } catch (error) {
    if (error instanceof Error) {
      console.error(`Error finding run-in-roblox binary: ${error.message}`);
    } else {
      console.error('Unknown error finding run-in-roblox binary');
    }
    throw error;
  }
}

function main() {
  try {
    const binaryPath = getBinaryPath();
    const args = process.argv.slice(2);

    const child = spawn(binaryPath, args, {
      stdio: 'inherit',
      shell: false
    });

    child.on('error', (error) => {
      console.error(`Failed to start run-in-roblox: ${error.message}`);
      process.exit(1);
    });

    child.on('close', (code) => {
      process.exit(code || 0);
    });
  } catch (error) {
    if (error instanceof Error) {
      console.error(error.message);
    } else {
      console.error('An unknown error occurred');
    }
    process.exit(1);
  }
}

main();