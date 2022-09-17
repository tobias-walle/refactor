use std::path::PathBuf;

use clap::builder::TypedValueParser;

#[derive(Debug, Clone)]
pub struct Move {
    pub source: PathBuf,
    pub destination: PathBuf,
}

#[derive(Debug, Clone)]
pub struct MoveValueParser {}

impl MoveValueParser {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for MoveValueParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TypedValueParser for MoveValueParser {
    type Value = Move;

    fn parse_ref(
        &self,
        _cmd: &clap::Command,
        _arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let value = match value.to_str() {
            Some(value) => value,
            None => {
                return Err(clap::Error::raw(
                    clap::ErrorKind::InvalidUtf8,
                    "Argument is an invalid UTF-8 string",
                ))
            }
        };
        let (source, destination) = match value.split_once("::") {
            Some(value) => value,
            None => {
                return Err(clap::Error::raw(
                    clap::ErrorKind::Format,
                    r#"Argument needs to have a "::" delimiter"#,
                ))
            }
        };
        Ok(Move {
            source: source
                .parse()
                .map_err(|_| clap::Error::raw(clap::ErrorKind::Format, r#"Invalid source path"#))?,
            destination: destination.parse().map_err(|_| {
                clap::Error::raw(clap::ErrorKind::Format, r#"Invalid destination path"#)
            })?,
        })
    }
}
