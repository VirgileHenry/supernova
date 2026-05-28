use crate::propellant::assets;

pub struct AssetManager {
    /// Handle to the Vulkan device.
    vk_device: crate::propellant::vulkan::VkDeviceHandle,

    /// Keep track of the asset directory
    asset_directory: String,

    segments: assets::collection::AssetCollection<assets::segment::Segment>,
    segment_buffer: assets::segment::SegmentGpuAllocation,
}

impl AssetManager {
    /// Load all the required game assets from a given directory.
    pub fn load(vk_device: crate::propellant::vulkan::VkDeviceHandle, asset_directory: &str) -> crate::ScResult<Self> {
        log::info!("Loading assets from \"./{asset_directory}\"");

        let segments_directory = format!("{asset_directory}/segments");

        let mut segments = assets::collection::AssetCollection::load(&segments_directory)?;
        let segment_buffer = assets::segment::SegmentGpuAllocation::create(&vk_device, segments.assets_mut())?;
        log::info!(
            "Loaded {} assets in the \"segments\" collection, {} bytes on the GPU",
            segments.count(),
            segment_buffer.buffer_size(),
        );

        Ok(Self {
            vk_device: vk_device.clone(),
            asset_directory: asset_directory.to_string(),
            segments,
            segment_buffer,
        })
    }

    pub fn reload_all(&mut self) -> crate::ScResult<()> {
        log::info!("Reloading assets from \"./{}\"", self.asset_directory);

        let segments_directory = format!("{}/segments", self.asset_directory);
        let mut new_segments = assets::collection::AssetCollection::load(&segments_directory)?;
        let new_segment_buffer = assets::segment::SegmentGpuAllocation::create(&self.vk_device, new_segments.assets_mut())?;
        log::info!(
            "Reloaded {} assets in the \"segments\" collection, {} bytes on the GPU",
            new_segments.count(),
            new_segment_buffer.buffer_size(),
        );

        self.segments = new_segments;
        self.segment_buffer = new_segment_buffer;

        Ok(())
    }

    /// Attempt to get an asset segment by its name.
    pub fn get_segment_handle(&self, segment_name: &str) -> Option<assets::AssetHandle<assets::Segment>> {
        self.segments.get_handle_from_name(segment_name)
    }
}
