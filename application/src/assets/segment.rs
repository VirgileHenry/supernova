/// A segment is a single spaceship part.
/// The segment asset is loaded from the asset files.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SerializedSegment {
    name: String,
    shell: csg::CsgTree,
    interior: csg::CsgTree,
}

/// Actual segment asset, ready to use in game.
pub struct SegmentAsset {
    name: String,
    shell: csg::CsgTree,
    interior: csg::CsgTree,
    flat_shell: Vec<csg::CsgNodeRepr>,
    flat_interior: Vec<csg::CsgNodeRepr>,
    shell_buffer_view: propellant::vulkan::BufferView<csg::CsgNodeRepr>,
    interior_buffer_view: propellant::vulkan::BufferView<csg::CsgNodeRepr>,
    bounding_box: csg::AxisAlignedBoundingBox,
}

impl SegmentAsset {
    pub fn bounding_box(&self) -> &csg::AxisAlignedBoundingBox {
        &self.bounding_box
    }
    pub fn shell_view(&self) -> propellant::vulkan::BufferView<csg::CsgNodeRepr> {
        self.shell_buffer_view
    }
    pub fn interior_view(&self) -> propellant::vulkan::BufferView<csg::CsgNodeRepr> {
        self.interior_buffer_view
    }
}

impl From<SerializedSegment> for SegmentAsset {
    fn from(serialized: SerializedSegment) -> Self {
        let flat_shell = serialized.shell.flatten();
        let flat_interior = serialized.interior.flatten();
        let bounding_box = serialized.shell.bounding_box();

        Self {
            name: serialized.name,
            shell: serialized.shell,
            interior: serialized.interior,
            flat_shell,
            flat_interior,
            shell_buffer_view: propellant::vulkan::BufferView::empty(),
            interior_buffer_view: propellant::vulkan::BufferView::empty(),
            bounding_box,
        }
    }
}

impl propellant::assets::Asset for SegmentAsset {
    const ASSET_KIND: &'static str = "Segment";

    type Serialized = SerializedSegment;
    type Storage = propellant::assets::GpuAssetStorage<csg::CsgNodeRepr>;

    fn name(&self) -> &str {
        &self.name
    }
}

impl propellant::assets::GraphicAsset for SegmentAsset {
    type GpuData = csg::CsgNodeRepr;

    fn size(&self) -> usize {
        self.flat_shell.len() + self.flat_interior.len()
    }

    fn load(
        &mut self,
        buffer: &mut propellant::vulkan::StagingBuffer<Self::GpuData>,
        offset: usize,
    ) -> Result<usize, propellant::vulkan::BufferWriteError> {
        let mut written = 0;

        let shell_view = buffer.write(offset + written, self.flat_shell.as_slice())?;
        self.shell_buffer_view = shell_view;
        written += shell_view.length();

        let interior_view = buffer.write(offset + written, self.flat_interior.as_slice())?;
        self.interior_buffer_view = interior_view;
        written += interior_view.length();

        Ok(written)
    }
}
