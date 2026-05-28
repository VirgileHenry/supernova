use crate::propellant::assets;

pub struct AssetHandle<T: assets::Asset> {
    _m: std::marker::PhantomData<T>,
    id: usize,
}

impl<T: assets::Asset> AssetHandle<T> {
    pub fn new(id: usize) -> Self {
        Self {
            _m: std::marker::PhantomData,
            id,
        }
    }
}
