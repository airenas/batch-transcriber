use std::{error::Error, fs, path::PathBuf};

#[derive(Clone)]
pub struct Filer {
    base_dir: String,
}

impl Filer {
    pub fn new(base_dir: String) -> Self {
        log::info!("Creating new Filer with base dir: {}", base_dir);
        Self { base_dir }
    }

    pub fn move_working(&self, file: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut source_path = PathBuf::from(self.base_dir.as_str());
        source_path.extend(&["incoming", file]);
        let f = source_path
            .to_str()
            .ok_or("Failed to convert path to string")?;
        log::info!("Adding file: {}", f);
        if !source_path.exists() {
            return Err(format!("File {f} does not exist").into());
        }

        let mut dest_path = PathBuf::from(self.base_dir.as_str());
        dest_path.extend(&["working", file]);
        let f_new = dest_path
            .to_str()
            .ok_or("Failed to convert path to string")?;
        if dest_path.exists() {
            return Err(format!("File {f_new} exists").into());
        }
        fs::rename(&source_path, &dest_path)
            .map_err(|err| format!("Can't move file: {} \u{017D} {}", file, err))?;
        log::info!("moved: {} -> {}", f, f_new);
        Ok(())
    }
}
