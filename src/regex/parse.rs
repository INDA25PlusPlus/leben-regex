use crate::utf8::{UnicodeCodepoint, Utf8DecodeError, decode_utf8};
use parsable::{
    CharLiteral, CharRange, Intersperse, Parsable, RepeatLimited, Span,
    WithEnd, ZeroPlus,
};
use serde::Serialize;

#[derive(Debug, Parsable, Serialize)]
pub struct RegexAst {
    pub root: WithEnd<AltExpr>,
}

#[derive(Debug, Parsable, Serialize)]
pub struct AltExpr {
    pub alts: Intersperse<ConcatExpr, CharLiteral<b'|'>>,
}

#[derive(Debug, Parsable, Serialize)]
pub struct ConcatExpr {
    pub parts: ZeroPlus<KleeneExpr>,
}

#[derive(Debug, Parsable, Serialize)]
pub struct KleeneExpr {
    pub atom: Atom,
    pub star: Option<CharLiteral<b'*'>>,
}

#[derive(Debug, Parsable, Serialize)]
pub enum Atom {
    CharacterAtom(Character),
    Capture {
        _0: CharLiteral<b'('>,
        alt: AltExpr,
        _1: CharLiteral<b')'>,
    },
}

#[derive(Debug, Parsable, Serialize)]
pub enum Character {
    Ascii(Span<AsciiCharacter>),
    Unicode(Span<UnicodeCharacter>),
    Escaped(EscapedCharacter),
}

impl Character {
    pub fn to_codepoint(&self) -> Result<UnicodeCodepoint, Utf8DecodeError> {
        match self {
            Character::Ascii(s) => Ok(UnicodeCodepoint::try_from(
                *s.span
                    .first()
                    .expect("ascii character span should not be empty")
                    as u32,
            )
            .expect("ascii character should be a valid unicode codepoint")),
            Character::Unicode(s) => {
                let s = decode_utf8(&s.span)?;
                assert_eq!(
                    s.len(),
                    1,
                    "single unicode codepoint should be decoded as exactly one codepoint"
                );
                Ok(*s.first().unwrap())
            }
            Character::Escaped(e) => match e {
                EscapedCharacter::LeftParen => Ok('('.into()),
                EscapedCharacter::RightParen => Ok(')'.into()),
                EscapedCharacter::Asterisk => Ok('*'.into()),
                EscapedCharacter::Backslash => Ok('\\'.into()),
                EscapedCharacter::VerticalBar => Ok('|'.into()),
            },
        }
    }
}

#[derive(Debug, Parsable, Serialize)]
pub enum AsciiCharacter {
    Ascii1(CharRange<b' ', b'\''>),
    // skip ( ) *
    Ascii2(CharRange<b'+', b'['>),
    // skip \
    Ascii3(CharRange<b']', b'{'>),
    // skip |
    Ascii4(CharRange<b'}', b'~'>),
}

#[derive(Debug, Parsable, Serialize)]
pub struct UnicodeCharacter {
    pub b0: CharRange<0b1100_0000, 0b1111_0111>,
    pub bytes: RepeatLimited<CharRange<0b1000_0000, 0b1011_1111>, 1, 3>,
}

#[derive(Debug, Parsable, Serialize)]
pub enum EscapedCharacter {
    #[literal = b"\\("]
    LeftParen,
    #[literal = b"\\)"]
    RightParen,
    #[literal = b"\\*"]
    Asterisk,
    #[literal = b"\\\\"]
    Backslash,
    #[literal = b"\\|"]
    VerticalBar,
}
