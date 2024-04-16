//pub mod commands;

pub mod commands {
    pub struct PowerDisplay {
        en: bool,
    }
}

pub trait Command {}

pub trait ActiveLookClient {
    fn send(cmd: impl Command);
}

/// High level representation for BLE ActiveLook glasses
pub struct Glasses<C: ActiveLookClient> {
    client: C,
}

impl<C: ActiveLookClient> Glasses<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub fn display_power(&self, en: bool) {
        todo!();
    }

    pub fn clear(&self) {
        todo!();
    }
}
