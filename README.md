# Eluvio Fabric WASM Client Library

## Installing

### Install nvm and nodejs

```shell
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.38.0/install.sh | bash
export NVM_DIR="$([ -z "${XDG_CONFIG_HOME-}" ] && printf %s "${HOME}/.nvm" || printf %s "${XDG_CONFIG_HOME}/nvm")"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh" # This loads nvm

nvm install 14.7.0
```

### Install Assemblyscript

```shell
npm i assemblyscript
```

### Install Rust and add nightly toolchain and wasm32 targets

```shell
curl https://sh.rustup.rs -sSf | sh -s -- -y
source $HOME/.cargo/env
rustup toolchain install nightly
rustup update
rustup target add wasm32-unknown-unknown --toolchain nightly
rustup default nightly
```

### Install Tinygo

For Linux

```shell
wget https://github.com/tinygo-org/tinygo/releases/download/v0.27.0/tinygo_0.27.0_amd64.deb
sudo dpkg -i tinygo_0.27.0_amd64.deb
```

Alternately, for macos, run

```shell
brew tap tinygo-org/tools
brew install tinygo
```

## Building

### Rust

If you do not use nightly rust by default, you can add `+nightly` between `cargo` and `build` in order to use nightly rust to build this. Otherwise, the below command suffices.

```shell
cargo build --target wasm32-unknown-unknown --release --workspace

```

## Programming interface

[API](API.md)
