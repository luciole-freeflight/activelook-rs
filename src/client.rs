use embedded_io::{Read, Write};
use log::*;
use thiserror::Error;

use crate::{
    commands::{Command, Response},
    protocol::{CommandPacket, Packet, ProtocolError, ResponsePacket, PACKET_MAX_SIZE},
    traits::*,
};

/// Client which uses:
/// - Connection to Tx Activelook Server (Notify)
/// - Connection to Rx Activelook Server (Write)
/// - Connection to Control server (Notify)
pub struct ActiveLookClient<TxActiveLook, RxActiveLook, Ctrl>
where
    TxActiveLook: Read,
    RxActiveLook: Write,
    Ctrl: Read,
{
    /// Client Rx is connected to ActiveLook Tx
    rx: TxActiveLook,
    /// Client Tx is connected to ActiveLook Rx
    tx: RxActiveLook,
    ctrl: Ctrl,
    /// Sequence number
    query_id: u32,
}

/// Protocol implementation
/// https://github.com/ActiveLook/Activelook-API-Documentation/blob/fw-4.12.0_doc-revA/ActiveLook_API.md#35-control-server
impl<TxActiveLook, RxActiveLook, Ctrl> ActiveLookClient<TxActiveLook, RxActiveLook, Ctrl>
where
    TxActiveLook: Read,
    RxActiveLook: Write,
    Ctrl: Read,
{
    pub fn new(rx: TxActiveLook, tx: RxActiveLook, ctrl: Ctrl) -> Self {
        Self {
            rx,
            tx,
            ctrl,
            query_id: 0,
        }
    }

    /// Send a command
    pub fn send(&mut self, cmd: &impl Serializable) -> Result<(), ProtocolError> {
        self.query_id += 1;
        debug!("Sending command id {}", cmd.id().expect("Not a command?"));
        let packet = Packet::new_with_query_id(cmd, &self.query_id.to_be_bytes());
        let res = self.tx.write(&packet.to_bytes()[..]);
        match res {
            Ok(_) => Ok(()),
            Err(error) => {
                error!("{:?}", error);
                Err(ProtocolError::EmbeddedIOError)
            }
        }
    }

    pub fn send_command_expect_response(
        &mut self,
        cmd: &impl Serializable,
    ) -> Result<Response, ProtocolError> {
        self.query_id += 1;
        debug!(
            "Sending command id {}, expecting Response",
            cmd.id().expect("Not a command?")
        );
        let packet = Packet::new_with_query_id(cmd, &self.query_id.to_be_bytes());
        let res = self.tx.write(&packet.to_bytes()[..]);
        if let Err(error) = res {
            return Err(ProtocolError::EmbeddedIOError);
        }

        let mut response_pkt: ResponsePacket;
        loop {
            let resp = self.read_tx_char();
            if let Ok(pkt) = resp {
                response_pkt = pkt;
                break;
            }
        }
        debug!("Received response {:?}", &response_pkt.data);
        if let Some(id) = response_pkt.query_id {
            if id.len() != core::mem::size_of::<u32>() {
                return Err(ProtocolError::IncorrectQueryId);
            }
            // Here unwrap() is safe, because we checked the vec length beforehand
            if u32::from_be_bytes(id.try_into().unwrap()) == self.query_id {
                Ok(response_pkt.data)
            } else {
                Err(ProtocolError::IncorrectQueryId)
            }
        } else {
            Err(ProtocolError::IncorrectQueryId)
        }
    }

    // Get notification on TX characteristic
    pub fn read_tx_char(&mut self) -> Result<ResponsePacket, ProtocolError> {
        let mut rxbuf = [0; PACKET_MAX_SIZE];
        if let Ok(len) = self.rx.read(&mut rxbuf) {
            ResponsePacket::from_bytes(&rxbuf[..len])
        } else {
            Err(ProtocolError::Empty)
        }
    }

    // Get notification on TX characteristic
    pub fn read_ctrl_char(&mut self) -> Result<u8, ProtocolError> {
        let mut rxbuf = [0; PACKET_MAX_SIZE];
        if let Ok(_len) = self.ctrl.read(&mut rxbuf) {
            Ok(rxbuf[0])
        } else {
            Err(ProtocolError::Empty)
        }
    }
}
