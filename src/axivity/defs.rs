#![allow(dead_code)]

use crate::error::{ActfastError, FileLocation, Result};

pub const HEADER_SIZE: usize = 1024;
pub const SECTOR_SIZE: usize = 512;
pub const HEADER_MAGIC: &[u8; 2] = b"MD";
pub const DATA_MAGIC: &[u8; 2] = b"AX";

// Hardware type bytes from header offset 4
pub const HW_AX3_DEFAULT: u8 = 0x00;
pub const HW_AX3_ALT: u8 = 0xFF;
pub const HW_AX6: u8 = 0x64;

/// Decode a packed CWA timestamp (uint32, device local time) into a `chrono::DateTime<Utc>`.
///
/// Layout:
/// - bits  0-5  : seconds (0-59)
/// - bits  6-11 : minutes (0-59)
/// - bits 12-16 : hours   (0-23)
/// - bits 17-21 : day     (1-31)
/// - bits 22-25 : month   (1-12)
/// - bits 26-31 : year offset from 2000
///
/// The CWA spec stores wall-clock time of the configured timezone, but we
/// treat it as UTC here for consistency with the rest of the library. Users
/// who need a different timezone can apply an offset using the `time_zone`
/// metadata field.
pub fn decode_timestamp(
    packed: u32,
    location: FileLocation,
) -> Result<chrono::DateTime<chrono::Utc>> {
    let year = ((packed >> 26) & 0x3F) as i32 + 2000;
    let month = (packed >> 22) & 0x0F;
    let day = (packed >> 17) & 0x1F;
    let hour = (packed >> 12) & 0x1F;
    let minute = (packed >> 6) & 0x3F;
    let second = packed & 0x3F;

    use chrono::TimeZone;
    chrono::Utc
        .with_ymd_and_hms(year, month, day, hour, minute, second)
        .single()
        .ok_or_else(|| ActfastError::InvalidDateTime {
            value: format!(
                "packed=0x{:08x} ({:04}-{:02}-{:02} {:02}:{:02}:{:02})",
                packed, year, month, day, hour, minute, second
            ),
            format: "CWA packed timestamp",
            location,
        })
}

/// Decode the sample-rate / dynamic-range byte (sector offset 24, or header offset 36).
///
/// Returns `(sample_rate_hz, range_g)`. Sample rate is an integer for all CWA
/// rates >= 25 Hz; the 12.5 Hz / 6.25 Hz codes round to 13 / 6 — callers that
/// need sub-hertz precision should use `nanos_per_sample` instead.
pub fn decode_sample_rate(byte: u8) -> (u32, u16) {
    let rate_bits = (byte & 0x0F) as u32;
    let range_bits = ((byte >> 6) & 0x03) as u32;
    // rate = 3200 / 2^(15 - rate_bits)
    let rate = if rate_bits >= 15 {
        3200u32 << (rate_bits - 15)
    } else {
        3200u32 >> (15 - rate_bits)
    };
    // range_g = 16 >> range_bits  (16, 8, 4, 2)
    let range = 16u16 >> range_bits;
    (rate, range)
}

/// Nanoseconds per sample for a given rate byte. Exact integer for all CWA rates.
pub fn nanos_per_sample(rate_byte: u8) -> i64 {
    // ns/sample = 10^9 * 2^(15 - rate_bits) / 3200
    //           = 312_500 * 2^(15 - rate_bits)
    let rate_bits = (rate_byte & 0x0F) as i64;
    let shift = 15 - rate_bits;
    if shift >= 0 {
        312_500i64 << shift
    } else {
        312_500i64 >> (-shift)
    }
}

/// Decode one packed 4-byte sample (3 axes, 10-bit signed, with shared exponent in top 2 bits).
///
/// `eezzzzzzzzzz yyyyyyyyyy xxxxxxxxxx`  (MSB → LSB)
///
/// Returns the three raw integer values *after* applying the exponent shift,
/// but *before* applying `accel_scale` (the per-block accel range scale).
#[inline]
pub fn decode_packed_sample(word: u32) -> (i32, i32, i32) {
    let mut x = (word & 0x3FF) as i32;
    let mut y = ((word >> 10) & 0x3FF) as i32;
    let mut z = ((word >> 20) & 0x3FF) as i32;
    if x >= 0x200 {
        x -= 0x400;
    }
    if y >= 0x200 {
        y -= 0x400;
    }
    if z >= 0x200 {
        z -= 0x400;
    }
    let exponent = (word >> 30) & 0x03;
    (x << exponent, y << exponent, z << exponent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_sample_rate() {
        // rate_bits=10 → 100 Hz, range_bits=1 → 8g  (byte 0x4A from test file)
        assert_eq!(decode_sample_rate(0x4A), (100, 8));
        // rate_bits=15 → 3200 Hz
        assert_eq!(decode_sample_rate(0x0F), (3200, 16));
        // rate_bits=8 → 25 Hz, range_bits=0 → 16g
        assert_eq!(decode_sample_rate(0x08), (25, 16));
    }

    #[test]
    fn test_nanos_per_sample() {
        assert_eq!(nanos_per_sample(0x0A), 10_000_000); // 100 Hz
        assert_eq!(nanos_per_sample(0x09), 20_000_000); // 50 Hz
        assert_eq!(nanos_per_sample(0x08), 40_000_000); // 25 Hz
        assert_eq!(nanos_per_sample(0x07), 80_000_000); // 12.5 Hz
        assert_eq!(nanos_per_sample(0x0F), 312_500); // 3200 Hz
    }

    #[test]
    fn test_decode_packed_sample() {
        // First sample from ax3_testfile.cwa: bytes 15 fc d0 80 → 0x80d0fc15
        // exponent = 2, x=21, y=63, z=13 → after shift: 84, 252, 52
        assert_eq!(decode_packed_sample(0x80d0fc15), (84, 252, 52));

        // Zero word: all zeros, exponent zero.
        assert_eq!(decode_packed_sample(0), (0, 0, 0));

        // All bits set: x=y=z=-1 (10-bit signed), exponent=3 → -8 each
        assert_eq!(decode_packed_sample(0xFFFFFFFF), (-8, -8, -8));
    }

    #[test]
    fn test_decode_timestamp() {
        // 0x4cb4adc7 → 2019-02-26 10:55:07 UTC (first sector of ax3_testfile.cwa)
        let dt = decode_timestamp(0x4cb4adc7, FileLocation::new()).unwrap();
        assert_eq!(dt.timestamp(), 1_551_178_507);
    }

    #[test]
    fn test_decode_timestamp_invalid() {
        // month=0 is invalid
        let result = decode_timestamp(0, FileLocation::new());
        assert!(result.is_err());
    }
}
