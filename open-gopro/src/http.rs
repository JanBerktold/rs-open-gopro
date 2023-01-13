/* Endpoints TODO:
 *gopro/camera/analytics/set_client_info
/gopro/camera/state
/gopro/camera/digital_zoom?percent=50
/gopro/camera/get_date_time
/gopro/media/gpmf?path=100GOPRO/XXX.JPG
/gopro/media/gpmf?path=100GOPRO/XXX.MP4
/gopro/media/hilight/file?path=100GOPRO/XXX.JPG
/gopro/media/hilight/file?path=100GOPRO/XXX.MP4&ms=2500
/gopro/media/hilight/remove?path=100GOPRO/XXX.JPG
/gopro/media/hilight/remove?path=100GOPRO/XXX.MP4&ms=2500
/gopro/media/hilight/moment
/gopro/media/info?path=100GOPRO/XXX.JPG
/gopro/media/info?path=100GOPRO/XXX.MP4
/gopro/media/list
/gopro/media/screennail?path=100GOPRO/XXX.JPG
/gopro/media/screennail?path=100GOPRO/XXX.MP4
/gopro/media/telemetry?path=100GOPRO/XXX.JPG
/gopro/media/telemetry?path=100GOPRO/XXX.MP4
/gopro/media/thumbnail?path=100GOPRO/XXX.JPG
/gopro/media/thumbnail?path=100GOPRO/XXX.MP4
/gopro/media/turbo_transfer?p=0
/gopro/media/turbo_transfer?p=1
/gp/gpSoftUpdate (plus data)
/gp/gpSoftUpdate (plus data)
/gp/gpSoftUpdate?request=canceled
/gp/gpSoftUpdate?request=delete
/gp/gpSoftUpdate?request=progress
/gp/gpSoftUpdate?request=showui
/gp/gpSoftUpdate?request=start
/gopro/version
/gopro/camera/presets/get
/gopro/camera/presets/load?id=305441741
/gopro/camera/presets/set_group?id=1000
/gopro/camera/presets/set_group?id=1001
/gopro/camera/presets/set_group?id=1002
/gopro/camera/control/set_ui_controller?p=0
/gopro/camera/control/set_ui_controller?p=2
/gopro/camera/set_date_time?date=2023_1_31&time=3_4_5
/gopro/camera/set_date_time?date=2023_1_31&time=3_4_5&tzone=-120&dst=1
/gopro/camera/stream/start
/gopro/camera/stream/stop
/gopro/webcam/exit
/gopro/webcam/preview
/gopro/webcam/start
/gopro/webcam/start?port=12345
/gopro/webcam/start?res=12&fov=0
/gopro/webcam/status
/gopro/webcam/stop
/gopro/webcam/version
/gopro/camera/control/wired_usb?p=0
/gopro/camera/control/wired_usb?p=1
 *
 */
use std::time::Duration;

use crate::CameraControl;

use futures_util::StreamExt;
use mdns_sd::{ServiceDaemon, ServiceEvent};

pub struct HttpCamera {
    client: reqwest::Client,
    base: reqwest::Url,
}

impl HttpCamera {
    pub fn new_wifi() -> Result<Self, crate::Error> {
        // See https://gopro.github.io/OpenGoPro/http_2_0#wifi-2.
        const WIFI_URL: &'static str = "http://10.5.5.9:8080";
        Self::new_custom_address(WIFI_URL)
    }

    pub async fn new_usb() -> Result<Self, crate::Error> {
        const SERVICE_NAME: &'static str = "_gopro-web._tcp.local.";
        const QUERY_INTERVAL: std::time::Duration = Duration::from_secs(1);

        let mdns = ServiceDaemon::new().expect("Failed to create daemon");

        let receiver = mdns.browse(SERVICE_NAME).expect("Failed to browse");

        while let Ok(event) = receiver.recv_async().await {
            match event {
                ServiceEvent::ServiceResolved(info) => {
                    if let Some(address) = info.get_addresses().iter().next() {
                        return Self::new_custom_address(&format!(
                            "http://{}:{}",
                            address.to_string(),
                            info.get_port()
                        ));
                    }
                }
                _ => {}
            }
        }

        unreachable!()
    }

    pub fn new_custom_address(base: &str) -> Result<Self, crate::Error> {
        let mut base = reqwest::Url::parse(base)?;

        if base.port().is_none() {
            base.set_port(Some(8080))
                .map_err(|_| crate::Error::BadBaseUrl {
                    base: base.to_string(),
                })?;
        }

        Ok(HttpCamera {
            client: reqwest::Client::new(),
            base,
        })
    }
}

#[async_trait::async_trait]
impl CameraControl for HttpCamera {
    async fn set_shutter(&mut self, on: bool) -> Result<(), crate::Error> {
        let path = format!(
            "/gopro/camera/shutter/{}",
            if on { "start" } else { "stop" }
        );

        self.client
            .get(self.base.join(&path)?)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    async fn keep_alive(&mut self) -> Result<(), crate::Error> {
        self.client
            .get(self.base.join("/gopro/camera/keep_alive")?)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}
