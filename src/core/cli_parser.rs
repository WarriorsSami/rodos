use crate::application::create::CreateRequest;
use crate::application::help::HelpRequest;
use crate::application::ls::ListRequest;
use crate::application::neofetch::NeofetchRequest;
use crate::application::rename::RenameRequest;
use crate::application::Void;
use crate::core::content_type::ContentType;
use crate::{info, CONFIG};
use color_print::cprintln;
use std::error::Error;

pub(crate) struct CliParser;

impl CliParser {
    pub(crate) fn parse_help(input: &str) -> Result<HelpRequest, Box<dyn Error>> {
        let regex = regex::Regex::new(CONFIG.commands.get("help").unwrap().regex.as_str()).unwrap();
        let usage = CONFIG.commands.get("help").unwrap().usage.as_str();

        if regex.is_match(input) {
            Ok(HelpRequest::new())
        } else {
            info!("Usage: {}", usage);
            Err(Box::try_from("Invalid help command syntax!").unwrap())
        }
    }

    pub(crate) fn parse_exit(input: &str) -> Void {
        let regex = regex::Regex::new(CONFIG.commands.get("exit").unwrap().regex.as_str()).unwrap();
        let usage = CONFIG.commands.get("exit").unwrap().usage.as_str();

        if regex.is_match(input) {
            Ok(())
        } else {
            info!("Usage: {}", usage);
            Err(Box::try_from("Invalid exit command syntax!").unwrap())
        }
    }

    pub(crate) fn parse_neofetch(input: &str) -> Result<NeofetchRequest, Box<dyn Error>> {
        let regex =
            regex::Regex::new(CONFIG.commands.get("neofetch").unwrap().regex.as_str()).unwrap();
        let usage = CONFIG.commands.get("neofetch").unwrap().usage.as_str();

        if regex.is_match(input) {
            Ok(NeofetchRequest::new())
        } else {
            info!("Usage: {}", usage);
            Err(Box::try_from("Invalid neofetch command syntax!").unwrap())
        }
    }

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

            if name.len() > 8 {
                return Err(Box::try_from("Name must be 8 characters or less!").unwrap());
            }

            if extension.len() > 3 {
                return Err(Box::try_from("Extension must be 3 characters or less!").unwrap());
            }

            if dim > 10000 {
                return Err(Box::try_from("Dimension must be 10000 or less!").unwrap());
            }

            if content_type == ContentType::Unknown {
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

    pub(crate) fn parse_ls(input: &str) -> Result<ListRequest, Box<dyn Error>> {
        let regex = regex::Regex::new(CONFIG.commands.get("ls").unwrap().regex.as_str()).unwrap();
        let usage = CONFIG.commands.get("ls").unwrap().usage.as_str();

        if regex.is_match(input) {
            Ok(ListRequest::new())
        } else {
            info!("Usage: {}", usage);
            Err(Box::try_from("Invalid ls command syntax!").unwrap())
        }
    }

    pub(crate) fn parse_rename(input: &str) -> Result<RenameRequest, Box<dyn Error>> {
        let regex =
            regex::Regex::new(CONFIG.commands.get("rename").unwrap().regex.as_str()).unwrap();
        let captures = regex.captures(input);
        let usage = CONFIG.commands.get("rename").unwrap().usage.as_str();

        if let Some(captures) = captures {
            let old_name = captures.name("old_name").unwrap().as_str();
            let old_extension = captures.name("old_extension").unwrap().as_str();
            let new_name = captures.name("new_name").unwrap().as_str();
            let new_extension = captures.name("new_extension").unwrap().as_str();

            if old_name.len() > 8 {
                return Err(Box::try_from("Old name must be 8 characters or less!").unwrap());
            }

            if old_extension.len() > 3 {
                return Err(Box::try_from("Old extension must be 3 characters or less!").unwrap());
            }

            if new_name.len() > 8 {
                return Err(Box::try_from("New name must be 8 characters or less!").unwrap());
            }

            if new_extension.len() > 3 {
                return Err(Box::try_from("New extension must be 3 characters or less!").unwrap());
            }

            Ok(RenameRequest::new(
                old_name.to_string(),
                old_extension.to_string(),
                new_name.to_string(),
                new_extension.to_string(),
            ))
        } else {
            info!("Usage: {}", usage);
            Err(Box::try_from("Invalid rename command syntax!").unwrap())
        }
    }
}
