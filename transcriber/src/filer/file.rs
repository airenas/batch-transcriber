use std::{
    fs,
    path::{Path, PathBuf},
};

use axum::{body::Bytes, BoxError};
use futures::Stream;
use tokio::{fs::File, io::{self, BufWriter}};
use futures::TryStreamExt;
use tokio_util::io::StreamReader;

#[derive(Clone)]
pub struct Filer {
    base_dir: String,
}

impl Filer {
    pub fn new(base_dir: &str) -> Self {
        log::info!("Creating new Filer with base dir: {}", base_dir);
        Self {
            base_dir: base_dir.to_string(),
        }
    }

    pub fn save_txt(&self, f_name: &str, folder: &str, txt: &str) -> anyhow::Result<()> {
        log::info!("saving file: {}", f_name);
        let mut dest_path = PathBuf::from(self.base_dir.as_str());
        dest_path.extend(&[folder, f_name]);
        let f_new = dest_path
            .to_str()
            .ok_or("Failed to convert path to string")
            .map_err(anyhow::Error::msg)?;
        self.try_create_folder(&dest_path)?;
        fs::write(f_new, txt)
            .map_err(|err| format!("Can't write file: {}\n{}", f_new, err))
            .map_err(anyhow::Error::msg)?;
        log::info!("saved: {}", f_new);
        Ok(())
    }

    pub async fn save_stream<S, E>(
        &self,
        f_name: &str,
        folder: &str,
        stream: S,
    ) -> anyhow::Result<()>
    where
        S: Stream<Item = Result<Bytes, E>>,
        E: Into<BoxError>,
    {
        log::info!("saving file: {}", f_name);
        let mut dest_path = PathBuf::from(self.base_dir.as_str());
        dest_path.extend(&[folder, f_name]);
        let f_new = dest_path
            .to_str()
            .ok_or("Failed to convert path to string")
            .map_err(anyhow::Error::msg)?;
        self.try_create_folder(&dest_path)?;

        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);
        
        let mut file = BufWriter::new(File::create(&dest_path).await?);
        futures::pin_mut!(body_reader);

        // Copy the body into the file.
        tokio::io::copy(&mut body_reader, &mut file).await?;
        log::info!("saved: {}", f_new);
        Ok(())
    }

    pub fn non_existing_name(&self, f_name: &str, folder: &str) -> anyhow::Result<String> {
        let mut i = 0;
        loop {
            let new_name = make_new_name(f_name, i);
            let mut dest_path = PathBuf::from(self.base_dir.as_str());
            dest_path.extend(&[folder, &new_name]);
            if !dest_path.exists() {
                return Ok(new_name);
            }
            i += 1;
        }
    }

    fn try_create_folder(&self, dest_path: &Path) -> anyhow::Result<()> {
        if let Some(dest_dir) = dest_path.parent() {
            if !dest_dir.exists() {
                log::info!("creating: {:?}", dest_dir);
                fs::create_dir_all(dest_dir).map_err(|err| {
                    anyhow::anyhow!("Failed to create directory {}: {}", dest_dir.display(), err)
                })?;
            }
        }
        Ok(())
    }

    pub fn move_to(
        &self,
        f_name: &str,
        to_name: &str,
        dir_from: &str,
        dir_to: &str,
    ) -> anyhow::Result<()> {
        let mut source_path = PathBuf::from(self.base_dir.as_str());
        source_path.extend(&[dir_from, f_name]);
        let f = source_path
            .to_str()
            .ok_or("Failed to convert path to string")
            .map_err(anyhow::Error::msg)?;
        log::info!("Adding file: {}", f);
        if !source_path.exists() {
            return Err(anyhow::anyhow!("File {f} does not exist"));
        }
        let mut dest_path = PathBuf::from(self.base_dir.as_str());
        dest_path.extend(&[dir_to, to_name]);
        let f_new = dest_path
            .to_str()
            .ok_or("Failed to convert path to string")
            .map_err(anyhow::Error::msg)?;

        self.try_create_folder(&dest_path)?;
        if dest_path.exists() {
            fs::remove_file(&dest_path).map_err(|err| {
                anyhow::anyhow!("Failed to remove existing file {}: {}", f_new, err)
            })?;
        }
        fs::rename(&source_path, &dest_path)
            .map_err(|err| format!("Can't move file: {}\n{}", f, err))
            .map_err(anyhow::Error::msg)?;
        log::info!("moved: {} -> {}", f, f_new);
        Ok(())
    }
}

pub fn make_name(f_name: &str, ext: &str) -> String {
    let path = Path::new(f_name);
    let mut new_path = PathBuf::from(path);
    let mut clean_ext = ext;
    while clean_ext.starts_with('.') {
        clean_ext = &clean_ext[1..]
    }
    new_path.set_extension(clean_ext);
    new_path.to_string_lossy().into_owned()
}

pub fn make_new_name(f_name: &str, num: i32) -> String {
    if num == 0 {
        return f_name.to_string();
    }
    let path = Path::new(f_name);
    let new_ext = match path.extension() {
        Some(e) => {
            format!("{}.{}", num, e.to_string_lossy())
        }
        None => format!("{}", num),
    };
    let mut new_path = PathBuf::from(path);
    new_path.set_extension(new_ext);
    new_path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("document.wav", ".txt", "document.txt"; "change extension")]
    #[test_case("archive.tar.gz", ".lat", "archive.tar.lat"; "Change last extension in multi-extension file")]
    fn test_make_name(original: &str, new_ext: &str, expected: &str) {
        let actual = make_name(original, new_ext);
        assert_eq!(expected, actual);
    }

    #[test_case("document.wav", 0, "document.wav"; "same")]
    #[test_case("archive.tar.gz", 0, "archive.tar.gz"; "several extensions")]
    #[test_case("document.wav", 1, "document.1.wav"; "same add")]
    #[test_case("archive.tar.gz", 20, "archive.tar.20.gz"; "several extensions add")]
    fn test_make_new_name(original: &str, i: i32, expected: &str) {
        let actual = make_new_name(original, i);
        assert_eq!(expected, actual);
    }
}
