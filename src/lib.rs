use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

/// This represents a named file
///
/// This can be used with clap's derive API like so
///
/// ```rust
/// # use clap_file::NamedFile;
/// #[derive(clap::Parser)]
/// struct CliArgs {
///     input: NamedFile,
/// }
/// ```
pub struct NamedFile {
    pub file: fs::File,
    pub path: PathBuf,
}

/// A clap parser for parsing [`NamedFiles`](NamedFile)
#[derive(Copy, Clone, Debug)]
pub struct NamedFileParser;

impl clap::builder::ValueParserFactory for NamedFile {
    type Parser = NamedFileParser;

    #[inline]
    fn value_parser() -> Self::Parser {
        NamedFileParser
    }
}

/// a wrapper around [`io::Error`] which knows which file it came from
#[derive(Debug)]
pub struct IoError {
    path: PathBuf,
    err: io::Error,
}

impl From<IoError> for io::Error {
    #[inline]
    fn from(value: IoError) -> Self {
        value.err
    }
}

impl core::fmt::Display for IoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Encountered an error while opening the file at {}: {}",
            self.path.display(),
            self.err
        )
    }
}

impl std::error::Error for IoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.err)
    }
}

impl NamedFile {
    pub fn read(&self) -> Result<Vec<u8>, IoError> {
        let size = self.file().metadata().map_or(0, |metadata| metadata.len());
        let mut output = Vec::with_capacity(size as usize);
        io::BufReader::new(self.file())
            .read_to_end(&mut output)
            .map_err(|err| IoError {
                path: self.path.clone(),
                err,
            })?;
        Ok(output)
    }

    pub fn read_to_string(&self) -> Result<String, IoError> {
        let size = self.file().metadata().map_or(0, |metadata| metadata.len());
        let mut output = String::with_capacity(size as usize);
        io::BufReader::new(self.file())
            .read_to_string(&mut output)
            .map_err(|err| IoError {
                path: self.path.clone(),
                err,
            })?;
        Ok(output)
    }

    pub fn file(&self) -> &fs::File {
        &self.file
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

// This shouldn't be necessary, but `clap` requires it
impl Clone for NamedFile {
    fn clone(&self) -> Self {
        Self {
            file: self.file.try_clone().unwrap(),
            path: self.path.clone(),
        }
    }
}

impl clap::builder::TypedValueParser for NamedFileParser {
    type Value = NamedFile;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        self.parse(cmd, arg, value.into())
    }

    fn parse(
        &self,
        cmd: &clap::Command,
        arg: Option<&clap::Arg>,
        value: std::ffi::OsString,
    ) -> Result<Self::Value, clap::Error> {
        let path = Path::new(&value);
        let file = std::fs::File::open(path).map_err(|err| match arg {
            Some(arg) => clap::Error::raw(
                clap::error::ErrorKind::ValueValidation,
                format_args!(
                    "Could not open file specified at {}: {path:?}\n{err}\n",
                    arg.get_value_names().unwrap()[0],
                ),
            )
            .with_cmd(cmd),
            _ => clap::Error::raw(
                clap::error::ErrorKind::ValueValidation,
                format_args!("Could not open file: {path:?}\n{err}\n"),
            )
            .with_cmd(cmd),
        })?;

        Ok(NamedFile {
            file,
            path: value.into(),
        })
    }
}
