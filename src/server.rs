//! ActiveLook server
//!
//! This is used in the ActiveLook emulator, to simulate the behaviour of ActiveLook glasses and
//! accelerate development.

use embedded_io::{Read, Write};
use log::*;
use thiserror::Error;

use crate::{
    commands::{Command, Response},
    protocol::{CommandPacket, Packet, ProtocolError, ResponsePacket, PACKET_MAX_SIZE},
    traits::*,
};

/// Server which uses:
/// - Connection to Tx Activelook Server (Write)
/// - Connection to Rx Activelook Server (Notify)
/// - Connection to Control server (Write)
pub struct ActiveLookServer<TxActiveLook, RxActiveLook, Ctrl>
where
    TxActiveLook: Write,
    RxActiveLook: Read,
    Ctrl: Write,
{
    /// Server Rx is connected to ActiveLook Rx
    rx: RxActiveLook,
    /// Server Tx is connected to ActiveLook Tx
    tx: TxActiveLook,
    ctrl: Ctrl,
}

/// Protocol implementation
/// https://github.com/ActiveLook/Activelook-API-Documentation/blob/fw-4.12.0_doc-revA/ActiveLook_API.md#35-control-server
impl<TxActiveLook, RxActiveLook, Ctrl> ActiveLookServer<TxActiveLook, RxActiveLook, Ctrl>
where
    TxActiveLook: Write,
    RxActiveLook: Read,
    Ctrl: Write,
{
    pub fn new(rx: RxActiveLook, tx: TxActiveLook, ctrl: Ctrl) -> Self {
        Self { rx, tx, ctrl }
    }

    pub fn read_data(&mut self) -> Result<CommandPacket, ProtocolError> {
        let mut rxbuf = [0; PACKET_MAX_SIZE];
        if let Ok(len) = self.rx.read(&mut rxbuf) {
            CommandPacket::from_bytes(&rxbuf[..len])
        } else {
            //trace!("No data to read");
            Err(ProtocolError::Empty)
        }
    }

    pub fn send_response(&mut self, response: ResponsePacket) {
        let bytes = response.to_bytes();
        self.tx.write(&bytes);
    }
}
