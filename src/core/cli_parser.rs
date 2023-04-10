use crate::application::cat::CatRequest;
use crate::application::cp::CopyRequest;
use crate::application::create::CreateRequest;
use crate::application::del::DeleteRequest;
use crate::application::help::HelpRequest;
use crate::application::ls::ListRequest;
use crate::application::neofetch::NeofetchRequest;
use crate::application::rename::RenameRequest;
use crate::application::Void;
use crate::core::content_type::ContentType;
use crate::{info, CONFIG};
use color_print::cprintln;
use log::log;
use std::error::Error;

pub(crate) struct CliParser;

impl CliParser {
    pub(crate) fn parse_help(input: &str) -> Result<HelpRequest, Box<dyn Error>> {
        log::info!("Parsing help command...");

        let regex = regex::Regex::new(CONFIG.commands.get("help").unwrap().regex.as_str()).unwrap();
        let usage = CONFIG.commands.get("help").unwrap().usage.as_str();

        if regex.is_match(input) {
            log::info!("Help command parsed successfully!");
            Ok(HelpRequest::new())
        } else {
            info!("Usage: {}", usage);
            Err(Box::try_from("Invalid help command syntax!").unwrap())
        }
    }

    pub(crate) fn parse_exit(input: &str) -> Void {
        log::info!("Parsing exit command...");

        let regex = regex::Regex::new(CONFIG.commands.get("exit").unwrap().regex.as_str()).unwrap();
        let usage = CONFIG.commands.get("exit").unwrap().usage.as_str();

        if regex.is_match(input) {
            log::info!("Exit command parsed successfully!");
            Ok(())
        } else {
            info!("Usage: {}", usage);
            Err(Box::try_from("Invalid exit command syntax!").unwrap())
        }
    }

    pub(crate) fn parse_neofetch(input: &str) -> Result<NeofetchRequest, Box<dyn Error>> {
        log::info!("Parsing neofetch command...");

        let regex =
            regex::Regex::new(CONFIG.commands.get("neofetch").unwrap().regex.as_str()).unwrap();
        let usage = CONFIG.commands.get("neofetch").unwrap().usage.as_str();

        if regex.is_match(input) {
            log::info!("Neofetch command parsed successfully!");
            Ok(NeofetchRequest::new())
        } else {
            info!("Usage: {}", usage);
            Err(Box::try_from("Invalid neofetch command syntax!").unwrap())
        }
    }

    pub(crate) fn parse_create(input: &str) -> Result<CreateRequest, Box<dyn Error>> {
        log::info!("Parsing create command...");

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

            log::info!("Create command parsed successfully: {}", input);
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
        log::info!("Parsing ls command...");

        let regex = regex::Regex::new(CONFIG.commands.get("ls").unwrap().regex.as_str()).unwrap();
        let usage = CONFIG.commands.get("ls").unwrap().usage.as_str();

        if regex.is_match(input) {
            log::info!("Ls command parsed successfully!");
            Ok(ListRequest::new())
        } else {
            info!("Usage: {}", usage);
            Err(Box::try_from("Invalid ls command syntax!").unwrap())
        }
    }

    pub(crate) fn parse_rename(input: &str) -> Result<RenameRequest, Box<dyn Error>> {
        log::info!("Parsing rename command...");

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

            log::info!("Rename command parsed successfully: {}", input);
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

    pub(crate) fn parse_del(input: &str) -> Result<DeleteRequest, Box<dyn Error>> {
        log::info!("Parsing del command...");

        let regex = regex::Regex::new(CONFIG.commands.get("del").unwrap().regex.as_str()).unwrap();
        let captures = regex.captures(input);
        let usage = CONFIG.commands.get("del").unwrap().usage.as_str();

        if let Some(captures) = captures {
            let name = captures.name("name").unwrap().as_str();
            let extension = captures.name("extension").unwrap().as_str();

            if name.len() > 8 {
                return Err(Box::try_from("Name must be 8 characters or less!").unwrap());
            }

            if extension.len() > 3 {
                return Err(Box::try_from("Extension must be 3 characters or less!").unwrap());
            }

            log::info!("Del command parsed successfully: {}", input);
            Ok(DeleteRequest::new(name.to_string(), extension.to_string()))
        } else {
            info!("Usage: {}", usage);
            Err(Box::try_from("Invalid delete command syntax!").unwrap())
        }
    }

    pub(crate) fn parse_cat(input: &str) -> Result<CatRequest, Box<dyn Error>> {
        log::info!("Parsing cat command...");

        let regex = regex::Regex::new(CONFIG.commands.get("cat").unwrap().regex.as_str()).unwrap();
        let captures = regex.captures(input);
        let usage = CONFIG.commands.get("cat").unwrap().usage.as_str();

        if let Some(captures) = captures {
            let name = captures.name("name").unwrap().as_str();
            let extension = captures.name("extension").unwrap().as_str();

            if name.len() > 8 {
                return Err(Box::try_from("Name must be 8 characters or less!").unwrap());
            }

            if extension.len() > 3 {
                return Err(Box::try_from("Extension must be 3 characters or less!").unwrap());
            }

            log::info!("Cat command parsed successfully: {}", input);
            Ok(CatRequest::new(name.to_string(), extension.to_string()))
        } else {
            info!("Usage: {}", usage);
            Err(Box::try_from("Invalid cat command syntax!").unwrap())
        }
    }

    pub(crate) fn parse_cp(input: &str) -> Result<CopyRequest, Box<dyn Error>> {
        log::info!("Parsing cp command...");

        let regex = regex::Regex::new(CONFIG.commands.get("cp").unwrap().regex.as_str()).unwrap();
        let captures = regex.captures(input);
        let usage = CONFIG.commands.get("cp").unwrap().usage.as_str();

        if let Some(captures) = captures {
            let src_name = captures.name("src_name").unwrap().as_str();
            let src_extension = captures.name("src_extension").unwrap().as_str();
            let dst_name = captures.name("dest_name").unwrap().as_str();
            let dst_extension = captures.name("dest_extension").unwrap().as_str();

            if src_name.len() > 8 {
                return Err(Box::try_from("Source name must be 8 characters or less!").unwrap());
            }

            if src_extension.len() > 3 {
                return Err(
                    Box::try_from("Source extension must be 3 characters or less!").unwrap(),
                );
            }

            if dst_name.len() > 8 {
                return Err(
                    Box::try_from("Destination name must be 8 characters or less!").unwrap(),
                );
            }

            if dst_extension.len() > 3 {
                return Err(
                    Box::try_from("Destination extension must be 3 characters or less!").unwrap(),
                );
            }

            log::info!("Copy command parsed successfully: {}", input);
            Ok(CopyRequest::new(
                src_name.to_string(),
                src_extension.to_string(),
                dst_name.to_string(),
                dst_extension.to_string(),
            ))
        } else {
            info!("Usage: {}", usage);
            Err(Box::try_from("Invalid copy command syntax!").unwrap())
        }
    }
}
