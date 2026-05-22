use crate::propellant::assets;

pub struct AssetHandle<T: assets::Asset> {
    _m: std::marker::PhantomData<T>,
    index: usize,
}
