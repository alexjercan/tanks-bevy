# Tanks Bevy

A simple multiplayer game using Bevy.
The game is split into two parts: the server and the client.

## Quickstart

### Server

```console
cargo run --bin tanks_server --no-default-features --features="server"
```

### Client Native

```console
cargo run --bin tanks_client --no-default-features --features="client"
```

### Client Wasm

```console
trunk serve
```
