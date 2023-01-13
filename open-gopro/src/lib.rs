//! A Rust crate that allows interacting with GoPro cameras via the Open GoPro standard.
// #![deny(missing_docs)]

mod ble;
mod http;

pub use ble::*;
pub use http::*;

/// Any error case exposed by this crate.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    Bluetooth(#[from] btleplug::Error),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
    //#[error(transparent)]
    //DnsError(#[from] mdns::Error),
    #[error("Base url for GoPro must be following http://0.0.0.0 format, got '{base:?}'")]
    BadBaseUrl { base: String },
}

#[async_trait::async_trait]
pub trait CameraControl: private::Sealed {
    async fn set_shutter(&mut self, on: bool) -> Result<(), crate::Error>;

    /// Reset the keep alive time to keep the camera from entering sleep mode.
    ///
    /// If the intention is to keep the camera alive forever, then the best practice
    /// is to send this command every 3 seconds.
    ///
    /// Also see <https://gopro.github.io/OpenGoPro/ble_2_0#keep-alive>
    async fn keep_alive(&mut self) -> Result<(), crate::Error>;
}

mod private {
    pub trait Sealed {}

    impl Sealed for crate::HttpCamera {}
    impl Sealed for crate::BluetoothCamera {}
}
