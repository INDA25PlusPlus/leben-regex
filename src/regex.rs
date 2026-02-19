use crate::math::{BitMatrix, BitVector};
use crate::regex::graph::{Graph, NodeRef};
use crate::regex::parse::{Atom, ConcatExpr, RegexAst};
use crate::utf8::{UnicodeCodepoint, Utf8DecodeError};
use parsable::Parsable;
use std::collections::HashMap;

mod compile;
mod graph;
mod parse;

pub struct Regex {
    token_matrices: HashMap<UnicodeCodepoint, BitMatrix>,
    final_nodes: BitVector,
}

#[derive(Debug, thiserror::Error)]
pub enum RegexParseError {
    #[error("parse error: 'expected regular expression'")]
    MissingParseResultError,
    #[error(
        "parse error at index {}: 'expected {}'",
        .0.first().map_or(0, |e| e.source_position),
        .0.first().map_or("", |e| &e.error[..]),
    )]
    ParseError(parsable::ParseErrorStack),
}

#[derive(Debug, thiserror::Error)]
pub enum RegexError {
    #[error("{0}")]
    ParseError(RegexParseError),
    #[error("invalid utf8 codepoint: {0}")]
    Utf8DecodeError(Utf8DecodeError),
}

impl Regex {
    pub fn parse_str(source: &str) -> Result<Regex, RegexParseError> {
        Regex::parse(source.as_bytes()).map_err(|e| match e {
            RegexError::ParseError(e) => e,
            RegexError::Utf8DecodeError(_) => panic!(
                "valid UTF-8 string shouldn't result in UTF-8 decoding error"
            ),
        })
    }

    pub fn parse(source: &[u8]) -> Result<Regex, RegexError> {
        let mut stream = parsable::ScopedStream::new(source);
        let outcome = RegexAst::parse(&mut stream);
        let regex = match outcome {
            None => {
                return Err(RegexError::ParseError(
                    RegexParseError::MissingParseResultError,
                ));
            }
            Some(result) => match result {
                Ok(regex) => regex,
                Err(e) => {
                    return Err(RegexError::ParseError(
                        RegexParseError::ParseError(e),
                    ));
                }
            },
        };

        let mut graph = Graph::new();
        let start_node = graph.get_initial_node();
        let final_node = graph.add_node();
        graph.set_final(final_node);

        for a in regex.root.node.alts.nodes {
            add_alt(&mut graph, start_node, final_node, a)
                .map_err(RegexError::Utf8DecodeError)?;
        }

        graph.collapse_epsilons();

        let (token_matrices, final_nodes) = graph.compile();

        Ok(Regex {
            token_matrices,
            final_nodes,
        })
    }
}

fn add_alt(
    graph: &mut Graph,
    start: NodeRef,
    end: NodeRef,
    alt: ConcatExpr,
) -> Result<(), Utf8DecodeError> {
    let mut prev = start;
    for p in alt.parts.nodes {
        let is_kleene = p.star.is_some();
        let next = if is_kleene { prev } else { graph.add_node() };
        match p.atom {
            Atom::CharacterAtom(c) => {
                let token = c.to_codepoint()?;
                graph.connect(prev, next, token);
            }
            Atom::Capture { alt, .. } => {
                for a in alt.alts.nodes {
                    add_alt(graph, prev, next, a)?;
                }
            }
        }
        prev = next;
    }
    if prev != end {
        graph.connect_epsilon(prev, end);
    }
    Ok(())
}
