use btleplug::{
    api::Manager as _,
    api::{Central, Peripheral as _, ScanFilter},
    platform::*,
};
use open_gopro::Camera;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let manager = Manager::new().await?;

    // get the first bluetooth adapter
    let adapters = manager.adapters().await?;
    let central = adapters.first().unwrap();

    // start scanning for devices
    central
        .start_scan(open_gopro::GOPRO_SCANFILTER.clone())
        .await?;

    // instead of waiting, you can use central.events() to get a stream which will
    // notify you of new devices, for an example of that see examples/event_driven_discovery.rs
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let mut peripheral = central.peripherals().await?;

    let camera = peripheral.pop().unwrap();
    let camera = Camera::connect(camera).await;

    loop {
        camera.set_shutter(true).await;
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        camera.set_shutter(false).await;
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    Ok(())
}
