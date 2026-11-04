#[derive(Debug, Clone, Copy)]
pub struct Version {
    major: u8,
    minor: u8,
    patch: u8,
}

impl Version {
    pub(crate) fn to_u32(self) -> u32 {
        let major = u32::from(self.major);
        let minor = u32::from(self.minor);
        let patch = u32::from(self.patch);
        major << 16 | minor << 8 | patch
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

pub fn version(major: u8, minor: u8, patch: u8) -> Version {
    Version { major, minor, patch }
}
