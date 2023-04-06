use std::fmt::Display;
use std::str::FromStr;

pub(crate) struct ContentGenerator;

impl ContentGenerator {
    fn generate_alpha(size: u32) -> Vec<u8> {
        let mut result = Vec::new();
        result.resize(size as usize, 0);

        (0..size).for_each(|i| result[i as usize] = rand::random::<u8>() % 26 + 65);

        result
    }

    fn generate_num(size: u32) -> Vec<u8> {
        let mut result = Vec::new();
        result.resize(size as usize, 0);

        (0..size).for_each(|i| result[i as usize] = rand::random::<u8>() % 10 + 48);

        result
    }

    fn generate_stdin() -> Vec<u8> {
        Vec::default()
    }

    pub(crate) fn generate(content_type: ContentType, size: u32) -> Vec<u8> {
        match content_type {
            ContentType::Alpha => Self::generate_alpha(size),
            ContentType::Num => Self::generate_num(size),
            ContentType::Stdin => Self::generate_stdin(),
            ContentType::Unknown => Vec::default(),
        }
    }
}

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
