# iOS

## Requirements

- Install [Lipo](https://github.com/TimNN/cargo-lipo): `cargo install cargo-lipo`
- Add Rust iOS targets: `rustup target add aarch64-apple-ios x86_64-apple-ios`
- Install XcodeGen: `brew install xcodegen`

## Build makepad app as a static lib

- `cargo lipo` or `cargo lipo --release`

## Setup Xcode project for final native iOS app

- Adapt `project.yml`
- Run XcodeGen: `xcodegen`
- Open project in Xcode and build the app

### Helpful Articles

- <http://iauns.com/gpu/Metal_GPU_Programming_01_-_Clear_Screen.html>
