use crate::propellant::assets;

pub struct AssetCollection<T: assets::Asset> {
    assets: Vec<T>,
    lookup: std::collections::HashMap<String, usize>,
}

impl<T: assets::Asset> AssetCollection<T> {
    /// Load the assets from a given directory.
    ///
    /// This will iterate over all files in that directory,
    /// and attempt to load that file as a .ron file for T.
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

        let lookup: std::collections::HashMap<_, _> = assets
            .iter()
            .enumerate()
            .map(|(id, asset)| (asset.name().to_string(), id))
            .collect();

        Ok(Self { assets, lookup })
    }

    /// Get the number of loaded assets for this collection.
    pub fn count(&self) -> usize {
        self.assets.len()
    }

    /// Get a view of all the assets of the collection.
    pub fn assets(&self) -> &[T] {
        self.assets.as_slice()
    }

    /// Get a view of all the assets of the collection.
    pub fn assets_mut(&mut self) -> &mut [T] {
        self.assets.as_mut_slice()
    }

    /// Attempt to get an asset in the collection by its name.
    pub fn get_handle_from_name(&self, name: &str) -> Option<assets::AssetHandle<T>> {
        let id = self.lookup.get(name)?;
        Some(assets::AssetHandle::new(*id))
    }
}
