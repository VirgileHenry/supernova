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
    shell_buffer_view: SegmentBufferView,
    interior_buffer_view: SegmentBufferView,
    bounding_box: csg::AxisAlignedBoundingBox,
}

impl From<SerializedSegment> for SegmentAsset {
    fn from(serialized: SerializedSegment) -> Self {
        let flat_shell = serialized.shell.flatten();
        let flat_interior = serialized.interior.flatten();
        let bounding_box = serialized.shell.bounding_box();

        Segment {
            name: serialized.name,
            shell: serialized.shell,
            interior: serialized.interior,
            flat_shell,
            flat_interior,
            shell_buffer_view: SegmentBufferView::empty(),
            interior_buffer_view: SegmentBufferView::empty(),
            bounding_box,
        }
    }
}

impl propellant::assets::Asset for SegmentAsset {
    const ASSET_KIND: &'static str = "Segment";

    type Serialized = SerializedSegment;

    fn name(&self) -> &str {
        ""
    }

    fn load(data: &str) -> std::io::Result<Self> {
        unimplemented!()
    }
}

impl propellant::assets::GraphicAsset for SegmentAsset {
    type GpuData = csg::CsgNodeRepr;
    fn size(&self) -> usize {
        self.flat_shell.len() + self.flat_interior.len()
    }
    fn load(&self, buffer: ()) -> () {}
}
