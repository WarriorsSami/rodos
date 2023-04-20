use crate::CONFIG;
use std::fmt::Display;
use std::str::FromStr;

const ALPHA_CHARS: [char; 26] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

const NUM_CHARS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

const HEX_CHARS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
];

pub(crate) struct ContentGenerator;

impl ContentGenerator {
    fn generate_alpha(size: u32) -> Vec<u8> {
        let mut result = Vec::new();
        result.resize(size as usize, 0);

        let alpha_cycle = ALPHA_CHARS.iter().cycle();
        (0..size)
            .for_each(|i| result[i as usize] = *alpha_cycle.clone().nth(i as usize).unwrap() as u8);

        result
    }

    fn generate_num(size: u32) -> Vec<u8> {
        let mut result = Vec::new();
        result.resize(size as usize, 0);

        let num_cycle = NUM_CHARS.iter().cycle();
        (0..size)
            .for_each(|i| result[i as usize] = *num_cycle.clone().nth(i as usize).unwrap() as u8);

        result
    }

    fn generate_hex(size: u32) -> Vec<u8> {
        let mut result = Vec::new();
        result.resize(size as usize, 0);

        let hex_cycle = HEX_CHARS.iter().cycle();
        (0..size)
            .for_each(|i| result[i as usize] = *hex_cycle.clone().nth(i as usize).unwrap() as u8);

        result
    }

    fn generate_from_file(file_path: &str) -> Vec<u8> {
        let mut result = Vec::new();

        match std::fs::read(file_path) {
            Ok(content) => result = content,
            Err(e) => log::error!("Unable to read file: {}", e),
        }

        result
    }

    pub(crate) fn generate(content_type: ContentType, size: u32) -> Vec<u8> {
        match content_type {
            ContentType::Alpha => Self::generate_alpha(size),
            ContentType::Num => Self::generate_num(size),
            ContentType::Hex => Self::generate_hex(size),
            ContentType::Stdin => Self::generate_from_file(CONFIG.stdin_file_path.as_str()),
            ContentType::Temp => Self::generate_from_file(CONFIG.temp_file_path.as_str()),
            ContentType::Unknown => Vec::default(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum ContentType {
    Alpha,
    Num,
    Hex,
    #[allow(dead_code)]
    Stdin,
    Temp,
    Unknown,
}

impl FromStr for ContentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "alpha" => Ok(ContentType::Alpha),
            "num" => Ok(ContentType::Num),
            "hex" => Ok(ContentType::Hex),
            _ => Ok(ContentType::Unknown),
        }
    }
}

impl Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentType::Alpha => write!(f, "alpha"),
            ContentType::Num => write!(f, "num"),
            ContentType::Hex => write!(f, "hex"),
            ContentType::Stdin => write!(f, "stdin"),
            ContentType::Temp => write!(f, "temp"),
            ContentType::Unknown => write!(f, "unknown"),
        }
    }
}
