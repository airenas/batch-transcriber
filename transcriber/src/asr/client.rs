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
    pub fn new(
        url: &str,
        auth_key: &str,
        model: &str,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
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

    pub async fn upload(&self, file_path: &str) -> anyhow::Result<String> {
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
        let url = format!("{}/transcriber/upload", self.url);

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

        let policy = RetryPolicy::exponential(Duration::from_secs(2))
            .with_max_retries(3)
            .with_jitter(true);

        let res = policy
            .retry_if(
                || async {
                    log::info!("prepare file: {}", file_path);
                    let file = File::open(file_path.clone())
                        .await
                        .map_err(|err| format!("can't open file {}: {}", file_path, err))?;
                    let stream = FramedRead::new(file, BytesCodec::new());
                    let file_body = Body::wrap_stream(stream);
                    let some_file = multipart::Part::stream(file_body)
                        .file_name(file_name.clone())
                        .mime_str("audio/bin")
                        .map_err(|err| format!("can't prepare multipart: {}", err))?;

                    let form = multipart::Form::new()
                        .text("recognizer", self.model.clone())
                        .text("numberOfSpeakers", "")
                        .part("file", some_file);

                    log::info!("call: {}", url);

                    let res = self
                        .client
                        .post(&url)
                        .headers(headers.clone())
                        .multipart(form)
                        .timeout(timeout)
                        .send()
                        .await?;
                    res.error_for_status()
                        .map_err(|err| Box::new(err) as Box<dyn Error + Send + Sync>)
                },
                is_retry_err,
            )
            .await
            .map_err(anyhow::Error::msg)?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or("can't read body".into());
            return Err(format!("Request failed with status: {}\n{}", status, body))
                .map_err(anyhow::Error::msg);
        }
        let parsed_json: UploadResponse = res.json().await?;
        Ok(parsed_json.id)
    }

    pub async fn status(&self, id: &str) -> Result<StatusResponse, Box<dyn Error + Send + Sync>> {
        log::info!("check status: {}", id);
        let mut headers = HeaderMap::new();
        headers.try_insert(
            HeaderName::from_static("accept"),
            HeaderValue::from_str("application/json")?,
        )?;
        let url = format!("{}/status.service/status/{}", self.url, id);

        let policy = RetryPolicy::exponential(Duration::from_secs(1))
            .with_max_retries(3)
            .with_jitter(true);

        let res = policy
            .retry_if(
                || async {
                    log::info!("call: {}", url);
                    let res = self
                        .client
                        .get(&url)
                        .headers(headers.clone()) // Clone headers for each retry
                        .timeout(Duration::from_secs(10))
                        .send()
                        .await?;
                    res.error_for_status()
                        .map_err(|err| Box::new(err) as Box<dyn Error + Send + Sync>)
                },
                is_retry_err,
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

    pub async fn result(&self, id: &str, file_name: &str) -> anyhow::Result<String> {
        log::info!("load file: {}, {}", id, file_name);
        let mut headers = HeaderMap::new();
        headers.try_insert(
            HeaderName::from_static("accept"),
            HeaderValue::from_str("text/plain")?,
        )?;
        let url = format!("{}/result.service/result/{}/{}", self.url, id, file_name);

        let policy = RetryPolicy::exponential(Duration::from_secs(1))
            .with_max_retries(3)
            .with_jitter(true);

        let res = policy
            .retry_if(
                || async {
                    log::info!("call: {}", url);
                    let res = self
                        .client
                        .get(&url)
                        .headers(headers.clone())
                        .timeout(Duration::from_secs(15))
                        .send()
                        .await?;
                    res.error_for_status()
                        .map_err(|err| Box::new(err) as Box<dyn Error + Send + Sync>)
                },
                is_retry_err,
            )
            .await
            .map_err(anyhow::Error::msg)?;
        if res.status().is_success() {
            log::info!("call ok");
            let content = res.text().await?;
            Ok(content)
        } else {
            log::info!("call failed");
            let status = res.status();
            let body = res.text().await.unwrap_or("can't read body".into());
            Err(anyhow::anyhow!("Request failed with status: {}\n{}", status, body))
        }
    }
}

#[allow(clippy::borrowed_box)]
fn is_retry_err(err: &Box<dyn Error + Send + Sync>) -> bool {
    log::info!("retry? {:?}", err);
    if let Some(reqwest_err) = err.downcast_ref::<reqwest::Error>() {
        log::info!("reqwest err");
        if reqwest_err.is_timeout() {
            log::info!("timeout");
            return true;
        }
        if reqwest_err.is_status() {
            log::info!("status");
            if let Some(status) = reqwest_err.status() {
                log::info!("status code {}", status);
                if status.is_server_error() {
                    log::info!("retry");
                    return true;
                }
                if status == 429 || status == 404 {
                    log::info!("retry");
                    return true;
                }
            }
        }
    }
    if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
        log::info!("io err");
        if io_err.kind() == std::io::ErrorKind::TimedOut {
            return true;
        }
    }
    log::error!("don't retry");
    false
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
