mod collection;
mod handle;
mod manager;
mod storage;

pub use handle::AssetHandle;
pub use manager::AssetLoadError;
pub use manager::AssetLoadingContext;
pub use manager::AssetManager;
pub use storage::GpuAssetStorage;

/// Trait for assets.
pub trait Asset: Sized + 'static {
    const ASSET_KIND: &'static str;

    type Serialized: Into<Self> + for<'de> serde::Deserialize<'de>;
    type Storage: storage::AssetStorage<Self>;

    fn name(&self) -> &str;
}

/// Trait for assets that needs to be uploaded to the Gpu.
pub trait GraphicAsset: Asset {
    type GpuData: bytemuck::Pod + bytemuck::Zeroable;
    fn size(&self) -> usize;
    fn load(
        &mut self,
        buffer: &mut crate::vulkan::StagingBuffer<Self::GpuData>,
        offset: usize,
    ) -> Result<usize, crate::vulkan::BufferWriteError>;
}
