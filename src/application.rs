use std::error::Error;

pub(crate) mod commands;
pub(crate) mod queries;

/// A type alias for a `Result` with no success value.
pub(crate) type Void = Result<(), Box<dyn Error>>;
