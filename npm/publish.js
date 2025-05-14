#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

// Configuration for platform/architecture mapping
const CONFIG = {
    platforms: [
        {
            rust: 'x86_64-pc-windows-msvc',
            node_pkg: 'run-in-roblox-windows-x64',
            node_os: 'win32',
            node_arch: 'x64',
            binary_name: 'run-in-roblox.exe'
        },
        {
            rust: 'aarch64-pc-windows-msvc',
            node_pkg: 'run-in-roblox-windows-arm64',
            node_os: 'win32',
            node_arch: 'arm64',
            binary_name: 'run-in-roblox.exe'
        },
        {
            rust: 'x86_64-apple-darwin',
            node_pkg: 'run-in-roblox-darwin-x64',
            node_os: 'darwin',
            node_arch: 'x64',
            binary_name: 'run-in-roblox'
        },
        {
            rust: 'aarch64-apple-darwin',
            node_pkg: 'run-in-roblox-darwin-arm64',
            node_os: 'darwin',
            node_arch: 'arm64',
            binary_name: 'run-in-roblox'
        },
        {
            rust: 'x86_64-unknown-linux-gnu',
            node_pkg: 'run-in-roblox-linux-x64',
            node_os: 'linux',
            node_arch: 'x64',
            binary_name: 'run-in-roblox'
        },
        {
            rust: 'aarch64-unknown-linux-gnu',
            node_pkg: 'run-in-roblox-linux-arm64',
            node_os: 'linux',
            node_arch: 'arm64',
            binary_name: 'run-in-roblox'
        }
    ],
    version: require('../Cargo.toml').package.version || process.env.npm_package_version || '0.3.2'
};

// Ensure directories exist
const TEMP_DIR = path.join(__dirname, 'temp');
if (!fs.existsSync(TEMP_DIR)) {
    fs.mkdirSync(TEMP_DIR, { recursive: true });
}

// Read the template file
const packageTemplate = fs.readFileSync(
    path.join(__dirname, 'package.json.tmpl'),
    'utf-8'
);

// Process each platform
function createPackages() {
    for (const platform of CONFIG.platforms) {
        console.log(`Processing ${platform.node_pkg}...`);

        // Create platform directory
        const platformDir = path.join(TEMP_DIR, platform.node_pkg);
        if (!fs.existsSync(platformDir)) {
            fs.mkdirSync(platformDir, { recursive: true });
        }

        // Generate package.json from template
        const packageJson = packageTemplate
            .replace('${node_pkg}', platform.node_pkg)
            .replace('${node_version}', CONFIG.version)
            .replace('${node_os}', platform.node_os)
            .replace('${node_arch}', platform.node_arch);

        fs.writeFileSync(
            path.join(platformDir, 'package.json'),
            packageJson
        );

        console.log(`Created package.json for ${platform.node_pkg}`);
    }
}

// Function to prepare app package
function prepareAppPackage() {
    const appDir = path.join(__dirname, 'app');
    console.log('Building TypeScript wrapper...');

    // Update version in package.json to match Cargo.toml
    const appPackageJsonPath = path.join(appDir, 'package.json');
    const appPackageJson = require(appPackageJsonPath);
    appPackageJson.version = CONFIG.version;

    // Update optionalDependencies versions
    Object.keys(appPackageJson.optionalDependencies).forEach(dep => {
        appPackageJson.optionalDependencies[dep] = CONFIG.version;
    });

    fs.writeFileSync(
        appPackageJsonPath,
        JSON.stringify(appPackageJson, null, 2)
    );

    // Build the TypeScript
    execSync('npm run build', {
        cwd: appDir,
        stdio: 'inherit'
    });

    console.log('App package prepared');
}

function displayInstructions() {
    console.log('\n--- PUBLISHING INSTRUCTIONS ---');
    console.log('1. Make sure you have built the Rust binaries for all platforms');
    console.log('2. Copy each binary to its respective npm/temp/{platform} directory');
    console.log('3. Publish the platform packages first:');
    CONFIG.platforms.forEach(platform => {
        console.log(`   cd npm/temp/${platform.node_pkg} && npm publish`);
    });
    console.log('4. Then publish the main package:');
    console.log('   cd npm/app && npm publish');
    console.log('\nYou can then use the package with:');
    console.log('   npx run-in-roblox --help');
}

// Main execution
try {
    createPackages();
    prepareAppPackage();
    displayInstructions();
} catch (error) {
    console.error('Error:', error);
    process.exit(1);
}