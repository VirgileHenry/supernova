/// Generic storage over an asset.
///
/// This trait allows different assets kinds to be stored in different ways.
/// The main use case is to upload assets to the GPU.
pub trait AssetStorage<Asset> {
    type LoadError: std::error::Error;
    fn load(context: &crate::assets::AssetLoadingContext, assets: &mut [Asset]) -> Result<Self, Self::LoadError>
    where
        Self: Sized;
}

pub struct GpuAssetStorage<T: bytemuck::Pod> {
    buffer: crate::vulkan::StorageBuffer<T>,
}

impl<T: bytemuck::Pod> GpuAssetStorage<T> {
    pub fn buffer_info(&self) -> ash::vk::DescriptorBufferInfo {
        self.buffer.descriptor_info()
    }
}

impl<Asset, T> AssetStorage<Asset> for GpuAssetStorage<T>
where
    Asset: crate::assets::GraphicAsset<GpuData = T>,
    T: bytemuck::Pod,
{
    type LoadError = ash::vk::Result;
    fn load(context: &crate::assets::AssetLoadingContext, assets: &mut [Asset]) -> Result<Self, Self::LoadError> {
        let gpu_buffer_size = assets.iter().map(|asset| asset.size()).sum();

        let gpu_buffer_size = match std::num::NonZeroUsize::new(gpu_buffer_size) {
            Some(size) => size,
            None => std::num::NonZeroUsize::new(1).unwrap(), /* Makes the logic simpler, allocate one element */
        };

        let mut staging_buffer = crate::vulkan::StagingBuffer::create(context.vk_device, gpu_buffer_size)?;

        let mut offset = 0;
        for asset in assets.iter_mut() {
            if let Err(e) = asset.load(&mut staging_buffer, offset) {
                log::error!("Failed to load asset to GPU: {e}");
            }
            offset += asset.size();
        }

        let mut storage_buffer = crate::vulkan::StorageBuffer::create(context.vk_device, gpu_buffer_size)?;

        /* transfer staging to storage buffer */
        let destination_view = storage_buffer.entire_buffer();
        crate::vulkan::copy_buffer(
            context.vk_device,
            &staging_buffer,
            &mut storage_buffer,
            staging_buffer.entire_buffer(),
            destination_view,
        )?;
        let bytes_size = crate::utils::format_byte_size(gpu_buffer_size.get());
        log::info!("\"{}\": allocated {} on the GPU", Asset::ASSET_KIND, bytes_size);

        Ok(Self { buffer: storage_buffer })
    }
}
