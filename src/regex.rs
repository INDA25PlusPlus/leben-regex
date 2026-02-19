use crate::regex::parse::RegexAst;
use parsable::{format_error_stack, Parsable, ScopedStream};

mod compile;
mod graph;
mod parse;

struct Regex {}

impl Regex {
    pub fn parse(buffer: &[u8]) -> Result<Regex, ()> {
        let mut stream = ScopedStream::new(buffer);
        let regex = RegexAst::parse(&mut stream).expect("failed to parse");

        match regex {
            Ok(parsed) => {
                let config = ron::ser::PrettyConfig::default();
                let output =
                    ron::ser::to_string_pretty(&parsed, config).unwrap();
                let output_path = std::path::Path::new("out.ron");
                std::fs::write(output_path, output).expect(
                    "failed to write file",
                );
            }
            Err(err) => {
                eprintln!("{}", format_error_stack(&buffer, err));
            }
        }
        todo!()
    }
}
