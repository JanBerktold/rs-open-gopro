use std::collections::hash_map::Entry;
use std::collections::{hash_map::OccupiedEntry, HashMap};
use std::sync::Arc;

use btleplug::api::{
    bleuuid::{self, uuid_from_u16, BleUuid},
    Characteristic, Peripheral as _, ScanFilter,
};
use btleplug::platform::Peripheral;
use tokio::sync::mpsc::{Receiver, UnboundedReceiver};
use uuid::Uuid;

lazy_static::lazy_static! {
    // See https://gopro.github.io/OpenGoPro/ble_2_0#services-and-characteristics for these.
    static ref WIFI_ACCESS_POINT_SERVICE: Uuid =
        Uuid::try_parse("b5f90001-aa8d-11e3-9046-0002a5d5c51b").unwrap();

    static ref CONTROL_QUERY_SERVICE: Uuid = uuid_from_u16(0xFEA6);

    static ref COMMAND_REQ_CHARACTERISTIC: Uuid = Uuid::try_parse("b5f90072-aa8d-11e3-9046-0002a5d5c51b").unwrap();
    static ref COMMAND_RESP_CHARACTERISTIC: Uuid = Uuid::try_parse("b5f90073-aa8d-11e3-9046-0002a5d5c51b").unwrap();

    pub static ref GOPRO_SCANFILTER: ScanFilter = ScanFilter {
        services: Vec::from([CONTROL_QUERY_SERVICE.clone()]),
    };
}

pub struct Camera {
    remote: Peripheral,

    command_notifications: Arc<tokio::sync::Mutex<HashMap<u8, tokio::sync::mpsc::Sender<Vec<u8>>>>>,
}

impl Camera {
    pub async fn connect(p: Peripheral) -> Self {
        p.discover_services().await.unwrap();

        if !p.is_connected().await.unwrap() {
            p.connect().await.unwrap();
        }

        let c = p
            .characteristics()
            .into_iter()
            .find(|c| c.uuid == *COMMAND_RESP_CHARACTERISTIC)
            .unwrap();

        p.subscribe(&c).await.unwrap();

        let command_notifications = Arc::new(tokio::sync::Mutex::new(HashMap::<
            u8,
            tokio::sync::mpsc::Sender<Vec<u8>>,
        >::new()));

        use tokio_stream::StreamExt;

        let moved_command_notifications = command_notifications.clone();
        // TODO: Can we avoid a tokio dependency?
        let mut stream = p.notifications().await.unwrap();
        tokio::task::spawn(async move {
            while let Some(msg) = stream.next().await {
                match msg.uuid {
                    c if c == *COMMAND_RESP_CHARACTERISTIC => {
                        let mut notif = moved_command_notifications.lock().await;

                        match notif.entry(msg.value[1]) {
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
            command_notifications,
        }
    }

    async fn wait_command_response(&self, command: u8) -> Receiver<Vec<u8>> {
        let (sender, receiver) = tokio::sync::mpsc::channel(1);

        {
            let mut entries = self.command_notifications.lock().await;
            entries.insert(command, sender);
        }

        receiver
    }
}

impl Camera {
    pub async fn set_shutter(&mut self, on: bool) {
        let c = self
            .remote
            .characteristics()
            .into_iter()
            .find(|c| c.uuid == *COMMAND_REQ_CHARACTERISTIC)
            .unwrap();

        let mut resp = self.wait_command_response(0x01).await;

        self.remote
            .write(
                &c,
                &[0x03, 0x01, 0x01, if on { 0x01 } else { 0x00 }],
                btleplug::api::WriteType::WithoutResponse,
            )
            .await
            .unwrap();

        let resp = resp.recv().await;
        println!("got resp {:#?}", resp);
    }
}
