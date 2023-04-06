use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum FileType {
    Alpha,
    Num,
    Stdin,
    Unknown,
}

impl FromStr for FileType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "alpha" => Ok(FileType::Alpha),
            "num" => Ok(FileType::Num),
            "stdin" => Ok(FileType::Stdin),
            _ => Ok(FileType::Unknown),
        }
    }
}

impl Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::Alpha => write!(f, "alpha"),
            FileType::Num => write!(f, "num"),
            FileType::Stdin => write!(f, "stdin"),
            FileType::Unknown => write!(f, "unknown"),
        }
    }
}
