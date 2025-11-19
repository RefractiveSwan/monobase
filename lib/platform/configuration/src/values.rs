use std::{env, ffi::OsString, str::FromStr};

use thiserror::Error;

/// Read a boolean env var (accepts true/false/1/0/on/off) returning `None` when unset.
pub fn bool_var(name: &str) -> Result<Option<bool>, EnvValueError> {
    match env::var(name) {
        Ok(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Ok(Some(true));
            }
            let lowered = trimmed.to_ascii_lowercase();
            match lowered.as_str() {
                "true" | "1" | "on" => Ok(Some(true)),
                "false" | "0" | "off" => Ok(Some(false)),
                _ => Err(EnvValueError::InvalidBool {
                    name: name.to_string(),
                    value: raw,
                }),
            }
        }
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(os)) => Err(EnvValueError::InvalidUnicode {
            name: name.to_string(),
            value: os,
        }),
    }
}

/// Read an unsigned 32-bit env var.
pub fn u32_var(name: &str) -> Result<Option<u32>, EnvValueError> {
    parse_numeric_var(name)
}

/// Read an unsigned 64-bit env var.
pub fn u64_var(name: &str) -> Result<Option<u64>, EnvValueError> {
    parse_numeric_var(name)
}

/// Read a port env var (1-65535).
pub fn port_var(name: &str) -> Result<Option<u16>, EnvValueError> {
    match parse_numeric_var::<u16>(name)? {
        Some(0) => Err(EnvValueError::InvalidPort {
            name: name.to_string(),
            value: "0".into(),
        }),
        other => Ok(other),
    }
}

fn parse_numeric_var<T>(name: &str) -> Result<Option<T>, EnvValueError>
where
    T: FromStr,
    T::Err: ToString,
{
    match env::var(name) {
        Ok(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            trimmed
                .parse::<T>()
                .map(Some)
                .map_err(|err| EnvValueError::InvalidNumber {
                    name: name.to_string(),
                    value: raw,
                    details: err.to_string(),
                })
        }
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(os)) => Err(EnvValueError::InvalidUnicode {
            name: name.to_string(),
            value: os,
        }),
    }
}

#[derive(Debug, Error)]
pub enum EnvValueError {
    #[error("environment variable {name} contains invalid boolean '{value}'")]
    InvalidBool { name: String, value: String },
    #[error("environment variable {name} contains invalid integer '{value}': {details}")]
    InvalidNumber {
        name: String,
        value: String,
        details: String,
    },
    #[error("environment variable {name} contains invalid unicode: {value:?}")]
    InvalidUnicode { name: String, value: OsString },
    #[error("environment variable {name} contains invalid port '{value}'")]
    InvalidPort { name: String, value: String },
}
