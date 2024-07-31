use std::{error::Error, fs, path::Path, time::Duration};

use again::RetryPolicy;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    multipart, Body,
};
use serde::Deserialize;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

#[derive(Deserialize, Debug)]
struct UploadResponse {
    id: String,
}

#[derive(Deserialize, Debug)]
pub struct StatusResponse {
    pub id: String,
    #[serde(default, rename = "errorCode")]
    pub error_code: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default, rename = "recognizedText")]
    pub recognized_text: Option<String>,
    #[serde(default)]
    pub progress: Option<u32>,
}

#[derive(Clone)]
pub struct ASRClient {
    url: String,
    auth_key: String,
    model: String,
    client: reqwest::Client,
}

impl ASRClient {
    pub fn new(url: &str, auth_key: &str, model: &str) -> Result<Self, Box<dyn Error>> {
        log::info!("Init ASRClient");
        log::info!("URL: {url}");
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()?;

        Ok(Self {
            url: url.to_string(),
            auth_key: auth_key.to_string(),
            client,
            model: model.to_string(),
        })
    }

    pub async fn upload(&self, file_path: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        log::info!("Send file to ASR: {}", file_path);
        let metadata = fs::metadata(file_path)?;
        let file_size = metadata.len();
        let timeout = get_timeout(file_size);
        log::info!("timeout: {:?}", timeout);
        let file_path = file_path.to_string();
        let fc = file_path.clone();
        let file_name = Path::new(fc.as_str())
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let file = File::open(file_path.clone())
            .await
            .map_err(|err| format!("can't open file {}: {}", file_path, err))?;
        let stream = FramedRead::new(file, BytesCodec::new());
        let file_body = Body::wrap_stream(stream);
        // let file_name = file_name.clone();
        let some_file = multipart::Part::stream(file_body)
            .file_name(file_name)
            .mime_str("audio/bin")
            .map_err(|err| format!("can't prepare multipart: {}", err))?;

        let form = multipart::Form::new()
            .text("recognizer", self.model.clone())
            .text("numberOfSpeakers", "")
            .part("file", some_file);

        let mut headers = HeaderMap::new();
        if !self.auth_key.is_empty() {
            log::info!("init auth header");
            headers.try_insert(
                HeaderName::from_static("authorization"),
                HeaderValue::from_str(format!("Key {}", self.auth_key).as_str())?,
            )?;
        }
        headers.try_insert(
            HeaderName::from_static("accept"),
            HeaderValue::from_str("application/json")?,
        )?;
        log::info!("init hreaders");
        let url = format!("{}/transcriber/upload", self.url);
        log::info!("call: {}", url);
        let res = self
            .client
            .post(url)
            .headers(headers)
            .multipart(form)
            .timeout(get_timeout(file_size))
            .send()
            .await?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or("can't read body".into());
            return Err(format!("Request failed with status: {}\n{}", status, body).into());
        }
        let parsed_json: UploadResponse = res.json().await?;
        Ok::<String, Box<dyn Error + Send + Sync>>(parsed_json.id)
    }

    pub async fn status(&self, id: &str) -> Result<StatusResponse, Box<dyn Error + Send + Sync>> {
        log::info!("check status: {}", id);
        let mut headers = HeaderMap::new();
        headers.try_insert(
            HeaderName::from_static("accept"),
            HeaderValue::from_str("application/json")?,
        )?;
        let url = format!("{}/status.service/status/{}", self.url, id);

        let policy = RetryPolicy::exponential(Duration::from_millis(100))
            .with_max_retries(3)
            .with_jitter(true);

        let res = policy
            .retry_if(
                || async {
                    log::info!("call: {}", url);
                    self
                        .client
                        .get(&url)
                        .headers(headers.clone()) // Clone headers for each retry
                        .timeout(Duration::from_secs(10))
                        .send()
                        .await
                },
                reqwest::Error::is_status,
            )
            .await?;
        if res.status().is_success() {
            log::info!("call ok");
            let content = res.text().await?;
            let parsed_json: StatusResponse = serde_json::from_str(&content)
                .map_err(|err| format!("can't deserialize '{}': {}", content, err))?;
            Ok::<StatusResponse, Box<dyn Error + Send + Sync>>(parsed_json)
        } else {
            log::info!("call failed");
            let status = res.status();
            let body = res.text().await.unwrap_or("can't read body".into());
            Err(format!("Request failed with status: {}\n{}", status, body).into())
        }
    }
}

fn get_timeout(file_size: u64) -> Duration {
    let file_size_mb = file_size as f64 / (1024.0 * 1024.0);
    Duration::from_secs(10) + Duration::from_secs_f64(file_size_mb * 0.5)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(1024 * 1024, Duration::from_secs(10) + Duration::from_secs_f64(1.0 * 0.5); "1 MB")]
    #[test_case(10 * 1024 * 1024, Duration::from_secs(15); "10 MB")]
    #[test_case(100 * 1024 * 1024, Duration::from_secs(60); "100 MB")]
    fn test_get_timeout(file_size: u64, wanted: Duration) {
        let actual = get_timeout(file_size);
        assert_eq!(wanted, actual)
    }
}
