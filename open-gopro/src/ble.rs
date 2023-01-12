use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Arc;

use btleplug::api::{bleuuid::uuid_from_u16, Characteristic, Peripheral as _, ScanFilter};
use btleplug::platform::Peripheral;
use tokio::sync::mpsc::Receiver;
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::CameraControl;

lazy_static::lazy_static! {
    // See https://gopro.github.io/OpenGoPro/ble_2_0#services-and-characteristics for these.
    static ref WIFI_ACCESS_POINT_SERVICE: Uuid =
        Uuid::try_parse("b5f90001-aa8d-11e3-9046-0002a5d5c51b").unwrap();

    static ref CONTROL_QUERY_SERVICE: Uuid = uuid_from_u16(0xFEA6);

    static ref COMMAND_REQ_CHARACTERISTIC: Uuid = Uuid::try_parse("b5f90072-aa8d-11e3-9046-0002a5d5c51b").unwrap();
    static ref COMMAND_RESP_CHARACTERISTIC: Uuid = Uuid::try_parse("b5f90073-aa8d-11e3-9046-0002a5d5c51b").unwrap();

    pub static ref GOPRO_SCANFILTER: ScanFilter = ScanFilter {
        services: Vec::from([*CONTROL_QUERY_SERVICE]),
    };
}

// The OpCode for each OpenGopro BLE command.
// See https://gopro.github.io/OpenGoPro/ble_2_0#commands.
#[repr(u8)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
enum CommandID {
    SetShutter = 0x01,
    Sleep = 0x05,
    SetDateTime = 0x0D,
    GetDateTime = 0x0E,
    SetLocalDateTime = 0x0F,
    GetLocalDateTime = 0x10,
    SetLiveStreamMode = 0x15,
    APControl = 0x17,
    HighlightMoment = 0x18,
    GetHardwareInfo = 0x3C,
    LoadPresetGroup = 0x3E,
    LoadPreset = 0x40,
    Analytics = 0x50,
    OpenGoPro = 0x51,
}

impl TryFrom<u8> for CommandID {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            val if val == CommandID::SetShutter as u8 => Ok(CommandID::SetShutter),
            val if val == CommandID::Sleep as u8 => Ok(CommandID::Sleep),
            val if val == CommandID::SetDateTime as u8 => Ok(CommandID::SetDateTime),
            val if val == CommandID::GetDateTime as u8 => Ok(CommandID::GetDateTime),
            val if val == CommandID::SetLocalDateTime as u8 => Ok(CommandID::SetLocalDateTime),
            val if val == CommandID::GetLocalDateTime as u8 => Ok(CommandID::GetLocalDateTime),
            val if val == CommandID::SetLiveStreamMode as u8 => Ok(CommandID::SetLiveStreamMode),
            val if val == CommandID::APControl as u8 => Ok(CommandID::APControl),
            val if val == CommandID::HighlightMoment as u8 => Ok(CommandID::HighlightMoment),
            val if val == CommandID::GetHardwareInfo as u8 => Ok(CommandID::GetHardwareInfo),
            val if val == CommandID::LoadPresetGroup as u8 => Ok(CommandID::LoadPresetGroup),
            val if val == CommandID::LoadPreset as u8 => Ok(CommandID::LoadPreset),
            val if val == CommandID::Analytics as u8 => Ok(CommandID::Analytics),
            val if val == CommandID::OpenGoPro as u8 => Ok(CommandID::OpenGoPro),
            _ => Err(()),
        }
    }
}

#[repr(u8)]
enum CommandResponseCode {
    Success = 0,
    Error = 1,
    InvalidParameter = 2,
    Unknown = 255,
}

impl From<u8> for CommandResponseCode {
    fn from(value: u8) -> Self {
        match value {
            val if val == CommandResponseCode::Success as u8 => CommandResponseCode::Success,
            val if val == CommandResponseCode::Error as u8 => CommandResponseCode::Error,
            val if val == CommandResponseCode::InvalidParameter as u8 => {
                CommandResponseCode::InvalidParameter
            }
            _ => CommandResponseCode::Unknown,
        }
    }
}

impl CommandResponseCode {
    fn is_error(&self) -> bool {
        match &self {
            CommandResponseCode::Success => false,
            _ => true,
        }
    }
}

pub struct BluetoothCamera {
    remote: Peripheral,

    command_req: Characteristic,

    command_notifications:
        Arc<tokio::sync::Mutex<HashMap<CommandID, tokio::sync::mpsc::Sender<Vec<u8>>>>>,
}

impl BluetoothCamera {
    pub async fn connect(p: Peripheral) -> Self {
        p.discover_services().await.unwrap();

        if !p.is_connected().await.unwrap() {
            p.connect().await.unwrap();
        }

        let mut characteristics = p.characteristics().into_iter();
        let command_resp = characteristics
            .find(|c| c.uuid == *COMMAND_RESP_CHARACTERISTIC)
            .unwrap();
        let command_req = characteristics
            .find(|c| c.uuid == *COMMAND_REQ_CHARACTERISTIC)
            .unwrap();

        p.subscribe(&command_resp).await.unwrap();

        let command_notifications = Arc::new(tokio::sync::Mutex::new(HashMap::<
            CommandID,
            tokio::sync::mpsc::Sender<Vec<u8>>,
        >::new()));

        let moved_command_notifications = command_notifications.clone();
        // TODO: Can we avoid a tokio dependency?
        let mut stream = p.notifications().await.unwrap();
        tokio::task::spawn(async move {
            while let Some(msg) = stream.next().await {
                match msg.uuid {
                    c if c == *COMMAND_RESP_CHARACTERISTIC => {
                        let mut notif = moved_command_notifications.lock().await;

                        let command_id = CommandID::try_from(msg.value[1]).unwrap();

                        match notif.entry(command_id) {
                            Entry::Occupied(sender) => {
                                sender.get().send(msg.value).await.unwrap();
                                sender.remove();
                            }
                            Entry::Vacant(_) => {
                                println!("got response {:#?}", msg);
                            }
                        }
                    }
                    other => println!("got unknown msg: {:#?}", other),
                }
            }
        });

        Self {
            remote: p,
            command_req,
            command_notifications,
        }
    }

    async fn wait_command_response(&self, command: CommandID) -> Receiver<Vec<u8>> {
        let (sender, receiver) = tokio::sync::mpsc::channel(1);

        {
            let mut entries = self.command_notifications.lock().await;
            entries.insert(command, sender);
        }

        receiver
    }
}

#[async_trait::async_trait]
impl CameraControl for BluetoothCamera {
    async fn set_shutter(&mut self, on: bool) -> Result<(), crate::Error> {
        let mut resp = self.wait_command_response(CommandID::SetShutter).await;

        self.remote
            .write(
                &self.command_req,
                &[
                    0x03,
                    CommandID::SetShutter as u8,
                    0x01,
                    if on { 0x01 } else { 0x00 },
                ],
                btleplug::api::WriteType::WithoutResponse,
            )
            .await?;

        let resp = resp.recv().await.unwrap();
        let code: CommandResponseCode = resp[2].into();

        if !code.is_error() {
            Ok(())
        } else {
            unimplemented!();
        }
    }
}
