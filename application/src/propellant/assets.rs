mod collection;
mod handle;
mod manager;
mod segment;

pub use handle::AssetHandle;
pub use handle::SegmentHandle;
pub use manager::AssetLoadError;
pub use manager::AssetManager;
pub use segment::Segment;

pub trait Asset: Sized {
    const ASSET_KIND: &'static str;
    fn name(&self) -> &str;
    fn load(data: &str) -> std::io::Result<Self>;
}
