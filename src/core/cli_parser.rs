use crate::application::commands::cd::ChangeDirectoryRequest;
use crate::application::commands::cp::CopyRequest;
use crate::application::commands::create::CreateRequest;
use crate::application::commands::defrag::DefragmentRequest;
use crate::application::commands::del::DeleteRequest;
use crate::application::commands::fmt::FormatRequest;
use crate::application::commands::mkdir::MakeDirectoryRequest;
use crate::application::commands::rename::RenameRequest;
use crate::application::commands::setattr::SetAttributesRequest;
use crate::application::queries::cat::CatRequest;
use crate::application::queries::help::HelpRequest;
use crate::application::queries::ls::ListRequest;
use crate::application::queries::neofetch::NeofetchRequest;
use crate::application::queries::pwd::PwdRequest;
use crate::application::Void;
use crate::core::content_type::ContentType;
use crate::core::filter_type::FilterType;
use crate::core::sort_type::SortType;
use crate::domain::file_entry::FileEntryAttributes;
use crate::{info, CONFIG};
use chrono::Utc;
use color_print::cprintln;
use std::error::Error;

/// CliParser is used to parse the input from the user based on the regex patterns defined in the
/// config file.
pub(crate) struct CliParser;

impl CliParser {
    pub(crate) fn parse_help(input: &str) -> Result<HelpRequest, Box<dyn Error>> {
        log::info!("Parsing help command...");

        let regex = regex::Regex::new(CONFIG.commands.get("help").unwrap().regex.as_str()).unwrap();
        let captures = regex.captures(input).unwrap();
        let usage = CONFIG.commands.get("help").unwrap().usage.as_str();

        if regex.is_match(input) {
            log::info!("Help command parsed successfully!");

            let command = captures.name("command");
            let command = command.map(|command| command.as_str().to_string());

            Ok(HelpRequest::new(command))
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
                FileEntryAttributes::combine(&[
                    FileEntryAttributes::File,
                    FileEntryAttributes::ReadWrite,
                    FileEntryAttributes::Visible,
                ]),
                Utc::now(),
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
        let captures = regex.captures(input);
        let usage = CONFIG.commands.get("ls").unwrap().usage.as_str();

        if let Some(captures) = captures {
            let filter_basic = captures.name("filter_basic");
            let filter_name = captures.name("filter_name");
            let filter_extension = captures.name("filter_extension");
            let sort = captures.name("sort");

            let mut filters: Vec<FilterType> = Vec::new();

            // parse the basic filters if any (all from FilterType excepting Name and Extension)
            // from string and extract a vector of those
            if let Some(filter_basic) = filter_basic {
                let filter_basic = filter_basic.as_str();
                filters.append(
                    filter_basic
                        .chars()
                        .map(FilterType::from)
                        .collect::<Vec<FilterType>>()
                        .as_mut(),
                );
            }

            // parse the file name filter and add it to the vector if present
            if let Some(filter_name) = filter_name {
                let filter_name = filter_name.as_str();
                filters.push(FilterType::Name(filter_name.to_string()));
            }

            // parse the file extension filter and add it to the vector if present
            if let Some(filter_extension) = filter_extension {
                let filter_extension = filter_extension.as_str();
                filters.push(FilterType::Extension(filter_extension.to_string()));
            }

            let mut sort_option: Option<SortType> = None;

            // parse the sort option if present
            if let Some(sort) = sort {
                let sort = sort.as_str();
                sort_option = Some(SortType::from(sort));
            }

            log::info!("Ls command parsed successfully!");
            Ok(ListRequest::new(filters, sort_option))
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
            let old_extension = captures.name("old_extension");
            let new_name = captures.name("new_name").unwrap().as_str();
            let new_extension = captures.name("new_extension");

            match (old_extension, new_extension) {
                // when both extensions are present, rename a file
                (Some(old_extension), Some(new_extension)) => {
                    let old_extension = old_extension.as_str();
                    let new_extension = new_extension.as_str();

                    if old_name.len() > 8 {
                        return Err(
                            Box::try_from("Old name must be 8 characters or less!").unwrap()
                        );
                    }

                    if old_extension.len() > 3 {
                        return Err(
                            Box::try_from("Old extension must be 3 characters or less!").unwrap()
                        );
                    }

                    if new_name.len() > 8 {
                        return Err(
                            Box::try_from("New name must be 8 characters or less!").unwrap()
                        );
                    }

                    if new_extension.len() > 3 {
                        return Err(
                            Box::try_from("New extension must be 3 characters or less!").unwrap()
                        );
                    }

                    log::info!("Rename command parsed successfully: {}", input);
                    Ok(RenameRequest::new(
                        old_name.to_string(),
                        old_extension.to_string(),
                        new_name.to_string(),
                        new_extension.to_string(),
                    ))
                }
                // when none of the extensions are present, rename a folder
                (None, None) => {
                    if old_name.len() > 8 {
                        return Err(
                            Box::try_from("Old name must be 8 characters or less!").unwrap()
                        );
                    }

                    if new_name.len() > 8 {
                        return Err(
                            Box::try_from("New name must be 8 characters or less!").unwrap()
                        );
                    }

                    log::info!("Rename command parsed successfully: {}", input);
                    Ok(RenameRequest::new(
                        old_name.to_string(),
                        "".to_string(),
                        new_name.to_string(),
                        "".to_string(),
                    ))
                }
                // else, return an error
                _ => Err(Box::try_from("Cannot rename a folder as a file or vice versa!").unwrap()),
            }
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
            let extension = match captures.name("extension") {
                Some(extension) => extension.as_str(),
                None => "",
            };

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
            let src_extension = captures.name("src_extension");
            let dest_name = captures.name("dest_name").unwrap().as_str();
            let dest_extension = captures.name("dest_extension");

            match (src_extension, dest_extension) {
                // when both extensions are present, copy a file
                (Some(src_extension), Some(dest_extension)) => {
                    let src_extension = src_extension.as_str();
                    let dest_extension = dest_extension.as_str();

                    if src_name.len() > 8 {
                        return Err(
                            Box::try_from("Source name must be 8 characters or less!").unwrap()
                        );
                    }

                    if src_extension.len() > 3 {
                        return Err(Box::try_from(
                            "Source extension must be 3 characters or less!",
                        )
                        .unwrap());
                    }

                    if dest_name.len() > 8 {
                        return Err(Box::try_from(
                            "Destination name must be 8 characters or less!",
                        )
                        .unwrap());
                    }

                    if dest_extension.len() > 3 {
                        return Err(Box::try_from(
                            "Destination extension must be 3 characters or less!",
                        )
                        .unwrap());
                    }

                    log::info!("Copy command parsed successfully: {}", input);
                    Ok(CopyRequest::new(
                        src_name.to_string(),
                        src_extension.to_string(),
                        dest_name.to_string(),
                        dest_extension.to_string(),
                    ))
                }
                // when both extensions are missing, copy a directory
                (None, None) => {
                    if src_name.len() > 8 {
                        return Err(
                            Box::try_from("Source name must be 8 characters or less!").unwrap()
                        );
                    }

                    if dest_name.len() > 8 {
                        return Err(Box::try_from(
                            "Destination name must be 8 characters or less!",
                        )
                        .unwrap());
                    }

                    log::info!("Copy command parsed successfully: {}", input);
                    Ok(CopyRequest::new(
                        src_name.to_string(),
                        "".to_string(),
                        dest_name.to_string(),
                        "".to_string(),
                    ))
                }
                // otherwise, the command is invalid
                _ => {
                    info!("Usage: {}", usage);
                    Err(
                        Box::try_from("Cannot copy a file with a directory name or vice versa!")
                            .unwrap(),
                    )
                }
            }
        } else {
            info!("Usage: {}", usage);
            Err(Box::try_from("Invalid copy command syntax!").unwrap())
        }
    }

    pub(crate) fn parse_fmt(input: &str) -> Result<FormatRequest, Box<dyn Error>> {
        log::info!("Parsing fmt command...");

        let regex = regex::Regex::new(CONFIG.commands.get("fmt").unwrap().regex.as_str()).unwrap();
        let captures = regex.captures(input);
        let usage = CONFIG.commands.get("fmt").unwrap().usage.as_str();

        if let Some(captures) = captures {
            let fat_type = captures.name("fat_type").unwrap().as_str();
            let fat_type = fat_type.parse::<u16>()?;

            log::info!("Format command parsed successfully: {}", input);
            Ok(FormatRequest::new(fat_type))
        } else {
            info!("Usage: {}", usage);
            Err(Box::try_from("Invalid format command syntax!").unwrap())
        }
    }

    pub(crate) fn parse_defrag(input: &str) -> Result<DefragmentRequest, Box<dyn Error>> {
        log::info!("Parsing defrag command...");

        let regex =
            regex::Regex::new(CONFIG.commands.get("defrag").unwrap().regex.as_str()).unwrap();
        let usage = CONFIG.commands.get("defrag").unwrap().usage.as_str();

        if regex.is_match(input) {
            log::info!("Defrag command parsed successfully: {}", input);
            Ok(DefragmentRequest::new())
        } else {
            info!("Usage: {}", usage);
            Err(Box::try_from("Invalid defrag command syntax!").unwrap())
        }
    }

    pub(crate) fn parse_setattr(input: &str) -> Result<SetAttributesRequest, Box<dyn Error>> {
        log::info!("Parsing setattr command...");

        let regex =
            regex::Regex::new(CONFIG.commands.get("setattr").unwrap().regex.as_str()).unwrap();
        let captures = regex.captures(input);
        let usage = CONFIG.commands.get("setattr").unwrap().usage.as_str();

        if let Some(captures) = captures {
            let name = captures.name("name").unwrap().as_str();
            let extension = captures.name("extension");
            let attributes = captures.name("attributes").unwrap().as_str();

            let extension = match extension {
                Some(extension) => extension.as_str(),
                None => "",
            };

            // split attributes into chunks of 2 characters
            // then parse each chunk into a FileEntryAttributes
            // and collect them into a vector
            let attributes = attributes
                .chars()
                .collect::<Vec<_>>()
                .chunks(2)
                .map(|chunk| chunk.iter().collect::<String>())
                .collect::<Vec<_>>()
                .iter()
                .map(|attr_str| attr_str.parse::<FileEntryAttributes>().unwrap())
                .collect::<Vec<_>>();

            Ok(SetAttributesRequest::new(
                name.to_string(),
                extension.to_string(),
                attributes,
            ))
        } else {
            info!("Usage: {}", usage);
            Err(Box::try_from("Invalid setattr command syntax!").unwrap())
        }
    }

    pub(crate) fn parse_mkdir(input: &str) -> Result<MakeDirectoryRequest, Box<dyn Error>> {
        let regex =
            regex::Regex::new(CONFIG.commands.get("mkdir").unwrap().regex.as_str()).unwrap();
        let captures = regex.captures(input);
        let usage = CONFIG.commands.get("mkdir").unwrap().usage.as_str();

        if let Some(captures) = captures {
            let name = captures.name("name").unwrap().as_str();

            if name.len() > 8 {
                return Err(Box::try_from("Name must be 8 characters or less!").unwrap());
            }

            log::info!("Mkdir command parsed successfully: {}", input);
            Ok(MakeDirectoryRequest::new(
                name.to_string(),
                FileEntryAttributes::combine(&[
                    FileEntryAttributes::Directory,
                    FileEntryAttributes::ReadWrite,
                    FileEntryAttributes::Visible,
                ]),
                Utc::now(),
            ))
        } else {
            info!("Usage: {}", usage);
            Err(Box::try_from("Invalid mkdir command syntax!").unwrap())
        }
    }

    pub(crate) fn parse_cd(input: &str) -> Result<ChangeDirectoryRequest, Box<dyn Error>> {
        let regex = regex::Regex::new(CONFIG.commands.get("cd").unwrap().regex.as_str()).unwrap();
        let captures = regex.captures(input);
        let usage = CONFIG.commands.get("cd").unwrap().usage.as_str();

        if let Some(captures) = captures {
            let name = captures.name("name").unwrap().as_str();

            if name.len() > 8 {
                return Err(Box::try_from("Name must be 8 characters or less!").unwrap());
            }

            log::info!("Cd command parsed successfully: {}", input);
            Ok(ChangeDirectoryRequest::new(name.to_string()))
        } else {
            info!("Usage: {}", usage);
            Err(Box::try_from("Invalid cd command syntax!").unwrap())
        }
    }

    pub(crate) fn parse_pwd(input: &str) -> Result<PwdRequest, Box<dyn Error>> {
        let regex = regex::Regex::new(CONFIG.commands.get("pwd").unwrap().regex.as_str()).unwrap();
        let usage = CONFIG.commands.get("pwd").unwrap().usage.as_str();

        if regex.is_match(input) {
            log::info!("Pwd command parsed successfully: {}", input);
            Ok(PwdRequest::new())
        } else {
            info!("Usage: {}", usage);
            Err(Box::try_from("Invalid pwd command syntax!").unwrap())
        }
    }

    pub(crate) fn parse_rmdir(input: &str) -> Result<DeleteRequest, Box<dyn Error>> {
        let regex =
            regex::Regex::new(CONFIG.commands.get("rmdir").unwrap().regex.as_str()).unwrap();
        let captures = regex.captures(input);
        let usage = CONFIG.commands.get("rmdir").unwrap().usage.as_str();

        if let Some(captures) = captures {
            let name = captures.name("name").unwrap().as_str();

            if name.len() > 8 {
                return Err(Box::try_from("Name must be 8 characters or less!").unwrap());
            }

            log::info!("Rmdir command parsed successfully: {}", input);
            Ok(DeleteRequest::new(name.to_string(), "".to_string()))
        } else {
            info!("Usage: {}", usage);
            Err(Box::try_from("Invalid rmdir command syntax!").unwrap())
        }
    }
}
