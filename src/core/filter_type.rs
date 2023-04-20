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
