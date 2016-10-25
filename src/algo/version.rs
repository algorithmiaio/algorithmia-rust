use std::fmt;

/// Version of an algorithm
pub enum Version {
    /// Latest published version
    Latest,
    /// Latest published version with the same minor version, e.g., 1.2 implies 1.2.*
    Minor(u32, u32),
    /// A specific published revision, e.g., 0.1.0
    Revision(u32, u32, u32),
    /// A specific git hash - only works for the algorithm's author
    Hash(String),
}


impl<'a> From<&'a str> for Version {
    fn from(version: &'a str) -> Self {
        match version.split('.').map(|p| p.parse::<u32>()).collect() {
            Ok(parts) => {
                let ver_parts: Vec<u32> = parts;
                match ver_parts.len() {
                    3 => Version::Revision(ver_parts[0], ver_parts[1], ver_parts[2]),
                    2 => Version::Minor(ver_parts[0], ver_parts[1]),
                    _ => Version::Hash(version.into()),
                }
            }
            _ => Version::Hash(version.into()),
        }
    }
}

impl From<(u32, u32)> for Version {
    fn from(version: (u32, u32)) -> Self {
        Version::Minor(version.0, version.1)
    }
}

impl From<(u32, u32, u32)> for Version {
    fn from(version: (u32, u32, u32)) -> Self {
        Version::Revision(version.0, version.1, version.2)
    }
}

/// Displays Version values suitable for printing
impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Version::Latest => write!(f, "latest"),
            Version::Minor(major, minor) => write!(f, "{}.{}", major, minor),
            Version::Revision(major, minor, revision) => {
                write!(f, "{}.{}.{}", major, minor, revision)
            }
            Version::Hash(ref hash) => write!(f, "{}", hash),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latest_string() {
        let version = Version::Latest;
        assert_eq!(version.to_string(), format!("{}", version));
        assert_eq!(&*version.to_string(), "latest");
    }

    #[test]
    fn test_revision_string() {
        let version = Version::Revision(1, 2, 3);
        assert_eq!(version.to_string(), format!("{}", version));
        assert_eq!(&*version.to_string(), "1.2.3");
    }

    #[test]
    fn test_minor_string() {
        let version = Version::Minor(1, 2);
        assert_eq!(version.to_string(), format!("{}", version));
        assert_eq!(&*version.to_string(), "1.2");
    }
}
