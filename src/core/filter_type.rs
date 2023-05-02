/// FilterType is used to filter the output of the ls command:
/// - `Name`: filter by name
/// - `Extension`: filter by extension
/// - `Files`: show only files
/// - `Directories`: show only directories
/// - `InShortFormat`: show in short format (just the name and extension)
/// - `InLongFormat`: show in long format (name, extension, size, date, etc.)
/// - `AllAndHidden`: show all files and hidden files
/// - `All`: show all files
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FilterType {
    Name(String),
    Extension(String),
    Files,
    Directories,
    InShortFormat,
    InLongFormat,
    AllAndHidden,
    All,
}

impl Default for FilterType {
    fn default() -> Self {
        Self::All
    }
}

impl From<char> for FilterType {
    fn from(c: char) -> Self {
        match c {
            'a' => Self::All,
            'h' => Self::AllAndHidden,
            's' => Self::InShortFormat,
            'l' => Self::InLongFormat,
            'f' => Self::Files,
            'd' => Self::Directories,
            _ => Self::default(),
        }
    }
}
