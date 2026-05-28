mod collection;
mod handle;
mod manager;
mod segment;

pub use handle::AssetHandle;
pub use manager::AssetManager;
pub use segment::Segment;

pub trait Asset: Sized {
    fn name(&self) -> &str;
    fn load(data: &str) -> std::io::Result<Self>;
}
