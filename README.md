# ActiveLook Rust driver

This is a driver for [ActiveLook](https://www.activelook.net) glasses.

It exposes the BLE commands and protocol as rust types.

The Luciole ESP32 embedded project uses it with a BLE client implementation connected to real ActiveLook hardware.

The `activelook-emulator` project implements the low level connectivity with ZeroMQ, but the same protocol and packet format is used.

---
> :warning: **WARNING** :warning:  
>
> This **Work In Progress** is by no means complete.  
> We are open to suggestions/criticism but have very limited time to address them.
---



Interesting files:

| File | Content |
|------|---------|
| commands.rs | All ActiveLook commands, as described in the official [API documentation](git@forge.kaizen-solutions.net:kzslab/stages/2024/luciole/presentation-projet-rust.git) |
| image.rs | Description of the `Image` type |
| protocol.rs | BLE `Packet` implementation |



## Binary de/serialization to BLE packet format

### Deku
The [`Deku` crate](https://docs.rs/deku) automatically offers the following:
- to_bytes / from_bytes
- try_from

We can specify a custom reader/writer for specific variants.
