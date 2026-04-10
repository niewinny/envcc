#!/usr/bin/env node

const { execFileSync } = require("child_process");
const path = require("path");
const fs = require("fs");

const PLATFORM_MAP = {
  "linux-x64": "envcc-linux-x64",
  "linux-arm64": "envcc-linux-arm64",
  "darwin-x64": "envcc-darwin-x64",
  "darwin-arm64": "envcc-darwin-arm64",
  "win32-x64": "envcc-win-x64.exe",
};

const key = `${process.platform}-${process.arch}`;
const binaryName = PLATFORM_MAP[key];

if (!binaryName) {
  console.error(`Unsupported platform: ${key}`);
  console.error(`Supported: ${Object.keys(PLATFORM_MAP).join(", ")}`);
  process.exit(1);
}

const binaryPath = path.join(__dirname, "..", "binaries", binaryName);

if (!fs.existsSync(binaryPath)) {
  console.error(`Binary not found: ${binaryPath}`);
  console.error("This platform may not have a prebuilt binary yet.");
  console.error("Install from source: cargo install envcc");
  process.exit(1);
}

try {
  execFileSync(binaryPath, process.argv.slice(2), { stdio: "inherit" });
} catch (e) {
  process.exit(e.status || 1);
}
