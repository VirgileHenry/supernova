mod error;

use crate::assets;
pub use error::AssetLoadError;

pub struct AssetManager {
    /// Handle to the Vulkan device.
    vk_device: crate::vulkan::VkDeviceHandle,
    /// Keep track of the asset directory
    asset_directory: std::path::PathBuf,
    /// Map from the asset type to the storage
    collections: std::collections::HashMap<std::any::TypeId, Box<dyn assets::collection::AssetCollectionImpl>>,
}

impl AssetManager {
    /// Load all the required game assets from a given directory.
    pub fn new<P: AsRef<std::path::Path>>(
        vk_device: crate::vulkan::VkDeviceHandle,
        asset_directory: P,
    ) -> Result<Self, AssetLoadError> {
        Ok(Self {
            vk_device: vk_device.clone(),
            asset_directory: asset_directory.as_ref().to_path_buf(),
            collections: std::collections::HashMap::new(),
        })
    }

    pub fn load<Asset: assets::Asset, P: AsRef<std::path::Path>>(&mut self, directory: P) -> Result<(), AssetLoadError> {
        let path = self.asset_directory.join(directory);
        let directory_display = path.to_string_lossy();

        log::info!("Loading assets \"{}\" from \"./{directory_display}\"", Asset::ASSET_KIND);

        let load_context = AssetLoadingContext {
            vk_device: &self.vk_device,
        };
        let collection = assets::collection::AssetCollection::<Asset>::load(&load_context, &path)?;
        log::info!(
            "Loaded {} assets in the \"{}\" collection",
            collection.count(),
            Asset::ASSET_KIND
        );

        let collection = Box::new(collection) as Box<dyn assets::collection::AssetCollectionImpl>;
        self.collections.insert(std::any::TypeId::of::<Asset>(), collection);

        Ok(())
    }

    /// Attempt to get an asset segment by its name.
    pub fn get_handle<Asset: assets::Asset>(&self, name: &str) -> Option<assets::AssetHandle<Asset>> {
        self.get_collection::<Asset>()?.get_handle_from_name(name)
    }

    /// Attempt to get a segment from it's handle.
    pub fn get_asset<Asset: assets::Asset>(&self, handle: &assets::AssetHandle<Asset>) -> Option<&Asset> {
        self.get_collection::<Asset>()?.get_asset_from_handle(handle)
    }

    /// Inner function to get a storage from an asset type
    pub fn get_collection<Asset: assets::Asset>(&self) -> Option<&assets::collection::AssetCollection<Asset>> {
        let key = std::any::TypeId::of::<Asset>();
        let dyn_collection: &dyn assets::collection::AssetCollectionImpl = &**self.collections.get(&key)?;
        let any: &dyn std::any::Any = dyn_collection; // upcast of the collection, not of the Box

        match any.downcast_ref::<assets::collection::AssetCollection<Asset>>() {
            Some(res) => Some(res),
            None => {
                log::error!("Invalid asset collection for asset type {}", std::any::type_name::<Asset>());
                None
            }
        }
    }
}

pub struct AssetLoadingContext<'ctx> {
    pub vk_device: &'ctx crate::vulkan::VkDeviceHandle,
}
