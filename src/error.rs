use std::fmt;

/// Location context for where an error occurred in a file
#[derive(Debug, Clone)]
pub struct FileLocation {
    pub byte_offset: Option<u64>,
    pub line_number: Option<usize>,
    pub record_index: Option<usize>,
    pub sample_index: Option<usize>,
}

impl FileLocation {
    pub fn new() -> Self {
        Self {
            byte_offset: None,
            line_number: None,
            record_index: None,
            sample_index: None,
        }
    }

    pub fn at_line(line: usize) -> Self {
        Self {
            line_number: Some(line),
            ..Self::new()
        }
    }

    pub fn at_record(record: usize) -> Self {
        Self {
            record_index: Some(record),
            ..Self::new()
        }
    }

    pub fn with_sample(mut self, sample: usize) -> Self {
        self.sample_index = Some(sample);
        self
    }
}

impl Default for FileLocation {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for FileLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if let Some(line) = self.line_number {
            parts.push(format!("line {}", line));
        }
        if let Some(record) = self.record_index {
            parts.push(format!("record {}", record));
        }
        if let Some(sample) = self.sample_index {
            parts.push(format!("sample {}", sample));
        }
        if let Some(offset) = self.byte_offset {
            parts.push(format!("byte offset {}", offset));
        }
        if parts.is_empty() {
            write!(f, "unknown location")
        } else {
            write!(f, "{}", parts.join(", "))
        }
    }
}

/// Main error type for actfast
#[derive(Debug)]
pub enum ActfastError {
    /// IO error with context
    Io {
        source: std::io::Error,
        context: String,
    },

    /// Could not identify file format from magic bytes
    UnknownFormat { magic: [u8; 4] },

    /// Recognized format but not supported for reading
    UnsupportedFormat {
        format: crate::file_format::FileFormat,
        suggestion: &'static str,
    },

    /// File format identified but parsing failed
    Parse {
        format: crate::file_format::FileFormat,
        message: String,
        location: FileLocation,
    },

    /// Invalid/malformed data in a specific field
    InvalidField {
        field: &'static str,
        value: String,
        expected: &'static str,
        location: FileLocation,
    },

    /// Unexpected end of file
    UnexpectedEof {
        context: String,
        location: FileLocation,
    },

    /// Date/time parsing error
    InvalidDateTime {
        value: String,
        format: &'static str,
        location: FileLocation,
    },

    /// Hex decoding error
    InvalidHex {
        value: String,
        location: FileLocation,
    },

    /// Bitreader error (malformed binary data)
    BitRead {
        context: String,
        location: FileLocation,
    },
}

impl fmt::Display for ActfastError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActfastError::Io { source, context } => {
                write!(f, "IO error {}: {}", context, source)
            }
            ActfastError::UnknownFormat { magic } => {
                write!(
                    f,
                    "Unknown file format (magic bytes: {:02x} {:02x} {:02x} {:02x}). \
                     Supported formats: Actigraph GT3X, GeneActiv BIN, Axivity CWA",
                    magic[0], magic[1], magic[2], magic[3]
                )
            }
            ActfastError::UnsupportedFormat { format, suggestion } => {
                write!(f, "Unsupported file format '{}': {}", format, suggestion)
            }
            ActfastError::Parse {
                format,
                message,
                location,
            } => {
                write!(
                    f,
                    "Failed to parse {} file at {}: {}",
                    format, location, message
                )
            }
            ActfastError::InvalidField {
                field,
                value,
                expected,
                location,
            } => {
                write!(
                    f,
                    "Invalid value for '{}' at {}: got '{}', expected {}",
                    field, location, value, expected
                )
            }
            ActfastError::UnexpectedEof { context, location } => {
                write!(f, "Unexpected end of file {} at {}", context, location)
            }
            ActfastError::InvalidDateTime {
                value,
                format,
                location,
            } => {
                write!(
                    f,
                    "Invalid datetime '{}' at {} (expected format: {})",
                    value, location, format
                )
            }
            ActfastError::InvalidHex { value, location } => {
                let preview: String = value.chars().take(20).collect();
                let suffix = if value.len() > 20 { "..." } else { "" };
                write!(
                    f,
                    "Invalid hex data '{}{}' at {}",
                    preview, suffix, location
                )
            }
            ActfastError::BitRead { context, location } => {
                write!(
                    f,
                    "Failed to read binary data ({}) at {}",
                    context, location
                )
            }
        }
    }
}

impl std::error::Error for ActfastError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ActfastError::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

// Conversion from std::io::Error with default context
impl From<std::io::Error> for ActfastError {
    fn from(err: std::io::Error) -> Self {
        ActfastError::Io {
            source: err,
            context: "reading file".to_string(),
        }
    }
}

// Helper trait for adding IO context
pub trait IoResultExt<T> {
    fn with_context(self, context: impl Into<String>) -> Result<T>;
}

impl<T> IoResultExt<T> for std::result::Result<T, std::io::Error> {
    fn with_context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|source| ActfastError::Io {
            source,
            context: context.into(),
        })
    }
}

// Conversion to PyErr
impl From<ActfastError> for pyo3::PyErr {
    fn from(err: ActfastError) -> pyo3::PyErr {
        pyo3::exceptions::PyValueError::new_err(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ActfastError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_location_display() {
        let loc = FileLocation::at_record(5).with_sample(10);
        let s = format!("{}", loc);
        assert!(s.contains("record 5"));
        assert!(s.contains("sample 10"));
    }

    #[test]
    fn test_error_display() {
        let err = ActfastError::UnknownFormat {
            magic: [0x00, 0x01, 0x02, 0x03],
        };
        let msg = format!("{}", err);
        assert!(msg.contains("00 01 02 03"));
        assert!(msg.contains("Unknown file format"));
    }

    #[test]
    fn test_io_error_context() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err = ActfastError::Io {
            source: io_err,
            context: "opening config".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("opening config"));
        assert!(msg.contains("file missing"));
    }
}
