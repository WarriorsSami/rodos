/// SortType is used to sort the files and folders fpr the output of the ls command:
/// - `NameAsc`: sort by name ascending
/// - `NameDesc`: sort by name descending
/// - `DateAsc`: sort by date ascending
/// - `DateDesc`: sort by date descending
/// - `SizeAsc`: sort by size ascending
/// - `SizeDesc`: sort by size descending
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SortType {
    NameAsc,
    NameDesc,
    DateAsc,
    DateDesc,
    SizeAsc,
    SizeDesc,
}

impl Default for SortType {
    fn default() -> Self {
        Self::NameAsc
    }
}

impl From<&str> for SortType {
    fn from(s: &str) -> Self {
        match s {
            "na" => Self::NameAsc,
            "nd" => Self::NameDesc,
            "ta" => Self::DateAsc,
            "td" => Self::DateDesc,
            "sza" => Self::SizeAsc,
            "szd" => Self::SizeDesc,
            _ => Self::default(),
        }
    }
}
