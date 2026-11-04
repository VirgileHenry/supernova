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

    pub fn load<T: assets::Asset, P: AsRef<std::path::Path>>(&mut self, directory: P) -> Result<(), AssetLoadError> {
        let directory_display = directory.as_ref().to_string_lossy();

        let path = self.asset_directory.join(&directory);
        log::info!("Loading assets \"{}\" from \"./{directory_display}\"", T::ASSET_KIND);

        let collection = assets::collection::AssetCollection::<T>::load(&path)?;
        log::info!("Loaded {} assets in the \"{}\" collection", collection.count(), T::ASSET_KIND);

        let collection = Box::new(collection) as Box<dyn assets::collection::AssetCollectionImpl>;
        self.collections.insert(std::any::TypeId::of::<T>(), collection);

        Ok(())
    }

    pub fn load_graphic<T, P>(&mut self, directory: P) -> Result<(), AssetLoadError>
    where
        T: assets::GraphicAsset,
        P: AsRef<std::path::Path>,
    {
        let directory_display = directory.as_ref().to_string_lossy();

        let path = self.asset_directory.join(&directory);
        log::info!("Loading assets \"{}\" from \"./{directory_display}\"", T::ASSET_KIND);

        let collection = assets::collection::AssetCollection::<T>::load(&path)?;
        log::info!("Loaded {} assets in the \"{}\" collection", collection.count(), T::ASSET_KIND);

        let gpu_buffer_size = collection.assets().iter().map(|asset| asset.size()).sum();

        if let Some(gpu_buffer_size) = std::num::NonZeroUsize::new(gpu_buffer_size) {
            let mut staging_buffer = crate::vulkan::StagingBuffer::create(&self.vk_device, gpu_buffer_size)?;

            let mut offset = 0;
            for asset in collection.assets() {
                asset.load(&mut staging_buffer, offset);
                offset += asset.size();
            }

            let mut storage_buffer = crate::vulkan::StorageBuffer::create(&self.vk_device, gpu_buffer_size)?;

            /* transfer staging to storage buffer */
            let destination_view = storage_buffer.entire_buffer();
            crate::vulkan::copy_buffer(
                &self.vk_device,
                &staging_buffer,
                &mut storage_buffer,
                staging_buffer.entire_buffer(),
                destination_view,
            )?;
            let bytes_size = crate::utils::format_byte_size(gpu_buffer_size.get());
            log::info!("\"{}\": allocated {} on the GPU", T::ASSET_KIND, bytes_size);
        } else {
            log::info!("\"{}\": no GPU allocation", T::ASSET_KIND);
        }

        let collection = Box::new(collection) as Box<dyn assets::collection::AssetCollectionImpl>;
        self.collections.insert(std::any::TypeId::of::<T>(), collection);

        Ok(())
    }

    /// Attempt to get an asset segment by its name.
    pub fn get_handle<T: assets::Asset>(&self, name: &str) -> Option<assets::AssetHandle<T>> {
        self.get_storage::<T>()?.get_handle_from_name(name)
    }

    /// Attempt to get a segment from it's handle.
    pub fn get_asset<T: assets::Asset>(&self, handle: &assets::AssetHandle<T>) -> Option<&T> {
        self.get_storage::<T>()?.get_asset_from_handle(handle)
    }

    /// Inner function to get a storage from an asset type
    fn get_storage<T: assets::Asset>(&self) -> Option<&assets::collection::AssetCollection<T>> {
        let key = std::any::TypeId::of::<T>();
        let dyn_collection = self.collections.get(&key)?;

        match (dyn_collection as &dyn std::any::Any).downcast_ref() {
            Some(res) => Some(res),
            None => {
                log::warn!("Invalid asset collection for asset type {}", std::any::type_name::<T>());
                None
            }
        }
    }
}
