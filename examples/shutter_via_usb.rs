use open_gopro::{CameraControl, HttpCamera};

#[tokio::main]
async fn main() {
    let mut camera = HttpCamera::new_usb().await.unwrap();

    camera.set_shutter(true).await.unwrap();
}
