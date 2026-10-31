pub struct SegmentRenderer {
    pub handle: crate::propellant::assets::SegmentHandle,
}

impl SegmentRenderer {
    pub fn new(handle: crate::propellant::assets::SegmentHandle) -> Self {
        SegmentRenderer { handle }
    }
}
