use crate::application::create::CreateRequest;
use crate::core::content_type::ContentType;
use crate::{info, CONFIG};
use color_print::cprintln;
use std::error::Error;

pub(crate) struct CliParser;

impl CliParser {
    pub(crate) fn parse_create(input: &str) -> Result<CreateRequest, Box<dyn Error>> {
        let regex =
            regex::Regex::new(CONFIG.commands.get("create").unwrap().regex.as_str()).unwrap();
        let captures = regex.captures(input);
        let usage = CONFIG.commands.get("create").unwrap().usage.as_str();

        if let Some(captures) = captures {
            let name = captures.name("name").unwrap().as_str();
            let extension = captures.name("extension").unwrap().as_str();
            let dim = captures
                .name("dim")
                .unwrap()
                .as_str()
                .parse::<u32>()
                .unwrap();
            let content_type = captures
                .name("type")
                .unwrap()
                .as_str()
                .parse::<ContentType>()
                .unwrap();

            if content_type == ContentType::Unknown {
                info!("Usage: {}", usage);
                return Err(Box::try_from("Invalid content type!").unwrap());
            }

            Ok(CreateRequest::new(
                name.to_string(),
                extension.to_string(),
                dim,
                content_type,
            ))
        } else {
            info!("Usage: {}", usage);
            Err(Box::try_from("Invalid create command syntax!").unwrap())
        }
    }
}
