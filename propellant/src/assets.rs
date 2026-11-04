mod collection;
mod handle;
mod manager;

pub use handle::AssetHandle;
pub use manager::AssetLoadError;
pub use manager::AssetManager;

/// Trait for assets.
pub trait Asset: Sized + 'static {
    const ASSET_KIND: &'static str;
    type Serialized: Into<Self>;
    fn name(&self) -> &str;
    fn load(data: &str) -> std::io::Result<Self>;
}

/// Trait for assets that needs to be uploaded to the Gpu.
pub trait GraphicAsset: Asset {
    type GpuData: bytemuck::Pod + bytemuck::Zeroable;
    fn size(&self) -> usize;
    fn load(&self, buffer: &mut crate::vulkan::StagingBuffer<Self::GpuData>, offset: usize) -> crate::vulkan::BufferWriteError;
}
