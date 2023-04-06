use std::sync::{Arc, Mutex};

pub(crate) mod cli_macros;
pub(crate) mod cli_parser;
pub(crate) mod content_type;

pub(crate) type Arm<T> = Arc<Mutex<T>>;
