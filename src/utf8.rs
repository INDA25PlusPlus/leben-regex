use crate::utf8::UnicodeError::{OutsideOfRange, SurrogateCodepoint};
use thiserror::Error;

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Default, Hash)]
pub struct UnicodeCodepoint(u32);

impl From<char> for UnicodeCodepoint {
    fn from(value: char) -> Self {
        UnicodeCodepoint(value.into())
    }
}

impl From<UnicodeCodepoint> for char {
    fn from(value: UnicodeCodepoint) -> Self {
        char::from_u32(value.0).unwrap()
    }
}

impl TryFrom<u32> for UnicodeCodepoint {
    type Error = UnicodeError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value < 0x00_d800 {
            Ok(UnicodeCodepoint(value))
        } else if value < 0x00_e000 {
            Err(SurrogateCodepoint(value))
        } else if value < 0x11_0000 {
            Ok(UnicodeCodepoint(value))
        } else {
            Err(OutsideOfRange(value))
        }
    }
}

impl From<UnicodeCodepoint> for u32 {
    fn from(value: UnicodeCodepoint) -> Self {
        value.0
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Error)]
pub enum UnicodeError {
    #[error("surrogate codepoint {0:#034x} (not a valid codepoint)")]
    SurrogateCodepoint(u32),
    #[error("integer {0:#034x} outside of unicode codepoint range")]
    OutsideOfRange(u32),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Error)]
pub enum Utf8DecodeError {
    #[error("unexpected end of stream")]
    UnexpectedEndOfStream,
    #[error("overlong encoding {0:#034x}")]
    OverlongEncoding(u32),
    #[error("{0:}")]
    UnicodeError(UnicodeError),
    #[error("invalid byte sequence {0:#010x}")]
    InvalidByte1(u8),
    #[error("invalid byte sequence {0:#010x}_{1:08x}")]
    InvalidByte2(u8, u8),
    #[error("invalid byte sequence {0:#010x}_{1:08x}_{2:08x}")]
    InvalidByte3(u8, u8, u8),
    #[error("invalid byte sequence {0:#010x}_{1:08x}_{2:08x}_{3:08x}")]
    InvalidByte4(u8, u8, u8, u8),
}

#[must_use]
pub fn encode_utf8(unicode: &[UnicodeCodepoint]) -> Vec<u8> {
    let mut out = Vec::<u8>::new();
    for UnicodeCodepoint(c) in unicode {
        let c = *c;
        if c < 0x1 << 7 {
            out.push(trunc_u8(c));
        } else if c < 0x1 << 11 {
            out.push(0b1100_0000 | trunc_u8(c >> 6));
            out.push(0b1000_0000 | trunc_u8(c & 0b0011_1111));
        } else if c < 0x1 << 16 {
            out.push(0b1110_0000 | trunc_u8(c >> 12));
            out.push(0b1000_0000 | trunc_u8((c >> 6) & 0b0011_1111));
            out.push(0b1000_0000 | trunc_u8(c & 0b0011_1111));
        } else {
            out.push(0b1111_0000 | trunc_u8(c >> 18));
            out.push(0b1000_0000 | trunc_u8((c >> 12) & 0b0011_1111));
            out.push(0b1000_0000 | trunc_u8((c >> 6) & 0b0011_1111));
            out.push(0b1000_0000 | trunc_u8(c & 0b0011_1111));
        }
    }
    out
}

pub fn decode_utf8(
    utf8: &[u8],
) -> Result<Vec<UnicodeCodepoint>, Utf8DecodeError> {
    let mut out = Vec::<UnicodeCodepoint>::new();
    let mut iter = utf8.iter();
    while let Some(b0) = iter.next() {
        let b0 = u32::from(*b0);
        if b0 >> 7 == 0 {
            out.push(UnicodeCodepoint(b0));
            continue;
        }

        let b1 = u32::from(
            *iter.next().ok_or(Utf8DecodeError::UnexpectedEndOfStream)?,
        );
        if b1 >> 6 != 0b10 {
            return Err(Utf8DecodeError::InvalidByte2(
                trunc_u8(b0),
                trunc_u8(b1),
            ));
        }
        if b0 >> 5 == 0b110 {
            let c = ((b0 & 0b0001_1111) << 6) | (b1 & 0b0011_1111);
            if c < 0x00_0080 {
                return Err(Utf8DecodeError::OverlongEncoding(c));
            }
            out.push(UnicodeCodepoint(c));
            continue;
        }

        let b2 = u32::from(
            *iter.next().ok_or(Utf8DecodeError::UnexpectedEndOfStream)?,
        );
        if b2 >> 6 != 0b10 {
            return Err(Utf8DecodeError::InvalidByte3(
                trunc_u8(b0),
                trunc_u8(b1),
                trunc_u8(b2),
            ));
        }
        if b0 >> 4 == 0b1110 {
            let c = ((b0 & 0b0000_1111) << 12)
                | ((b1 & 0b0011_1111) << 6)
                | (b2 & 0b0011_1111);
            if c < 0x00_0800 {
                return Err(Utf8DecodeError::OverlongEncoding(c));
            }
            if (0x00_d800..0x00_e000).contains(&c) {
                return Err(Utf8DecodeError::UnicodeError(SurrogateCodepoint(
                    c,
                )));
            }
            out.push(UnicodeCodepoint(c));
            continue;
        }

        let b3 = u32::from(
            *iter.next().ok_or(Utf8DecodeError::UnexpectedEndOfStream)?,
        );
        if b3 >> 6 != 0b10 {
            return Err(Utf8DecodeError::InvalidByte4(
                trunc_u8(b0),
                trunc_u8(b1),
                trunc_u8(b2),
                trunc_u8(b3),
            ));
        }
        if b0 >> 3 == 0b1_1110 {
            let c = ((b0 & 0b0000_0111) << 18)
                | ((b1 & 0b0011_1111) << 12)
                | ((b2 & 0b0011_1111) << 6)
                | (b3 & 0b0011_1111);
            if c < 0x01_0000 {
                return Err(Utf8DecodeError::OverlongEncoding(c));
            }
            out.push(UnicodeCodepoint(c));
            continue;
        }

        // invalid first byte sequence, matching one of these patterns:
        // 10xxxxxx
        // 11111xxx
        return Err(Utf8DecodeError::InvalidByte1(trunc_u8(b0)));
    }
    Ok(out)
}

#[allow(clippy::cast_possible_truncation)]
fn trunc_u8(x: u32) -> u8 {
    x as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_unicode() {
        for i in (0x00_0000u32..0x00_d800).chain(0x00_e000..0x11_0000) {
            let c = UnicodeCodepoint::from(char::from_u32(i).unwrap());
            assert_eq!(i, c.0);
            let c = UnicodeCodepoint::try_from(i).unwrap();
            assert_eq!(i, c.0);
            _ = char::from_u32(i).unwrap();
        }
    }

    #[test]
    fn to_unicode_invalid() {
        for i in 0x00_d800..0x00_e000 {
            assert_eq!(
                Err(SurrogateCodepoint(i)),
                UnicodeCodepoint::try_from(i)
            );
            assert_eq!(None, char::from_u32(i));
        }
        for i in [0x11_0000, 0x12_0000, 0x1000_0000, 0xffff_ffff] {
            assert_eq!(Err(OutsideOfRange(i)), UnicodeCodepoint::try_from(i));
        }
    }

    #[test]
    fn utf8_coding() {
        let strings = ["", "test", "\0\0\0", "🔥✅😄", "中文", "t̶̡̨͇̗͙͒͌͆̄̽̾̈́̇̈́͂́̅͘͝͠͠͝ę̸̢̛͔̱͕͍͚̗͔̰̗͎͚̣̐͂̋̃̉̈́͒̒̒́͆̉̽̕͘͝͝s̷̼̘͔͇̺͒̒̑͒̈͘͘t̵̡̧̡̧̡̹̹͖̣͚̯̩̤͕̩̝͓̦̾̂̑͊̿̿̇̕̕ ̶̛̞̼͖̬̟̠̿̇̂̓̿͛̆̏̀̑̑̀́͗̆́͂̚̚͘ś̵̲̤͉̙̻̲͜ͅt̸̡̨̯̫̯̭̦̻͎̰̝̹͉̻̖̭̞̺̠̰͍̏̎̒̈́͐͗͋͜r̸̨̛͎͔̰͇̐̂͊́̉̐͌̎̒͌́̒͛́̊̏̏̂̚͘͝į̸̢̨̖̟̲̰̣͓̫̪̯͍̤̘̱̼̘̜̙̻̔̈́͒́̀n̷̖͉̳͔͙̪̝̦͖͔̦̓͆̇̈́͂̑̒̇̈̈́́͊͆̈̃̄̀̉̈̆̿̐̆̕͘͝ͅg̵̨̧̛̻̝̝͈̼͍̻̝͓̖͇̟͛̂̈̿̅̋͆̈́͒̓́͆̈́͂̈́͐̏̔̐́̎͊̆̚̕͜͝ͅ"];
        for s in strings {
            let bytes = s.as_bytes();
            let unicode = decode_utf8(bytes).unwrap();
            let encoded = encode_utf8(&unicode);
            assert_eq!(bytes, encoded);
        }
    }

    #[test]
    fn utf8_invalid() {
        let strings: [&[u8]; 7] = [
            &[0xc3, 0x28],
            &[0xa0, 0xa1],
            &[0xe2, 0x28, 0xa1],
            &[0xe2, 0x82, 0x28],
            &[0xf0, 0x28, 0x8c, 0xbc],
            &[0xf0, 0x90, 0x28, 0xbc],
            &[0xf0, 0x28, 0x8c, 0x28],
        ];
        for s in strings {
            assert!(matches!(decode_utf8(s), Err(..)));
        }
    }
}
