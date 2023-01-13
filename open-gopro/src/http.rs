use std::time::Duration;

use futures_util::StreamExt;
use mdns::RecordKind;

use crate::CameraControl;

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
        const SERVICE_NAME: &'static str = "_gopro-web._tcp.local";
        const QUERY_INTERVAL: std::time::Duration = Duration::from_secs(1);

        let stream = mdns::discover::all(SERVICE_NAME, QUERY_INTERVAL)?.listen();
        tokio::pin!(stream);

        while let Some(Ok(response)) = stream.next().await {
            for record in response.records() {
                match record.kind {
                    RecordKind::A(address) => {
                        return Self::new_custom_address(&address.to_string())
                    }
                    _ => {}
                }
            }
        }

        unreachable!()
    }

    pub fn new_custom_address(base: &str) -> Result<Self, crate::Error> {
        let mut base = reqwest::Url::parse(base)?;
        base.set_port(Some(8080))
            .map_err(|_| crate::Error::BadBaseUrl {
                base: base.to_string(),
            })?;

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
}
