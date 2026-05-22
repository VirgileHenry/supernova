use crate::propellant::assets;

pub struct AssetCollection<T: assets::Asset> {
    assets: Vec<T>,
}

impl<T: assets::Asset> AssetCollection<T> {
    pub fn load(collection_directory: &str) -> std::io::Result<Self> {
        let mut assets = Vec::new();

        for entry in std::fs::read_dir(collection_directory)? {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    log::warn!("Failed to read directory entry in {collection_directory}: {e}");
                    continue;
                }
            };
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(e) => {
                    log::warn!("Failed to read file type for file {:?}: {e}", entry.path());
                    continue;
                }
            };
            if !file_type.is_file() {
                continue;
            }
            let data = match std::fs::read_to_string(entry.path()) {
                Ok(data) => data,
                Err(e) => {
                    log::warn!("Failed to read file content for file {:?}: {e}", entry.path());
                    continue;
                }
            };
            let asset = match T::load(data.as_str()) {
                Ok(asset) => asset,
                Err(e) => {
                    log::warn!("Failed to load asset from file {:?}: {e}", entry.path());
                    continue;
                }
            };

            assets.push(asset);
        }

        Ok(Self { assets })
    }

    pub fn count(&self) -> usize {
        self.assets.len()
    }
}
