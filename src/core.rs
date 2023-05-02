use std::sync::{Arc, Mutex};

pub(crate) mod cli_macros;
pub(crate) mod cli_parser;
pub(crate) mod config;
pub(crate) mod content_type;
pub(crate) mod filter_type;
pub(crate) mod sort_type;

/// A type alias for a `Arc<Mutex<T>>`.
pub(crate) type Arm<T> = Arc<Mutex<T>>;
