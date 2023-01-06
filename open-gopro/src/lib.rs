mod ble;
mod http;

pub use ble::*;
pub use http::*;

#[async_trait::async_trait]
pub trait CameraControl {
    async fn set_shutter(&mut self, on: bool) -> Result<(), ()>;
}
