/// macro `prompt!` for printing prompt messages to stdout
#[macro_export]
macro_rules! prompt {
    ($($arg:tt)*) => {
        cprint!(
            "<w!>{}</><b!>{}</><w!>{}</><b!>{}</>{}<b!>{}</> ",
            CONFIG.prompt.host,
            CONFIG.prompt.separator,
            CONFIG.prompt.user,
            CONFIG.prompt.path_prefix,
            DISK_ARC
                .clone()
                .lock()
                .unwrap()
                .get_working_directory_full_path(),
            CONFIG.prompt.terminator
        );

        // Flush the buffer to print the prompt before reading the input
        match std::io::stdout().flush() {
            Ok(..) => {}
            Err(err) => {
                warn!("Unable to flush stdout, please try again!");

                log::warn!("Unable to flush stdout, please try again! Error: {}", err);
                continue;
            }
        }
    };
}

/// macro `error!` for printing error messages to stdout
#[macro_export]
macro_rules! error {
    ($fmt:expr $(, $arg:tt)*) => {
        let s = format!($fmt $(, $arg)*);
        cprintln!("<r!>{}</>", s);
    };
}

/// macro `success!` for printing success messages to stdout
#[macro_export]
macro_rules! success {
    ($fmt:expr $(, $arg:tt)*) => {
        let s = format!($fmt $(, $arg)*);
        cprintln!("<g!>{}</>", s);
    };
}

/// macro `warn!` for printing warning messages to stdout
#[macro_export]
macro_rules! warn {
    ($fmt:expr $(, $arg:tt)*) => {
        let s = format!($fmt $(, $arg)*);
        cprintln!("<y!>{}</>", s);
    };
}

/// macro `info!` for printing info messages to stdout
#[macro_export]
macro_rules! info {
    ($fmt:expr $(, $arg:tt)*) => {
        let s = format!($fmt $(, $arg)*);
        cprintln!("<c!>{}</>", s);
    };
}

/// macro `handle!` for invoking the associated regex parser and mediator handler for a given command
#[macro_export]
macro_rules! handle {
    ($mediator:tt, $parser_fn:tt, $input:expr) => {
        match CliParser::$parser_fn($input) {
            Ok(request) => {
                if let Err(err) = $mediator.send(request).unwrap() {
                    error!("Error: {}", err);
                    log::error!("mediator_level: {}", err);
                }
            }
            Err(err) => {
                warn!("Warning: {}", err);
                log::warn!("parser_level: {}", err);
            }
        }
    };

    ($mediator:tt, $parser_fn:tt, $input:expr, $success:expr) => {
        match CliParser::$parser_fn($input) {
            Ok(request) => {
                if let Err(err) = $mediator.send(request).unwrap() {
                    error!("Error: {}", err);
                    log::error!("mediator_level: {}", err);
                } else {
                    success!("{}", $success);
                    log::info!("{}", $success);
                }
            }
            Err(err) => {
                warn!("Warning: {}", err);
                log::warn!("parser_level: {}", err);
            }
        }
    };

    ($mediator:tt, $parser_fn:tt, $input:expr, $success:expr, $call_back_fn:tt $(, $arg:tt)*) => {
        match CliParser::$parser_fn($input) {
            Ok(request) => {
                if let Err(err) = $mediator.send(request).unwrap() {
                    error!("Error: {}", err);
                    log::error!("mediator_level: {}", err);
                } else {
                    success!("{}", $success);
                    log::info!("{}", $success);

                    $call_back_fn($($arg)*);
                }
            }
            Err(err) => {
                warn!("Warning: {}", err);
                log::warn!("parser_level: {}", err);
            }
        }
    };

    ($parser_fn:tt, $input:expr, $call_back_fn:tt $(, $arg:tt)*) => {
        match CliParser::$parser_fn($input) {
            Ok(_) => $call_back_fn($($arg)*),
            Err(err) => {
                warn!("Warning: {}", err);
                log::warn!("parser_level: {}", err);
            }
        }
    };
}
