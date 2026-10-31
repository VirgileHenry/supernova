use crate::propellant::assets;

#[derive(PartialEq, Eq)]
pub struct AssetHandle<T: assets::Asset> {
    pub(super) id: usize,
    _m: std::marker::PhantomData<T>,
}

impl<T: assets::Asset> AssetHandle<T> {
    pub fn new(id: usize) -> Self {
        Self {
            _m: std::marker::PhantomData,
            id,
        }
    }
}

impl<T: assets::Asset> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            _m: std::marker::PhantomData,
        }
    }
}

impl<T: assets::Asset> Copy for AssetHandle<T> {}

impl<T: assets::Asset> std::fmt::Debug for AssetHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", T::ASSET_KIND, self.id)
    }
}

pub type SegmentHandle = AssetHandle<assets::Segment>;
