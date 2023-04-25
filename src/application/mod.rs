use std::error::Error;

pub(crate) mod cat;
pub(crate) mod cd;
pub(crate) mod cp;
pub(crate) mod create;
pub(crate) mod defrag;
pub(crate) mod del;
pub(crate) mod fmt;
pub(crate) mod help;
pub(crate) mod ls;
pub(crate) mod mkdir;
pub(crate) mod neofetch;
pub(crate) mod rename;
pub(crate) mod setattr;

pub(crate) type Void = Result<(), Box<dyn Error>>;
