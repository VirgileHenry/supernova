pub struct SegmentRenderer {
    pub handle: crate::assets::SegmentHandle,
}

impl SegmentRenderer {
    pub fn new(handle: crate::assets::SegmentHandle) -> Self {
        SegmentRenderer { handle }
    }
}
