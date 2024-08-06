use std::{collections::hash_map, path::Path};

use anyhow::anyhow;
use axum::{
    body::Bytes,
    extract::{self, Multipart, State},
    BoxError, Json,
};
use chrono::Local;
use scopeguard::guard;
use serde::Serialize;
use transcriber::{
    filer::file::{make_name, Filer},
    DIR_INCOMING, INFO_EXTENSION,
};

use super::error::ApiError;

use futures::Stream;

#[derive(Serialize, Clone)]
pub struct UploadResult {
    id: String,
}

pub async fn handler(
    State(filer): State<Filer>,
    mut multipart: Multipart,
) -> Result<extract::Json<UploadResult>, ApiError> {
    let mut values: hash_map::HashMap<String, String> = hash_map::HashMap::new();
    let mut saved_file: Option<String> = None;
    let saved_file1: Option<String> = None;
    
    
    let mut file_guard = guard(saved_file1, |saved_file| {
        tracing::debug!(value = saved_file, "guard run");
        if let Some(file) = saved_file {
            if let Err(err) = filer.delete(&file, DIR_INCOMING) {
                log::error!("{}", err);
            }
        }
    });

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| as_bad_request("can't parse multipart", err.into()))?
    {
        let name = field
            .name()
            .ok_or(anyhow!("can't parse multipart"))
            .map_err(|err| as_bad_request("can't parse multipart", err))?
            .to_string();
        let file_name = field.file_name().unwrap_or_default().to_string();
        if !file_name.is_empty() {
            validate_name(&file_name).map_err(err_bad_request)?;
            let saved = stream_to_file(&filer, &file_name, field).await?;
            saved_file = Some(saved.clone());
            file_guard.replace(saved);
        } else {
            let value = field
                .text()
                .await
                .map_err(|err| as_bad_request("can't parse multipart", err.into()))?;
            tracing::info!(name, value, "got");
            values.insert(name.to_string(), value);
        }
    }

    return match saved_file {
        Some(file) => {
            validate(&values).map_err(err_bad_request)?;

            values.insert("file".to_string(), file.to_string());

            let now = Local::now();
            let formatted = now.format("%Y-%m-%d %H:%M:%S").to_string();
            values.insert("time".to_string(), formatted);

            let data = make_data(&values)?;
            filer.save_txt(&make_name(&file, INFO_EXTENSION), DIR_INCOMING, &data)?;

            let res = UploadResult { id: file };
            file_guard.take();
            Ok(Json(res))
        }
        None => {
            return Err(ApiError::BadRequest(
                "no file".to_string(),
                "no file".to_string(),
            ));
        }
    };
}

fn as_bad_request(msg: &str, err: anyhow::Error) -> ApiError {
    ApiError::BadRequest(msg.to_string(), err.to_string())
}

fn err_bad_request(err: anyhow::Error) -> ApiError {
    ApiError::BadRequest(err.to_string(), "".to_string())
}

fn make_data(values: &hash_map::HashMap<String, String>) -> Result<String, anyhow::Error> {
    let mut data = String::new();
    data.push_str(&format!(
        "File     : {}\n",
        values.get("file").unwrap_or(&"".to_string())
    ));
    data.push_str(&format!(
        "Time     : {}\n",
        values.get("time").unwrap_or(&"".to_string())
    ));
    data.push_str(&format!(
        "Name     : {} {}\n",
        values.get("name").unwrap_or(&"".to_string()),
        values.get("surname").unwrap_or(&"".to_string())
    ));
    data.push_str(&format!(
        "Office   : {}\n",
        values.get("office").unwrap_or(&"".to_string())
    ));
    data.push_str(&format!(
        "Speakers : {}\n",
        values.get("speakers").unwrap_or(&"".to_string())
    ));
    Ok(data)
}

fn validate(values: &hash_map::HashMap<String, String>) -> Result<(), anyhow::Error> {
    if !values.contains_key("name") || values.get("name").is_some_and(|v| v.is_empty()) {
        return Err(anyhow::Error::msg("no name"));
    }
    if !values.contains_key("surname") || values.get("surname").is_some_and(|v| v.is_empty()) {
        return Err(anyhow::Error::msg("no surname"));
    }
    if !values.contains_key("office") || values.get("office").is_some_and(|v| v.is_empty()) {
        return Err(anyhow::Error::msg("no office"));
    }
    if !values.contains_key("speakers") || values.get("speakers").is_some_and(|v| v.is_empty()) {
        return Err(anyhow::Error::msg("no speakers"));
    }
    Ok(())
}

async fn stream_to_file<S, E>(f: &Filer, path: &str, stream: S) -> Result<String, ApiError>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    let name = f.non_existing_name(path, DIR_INCOMING)?;
    f.save_stream(&name, DIR_INCOMING, stream).await?;
    Ok(name)
}

fn validate_name(f_name: &str) -> Result<(), anyhow::Error> {
    let path = Path::new(f_name);
    let ext: String = match path.extension() {
        Some(e) => e.to_string_lossy().to_lowercase(),
        None => return Err(anyhow::anyhow!("audio expected")),
    };
    if ext != "mp3" && ext != "wav" && ext != "m4a" {
        return Err(anyhow::anyhow!("audio expected"));
    }
    Ok(())
}
