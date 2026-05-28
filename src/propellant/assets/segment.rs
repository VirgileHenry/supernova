type SegmentBufferView = crate::propellant::vulkan::BufferView<crate::csg::CsgNodeRepr>;

/// Segment, as an asset.
///
/// A Segment is a single building part that are assembled
/// together to create constructs.
pub struct Segment {
    name: String,
    shell: crate::csg::CsgTree,
    interior: crate::csg::CsgTree,
    flat_shell: Vec<crate::csg::CsgNodeRepr>,
    flat_interior: Vec<crate::csg::CsgNodeRepr>,
    shell_buffer_view: SegmentBufferView,
    interior_buffer_view: SegmentBufferView,
}

impl Segment {
    pub fn shell_view(&self) -> SegmentBufferView {
        self.shell_buffer_view
    }

    pub fn interior_view(&self) -> SegmentBufferView {
        self.interior_buffer_view
    }
}

impl crate::propellant::assets::Asset for Segment {
    fn name(&self) -> &str {
        &self.name
    }

    fn load(data: &str) -> std::io::Result<Self> {
        let serialized: SerializedSegment = ron::from_str(data).map_err(std::io::Error::other)?;

        let flat_shell = serialized.shell.flatten();
        let flat_interior = serialized.interior.flatten();

        Ok(Segment {
            name: serialized.name,
            shell: serialized.shell,
            interior: serialized.interior,
            flat_shell,
            flat_interior,
            shell_buffer_view: SegmentBufferView::empty(),
            interior_buffer_view: SegmentBufferView::empty(),
        })
    }
}

/// A serialized segment only role is to provide the
/// layout for defining segments in the asset files.
///
/// Segments in asset files are deserialized into this
/// struct, before being properly loaded by the engine.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SerializedSegment {
    name: String,
    shell: crate::csg::CsgTree,
    interior: crate::csg::CsgTree,
}

///
pub struct SegmentGpuAllocation {
    buffer: crate::propellant::vulkan::StorageBuffer<crate::csg::CsgNodeRepr>,
}

impl SegmentGpuAllocation {
    pub fn create(vk_device: &crate::propellant::vulkan::VkDeviceHandle, segments: &mut [Segment]) -> crate::ScResult<Self> {
        let required_buffer_size: usize = segments
            .iter()
            .map(|segment| segment.flat_shell.len() + segment.flat_interior.len())
            .sum();

        /* This WILL crash at some point :) */
        let required_buffer_size = std::num::NonZeroUsize::new(required_buffer_size).unwrap();

        let mut staging_buffer = crate::propellant::vulkan::StagingBuffer::create(vk_device, required_buffer_size)?;

        /* Fill up staging buffer, filling in the segment offset as we go */
        let mut offset = 0;
        for segment in segments.iter_mut() {
            match staging_buffer.write(offset, segment.flat_shell.as_slice()) {
                Ok(shell_view) => {
                    segment.shell_buffer_view = shell_view;
                    offset += shell_view.length();
                }
                Err(e) => log::error!("Failed to write segment data to GPU buffer: {e}"),
            }
            match staging_buffer.write(offset, segment.flat_interior.as_slice()) {
                Ok(interior_view) => {
                    segment.interior_buffer_view = interior_view;
                    offset += interior_view.length();
                }
                Err(e) => log::error!("Failed to write segment data to GPU buffer: {e}"),
            }
        }

        let mut storage_buffer = crate::propellant::vulkan::StorageBuffer::create(vk_device, required_buffer_size)?;

        /* transfer staging to storage buffer */
        let sourcev_view = staging_buffer.entire_buffer();
        let destination_view = storage_buffer.entire_buffer();

        crate::propellant::vulkan::copy_buffer(
            vk_device,
            &staging_buffer,
            &mut storage_buffer,
            sourcev_view,
            destination_view,
        )?;

        Ok(SegmentGpuAllocation { buffer: storage_buffer })
    }

    pub fn buffer_size(&self) -> usize {
        self.buffer.size()
    }
}
