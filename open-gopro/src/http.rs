use crate::CameraControl;

pub struct HttpCamera {
    client: reqwest::Client,
    base: reqwest::Url,
}

impl HttpCamera {
    pub fn new_wifi() -> Self {
        // See https://gopro.github.io/OpenGoPro/http_2_0#wifi-2.
        let base =
            reqwest::Url::parse("http://10.5.5.9:8080").expect("static URL is known to be good");
        Self::new_custom_address(base)
    }

    pub fn new_custom_address(base: reqwest::Url) -> Self {
        HttpCamera {
            client: reqwest::Client::new(),
            base,
        }
    }
}

#[async_trait::async_trait]
impl CameraControl for HttpCamera {
    async fn set_shutter(&mut self, on: bool) -> Result<(), ()> {
        unimplemented!()
    }
}
