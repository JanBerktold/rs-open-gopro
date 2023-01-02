use btleplug::api::{
    bleuuid::{self, uuid_from_u16, BleUuid},
    Peripheral as _, ScanFilter,
};
use btleplug::platform::Peripheral;
use uuid::Uuid;

lazy_static::lazy_static! {
    // See https://gopro.github.io/OpenGoPro/ble_2_0#services-and-characteristics for these.
    static ref WIFI_ACCESS_POINT_SERVICE: Uuid =
        Uuid::try_parse("b5f90001-aa8d-11e3-9046-0002a5d5c51b").unwrap();

    static ref CONTROL_QUERY_SERVICE: Uuid = uuid_from_u16(0xFEA6);

    static ref COMMAND_CHARACTERISTIC: Uuid = Uuid::try_parse("b5f90072-aa8d-11e3-9046-0002a5d5c51b").unwrap();

    pub static ref GOPRO_SCANFILTER: ScanFilter = ScanFilter {
        services: Vec::from([CONTROL_QUERY_SERVICE.clone()]),
    };
}

pub struct Camera {
    remote: Peripheral,
}

impl Camera {
    pub async fn connect(p: Peripheral) -> Self {
        p.discover_services().await.unwrap();

        if !p.is_connected().await.unwrap() {
            p.connect().await.unwrap();
        }

        Self { remote: p }
    }
}

impl Camera {
    pub async fn set_shutter(&self, on: bool) {
        let c = self
            .remote
            .characteristics()
            .into_iter()
            .find(|c| c.uuid == *COMMAND_CHARACTERISTIC)
            .unwrap();

        self.remote
            .write(
                &c,
                &[0x03, 0x01, 0x01, if on { 0x01 } else { 0x00 }],
                btleplug::api::WriteType::WithoutResponse,
            )
            .await
            .unwrap();
    }
}
