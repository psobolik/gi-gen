/*
 * Copyright (c) 2024 Paul Sobolik
 * Created 2024-04-10
 */

#[derive(Debug)]
pub(crate) enum Error {
    Reqwest(reqwest::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let (source, message) = match self {
            Error::Reqwest(error) => ("reqwest", error.to_string()),
        };
        write!(f, "[{}] {}", source, message)
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Error {
        Error::Reqwest(error)
    }
}

impl std::error::Error for Error {}
