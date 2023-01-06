mod ble;

pub use ble::*;

#[async_trait::async_trait]
pub trait CameraControl {
    async fn set_shutter(&mut self, on: bool) -> Result<(), ()>;
}
