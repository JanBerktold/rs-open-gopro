mod ble;
mod http;

pub use ble::*;
pub use http::*;

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    Bluetooth(#[from] btleplug::Error),
}

#[async_trait::async_trait]
pub trait CameraControl: private::Sealed {
    async fn set_shutter(&mut self, on: bool) -> Result<(), crate::Error>;
}

mod private {
    pub trait Sealed {}

    impl Sealed for crate::HttpCamera {}
    impl Sealed for crate::BluetoothCamera {}
}
