use parsable::{CharLiteral, CharRange, Intersperse, Parsable, RepeatLimited, Span, WithEnd, ZeroPlus};
use serde::Serialize;

#[derive(Debug, Parsable, Serialize)]
pub struct RegexAst {
    pub alt: WithEnd<AltExpr>,
}

#[derive(Debug, Parsable, Serialize)]
pub struct AltExpr {
    pub concats: Intersperse<ConcatExpr, CharLiteral<b'|'>>,
}

#[derive(Debug, Parsable, Serialize)]
pub struct ConcatExpr {
    pub kleenes: ZeroPlus<KleeneExpr>,
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
