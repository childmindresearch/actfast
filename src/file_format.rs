//! File format identification
//!
//! This module provides a function to identify the file format of a file based on magic numbers.
//!
//! There are also a lot of CSV and other standard file formats used by various manufacturers.
//! These are *not* supported by this library.
//! This library is designed to read binary files that contain raw sensor data.
//! Use any general purpose CSV reader to read these files.
//! Because CSV files do not necessarily contain a unique header we can not identify them from the file contents.
//!
//! Examples of CSV and other standard file formats that are not supported:
//! - Philips Actiwatch AWD:
//!   Uses the AWD file extension, but is a CSV file with 7 lines of header followed
//!   by [{time}" , "{activity counts}"\n"] in minute intervals.
//!   (Discontinued 2023: <https://www.camntech.com/actiwatch-discontinued/>)
//! - ActiGraph CSV:
//!   Note that ActiGraph also has a binary format (GT3X) that *is supported* by this library.
//! - ActiGraph AGD:
//!   These are sqlite databases.
//! - ActiWatch MTN:
//!   These are XML files.
//! - Axivity WAV:
//!   These are WAV audio files. First three channels are X, Y, Z accelerometer data,
//!   the fourth is temperature.
//! - Misc. XLS, XLSX, ODS, etc.:
//!   These are Microsoft Excel or Open Document Spreadsheets.

use std::fmt;

/// File formats supported by this library
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    ActigraphGt3x,
    AxivityCwa,
    GeneactivBin,
    GeneaBin,
    UnknownWav,
    UnknownSqlite,
}

impl fmt::Display for FileFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileFormat::ActigraphGt3x => write!(f, "Actigraph GT3X"),
            FileFormat::AxivityCwa => write!(f, "Axivity CWA"),
            FileFormat::GeneactivBin => write!(f, "GeneActiv BIN"),
            FileFormat::GeneaBin => write!(f, "Genea BIN"),
            FileFormat::UnknownWav => write!(f, "WAV audio"),
            FileFormat::UnknownSqlite => write!(f, "SQLite database"),
        }
    }
}

/// Identify the file format of a file based on its magic number
pub fn identify(magic: &[u8; 4]) -> Option<FileFormat> {
    match magic {
        b"PK\x03\x04" => Some(FileFormat::ActigraphGt3x),
        b"Devi" => Some(FileFormat::GeneactivBin),
        [b'M', b'D', ..] => Some(FileFormat::AxivityCwa),
        b"GENE" => Some(FileFormat::GeneaBin),
        b"RIFF" => Some(FileFormat::UnknownWav),
        b"SQLi" => Some(FileFormat::UnknownSqlite),
        _ => None,
    }
}
