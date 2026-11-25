//! The color value type, written `#RRGGBB` or `#RRGGBBAA` in an expression.

/// An RGBA color. Every combination of channels is a valid color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Reads the digits after a color's `#`: exactly 6 hex digits (opaque) or 8 (with alpha), in either case.
    pub(crate) fn from_hex(digits: &str) -> Option<Color> {
        if !matches!(digits.len(), 6 | 8) || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let channel = |index: usize| u8::from_str_radix(&digits[index..index + 2], 16).ok();
        let alpha = if digits.len() == 8 {
            channel(6)?
        } else {
            u8::MAX
        };
        Some(Color {
            r: channel(0)?,
            g: channel(2)?,
            b: channel(4)?,
            a: alpha,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_digits_are_opaque() {
        let expected = Color {
            r: 0xFF,
            g: 0x88,
            b: 0x00,
            a: 0xFF,
        };
        assert_eq!(Color::from_hex("FF8800"), Some(expected));
    }

    #[test]
    fn eight_digits_carry_alpha_in_either_case() {
        let expected = Color {
            r: 0xFF,
            g: 0x88,
            b: 0x00,
            a: 0x80,
        };
        assert_eq!(Color::from_hex("ff880080"), Some(expected));
        assert_eq!(Color::from_hex("Ff880080"), Some(expected));
    }

    #[test]
    fn other_lengths_and_non_hex_digits_are_rejected() {
        for digits in ["", "FFF", "FF88000", "FF8800FF00", "12345G", "FF88+0"] {
            assert_eq!(Color::from_hex(digits), None, "{digits:?}");
        }
    }
}
