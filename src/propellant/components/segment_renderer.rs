type SegmentAssetHandle = crate::propellant::AssetHandle<crate::propellant::assets::Segment>;

pub struct SegmentRenderer {
    segment: SegmentAssetHandle,
}

impl SegmentRenderer {
    pub fn new(segment: SegmentAssetHandle) -> Self {
        SegmentRenderer { segment }
    }
}
