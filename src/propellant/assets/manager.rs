use crate::propellant::assets;

pub struct AssetManager {
    segments: assets::collection::AssetCollection<assets::segment::Segment>,
}

impl AssetManager {
    pub fn load(asset_directory: &str) -> std::io::Result<Self> {
        log::info!("Loading assets from \"./{asset_directory}\"");

        let segments_directory = format!("{asset_directory}/segments");

        let segments = assets::collection::AssetCollection::load(&segments_directory)?;
        log::info!("Loaded {} assets in the \"segments\" collection", segments.count());

        Ok(Self { segments })
    }
}
