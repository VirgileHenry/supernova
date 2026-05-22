/// Segment, as an asset.
///
/// A Segment is a single building part that are assembled
/// together to create constructs.
pub struct Segment {
    name: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SerializedSegment {
    name: String,
    shell: crate::csg::CsgTree,
    interior: crate::csg::CsgTree,
    cosmetics: crate::csg::CsgTree,
}

impl crate::propellant::assets::Asset for Segment {
    fn name(&self) -> &str {
        &self.name
    }

    fn load(data: &str) -> std::io::Result<Self> {
        let serialized: SerializedSegment = ron::from_str(data).map_err(std::io::Error::other)?;

        Ok(Segment { name: serialized.name })
    }
}
