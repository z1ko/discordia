
use std::error::Error;

pub type Failable<T> = Result<T, Box<dyn Error>>;