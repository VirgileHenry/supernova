use crate::assets;

pub struct AssetCollection<Asset: assets::Asset> {
    assets: Vec<Asset>,
    lookup: std::collections::HashMap<String, usize>,
    storage: Asset::Storage,
}

impl<Asset: assets::Asset> AssetCollection<Asset> {
    /// Load the assets from a given directory.
    ///
    /// This will iterate over all files in that directory,
    /// and attempt to load that file as a .ron file for T.
    pub fn load<P: AsRef<std::path::Path>>(
        load_context: &assets::AssetLoadingContext,
        collection_directory: P,
    ) -> std::io::Result<Self> {
        let mut assets = Vec::new();
        let directory_display = collection_directory.as_ref().to_string_lossy();

        for entry in std::fs::read_dir(&collection_directory)? {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    log::warn!("Failed to read directory entry in {directory_display}: {e}");
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
            /* Fixme: ask the asset to have a particluar extentsion and only keep this ? */
            let data = match std::fs::read_to_string(entry.path()) {
                Ok(data) => data,
                Err(e) => {
                    log::warn!("Failed to read file content for file {:?}: {e}", entry.path());
                    continue;
                }
            };
            let asset: Asset = match ron::from_str::<Asset::Serialized>(data.as_str()) {
                Ok(serialized) => serialized.into(),
                Err(e) => {
                    log::warn!("Failed to load asset from file {:?}: {e}", entry.path());
                    continue;
                }
            };

            assets.push(asset);
        }

        /* Load the asset in their storage */
        let storage = <Asset::Storage as assets::storage::AssetStorage<Asset>>::load(load_context, assets.as_mut_slice())
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        let lookup: std::collections::HashMap<_, _> = assets
            .iter()
            .enumerate()
            .map(|(id, asset)| (asset.name().to_string(), id))
            .collect();

        Ok(Self { assets, lookup, storage })
    }

    /// Get the number of loaded assets for this collection.
    pub fn count(&self) -> usize {
        self.assets.len()
    }

    /// Get a view of all the assets of the collection.
    pub fn assets(&self) -> &[Asset] {
        self.assets.as_slice()
    }

    /// Get a view of all the assets of the collection.
    pub fn assets_mut(&mut self) -> &mut [Asset] {
        self.assets.as_mut_slice()
    }

    /// Attempt to get an asset in the collection by its name.
    pub fn get_handle_from_name(&self, name: &str) -> Option<assets::AssetHandle<Asset>> {
        let id = self.lookup.get(name)?;
        Some(assets::AssetHandle::new(*id))
    }

    /// Attempt to get an asset in the collection by its name.
    pub fn get_asset_from_handle(&self, handle: &assets::AssetHandle<Asset>) -> Option<&Asset> {
        self.assets.get(handle.id)
    }

    /// Access to the inner asset storage
    pub fn storage(&self) -> &Asset::Storage {
        &self.storage
    }
}

pub trait AssetCollectionImpl: std::any::Any {}

impl<T: assets::Asset + 'static> AssetCollectionImpl for AssetCollection<T> {}
