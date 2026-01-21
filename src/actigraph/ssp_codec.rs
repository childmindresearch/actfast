//! SSP floating point codec.
//! <https://github.com/actigraph/GT3X-File-Format/blob/main/LogRecords/Parameters.md>
//! <https://www.scribd.com/document/98159712/Simple-Serial-Protocol-SSP>

const FLOAT_MINIMUM: f64 = 0.00000011920928955078125; /* 2^-23 */
const FLOAT_MAXIMUM: f64 = 8388608.0; /* 2^23  */
const ENCODED_MINIMUM: u32 = 0x00800000;
const ENCODED_MAXIMUM: u32 = 0x007FFFFF;
const SIGNIFICAND_MASK: u32 = 0x00FFFFFF;
const EXPONENT_MINIMUM: i32 = -128;
const EXPONENT_MAXIMUM: i32 = 127;
const EXPONENT_MASK: u32 = 0xFF000000;
const EXPONENT_OFFSET: i32 = 24;

/// Decode SSP floating point format.
#[allow(dead_code)]
pub fn decode(value: u32) -> f64 {
    if ENCODED_MAXIMUM == value {
        return f64::MAX;
    } else if ENCODED_MINIMUM == value {
        return -f64::MAX;
    }

    let mut i32 = ((value & EXPONENT_MASK) >> EXPONENT_OFFSET) as i32;
    if 0 != (i32 & 0x80) {
        i32 |= 0xFFFFFF00u32 as i32;
    }
    let exponent = i32 as f64;

    let mut i32 = (value & SIGNIFICAND_MASK) as i32;
    if 0 != (i32 & ENCODED_MINIMUM as i32) {
        i32 |= 0xFF000000u32 as i32;
    }
    let significand = i32 as f64 / FLOAT_MAXIMUM;

    significand * 2.0f64.powf(exponent)
}

/// Encode SSP floating point format.
#[allow(dead_code)]
pub fn encode(f: f64) -> u32 {
    let mut significand = f.abs();
    if FLOAT_MINIMUM > significand {
        return 0;
    }

    let mut exponent: i32 = 0;
    while 1.0 <= significand {
        significand /= 2.0;
        exponent += 1;
    }
    while 0.5 > significand {
        significand *= 2.0;
        exponent -= 1;
    }

    if EXPONENT_MAXIMUM < exponent {
        return ENCODED_MAXIMUM;
    } else if EXPONENT_MINIMUM > exponent {
        return ENCODED_MINIMUM;
    }

    let mut code = (significand * FLOAT_MAXIMUM) as u32;
    if 0.0 > f {
        code = -(code as i32) as u32;
    }
    code &= SIGNIFICAND_MASK;
    code |= ((exponent << EXPONENT_OFFSET) & EXPONENT_MASK as i32) as u32;

    code
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    // These have been generated using the C# code of the original author
    const TEST_VALUES: [(f64, u32); 18] = [
        (1.0, 0x01400000),
        (2.0, 0x02400000),
        (3.0, 0x02600000),
        (10.0, 0x04500000),
        (10000.0, 0x0E4E2000),
        (0.1, 0xFD666666),
        (1.1, 0x01466666),
        (123.456, 0x077B74BC),
        (1E-05, 0xF053E2D6),
        (-1.0, 0x01C00000),
        (-2.0, 0x02C00000),
        (-3.0, 0x02A00000),
        (-10.0, 0x04B00000),
        (-10000.0, 0x0EB1E000),
        (-0.1, 0xFD99999A),
        (-1.1, 0x01B9999A),
        (-123.456, 0x07848B44),
        (-1E-05, 0xF0AC1D2A),
    ];

    #[test]
    fn test_decode() {
        for (val, expected) in TEST_VALUES {
            let f = decode(expected);
            assert_approx_eq!(val, f, 1e-5);
        }
    }

    #[test]
    fn test_encode() {
        for (val, expected) in TEST_VALUES {
            let e = encode(val);
            assert_eq!(expected, e);
        }
    }

    #[test]
    fn test_encode_decode() {
        for (val, _) in TEST_VALUES {
            let e = encode(val);
            let f = decode(e);
            assert_approx_eq!(val, f, 1e-5);
        }
    }

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn roundtrip_encode_decode(val in -1e6f64..1e6f64) {
            let encoded = encode(val);
            let decoded = decode(encoded);
            // Allow some floating point tolerance
            prop_assert!((val - decoded).abs() < val.abs() * 1e-5 + 1e-10);
        }
    }
}
