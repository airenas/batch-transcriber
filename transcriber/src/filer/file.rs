use std::{fs, path::{Path, PathBuf}};

#[derive(Clone)]
pub struct Filer {
    base_dir: String,
}

impl Filer {
    pub fn new(base_dir: String) -> Self {
        log::info!("Creating new Filer with base dir: {}", base_dir);
        Self { base_dir }
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

    pub fn move_to(&self, f_name: &str, dir_from: &str, dir_to: &str) -> anyhow::Result<()> {
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
        dest_path.extend(&[dir_to, f_name]);
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
