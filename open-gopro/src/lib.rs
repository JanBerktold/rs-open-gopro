mod ble;
mod http;

pub use ble::*;
pub use http::*;

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    Bluetooth(#[from] btleplug::Error),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
    #[error(transparent)]
    DnsError(#[from] mdns::Error),
    #[error("Base url for GoPro must be following http://0.0.0.0 format, got '{base}'")]
    BadBaseUrl { base: String },
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
