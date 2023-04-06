use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum ContentType {
    Alpha,
    Num,
    Stdin,
    Unknown,
}

impl FromStr for ContentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "alpha" => Ok(ContentType::Alpha),
            "num" => Ok(ContentType::Num),
            "stdin" => Ok(ContentType::Stdin),
            _ => Ok(ContentType::Unknown),
        }
    }
}

impl Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentType::Alpha => write!(f, "alpha"),
            ContentType::Num => write!(f, "num"),
            ContentType::Stdin => write!(f, "stdin"),
            ContentType::Unknown => write!(f, "unknown"),
        }
    }
}
