//#![feature(trait_alias)]
pub mod commands;
pub mod protocol;
pub mod test_cstr;
pub mod traits;
use crate::commands::{Command, Response};
use crate::traits::*;

/*
pub trait ActiveLookClient {
    fn send(&self, cmd: &Command);
    fn recv(&self) -> Option<Response>;
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
        let cmd = Command::PowerDisplay { en: en as u8 };
        self.client.send(&cmd);
    }

    pub fn clear(&self) {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestClient;
    impl ActiveLookClient for TestClient {
        fn send(&self, cmd: &Command) {
            println!("Sending command {} {:?}", cmd.id().unwrap(), cmd);
        }

        fn recv(&self) -> Option<Response> {
            todo!();
        }
    }

    #[test]
    fn test_display() {
        let client = TestClient {};
        let glasses = Glasses::new(client);

        glasses.display_power(true);
        //assert!(false);
    }
}
*/
