// macro prompt! for printing prompt messages to stdout
#[macro_export]
macro_rules! prompt {
    ($($arg:tt)*) => {
        cprint!(
            "<w!>{}</><b!>{}</><w!>{}</><b!>{}</>/<b!>{}</> ",
            CONFIG.prompt.host,
            CONFIG.prompt.separator,
            CONFIG.prompt.user,
            CONFIG.prompt.path_prefix,
            CONFIG.prompt.terminator
        );
        // Flush the buffer to print the prompt before reading the input
        std::io::stdout().flush().expect("Unable to flush stdout");
    };
}

// macro error! for printing error messages to stdout
#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {
        cprintln!("<r!>Error: {}</>", $($arg)+);
    };
}

// macro success! for printing success messages to stdout
#[macro_export]
macro_rules! success {
    ($($arg:tt)+) => {
        cprintln!("<g!>Success: {}</>", $($arg)+);
    };
}

// macro warn! for printing warning messages to stdout
#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => {
        cprintln!("<y!>Warning: {}</>", $($arg)+);
    };
}

// macro info! for printing info messages to stdout
#[macro_export]
macro_rules! info {
    ($fmt:expr $(, $arg:tt)*) => {
        let s = format!($fmt $(, $arg)*);
        cprintln!("<c>{}</>", s);
    };
}
