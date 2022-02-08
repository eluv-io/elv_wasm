# Eluvio Fabric WASM Client Library

## Installing

### Install nvm and nodejs
```
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.38.0/install.sh | bash
export NVM_DIR="$([ -z "${XDG_CONFIG_HOME-}" ] && printf %s "${HOME}/.nvm" || printf %s "${XDG_CONFIG_HOME}/nvm")"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh" # This loads nvm

nvm install 14.7.0
```

### Install Rust and add nightly toolchain and wasm32 targets

```
curl https://sh.rustup.rs -sSf | sh -s -- -y
source $HOME/.cargo/env
rustup target add wasm32-unknown-unknown
```

## Building


### Rust
```
cargo build --target wasm32-unknown-unknown --release --workspace

```

## Programming interface

[API](API.md)
