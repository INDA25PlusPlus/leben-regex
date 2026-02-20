use leben_regex::UnicodeCodepoint;
use std::io::Read;

fn main() -> Result<(), anyhow::Error> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 2 {
        anyhow::bail!("Usage: EXE <regex>");
    }
    let regex_format_string = args.get(1).unwrap().as_encoded_bytes();
    let regex = leben_regex::Regex::new(regex_format_string)?;

    let mut buffer = Vec::new();
    std::io::stdin().read_to_end(&mut buffer)?;

    let string = leben_regex::decode_utf8(&buffer)?;

    if let Some((match_index, len)) = regex.find(&string) {
        let match_end = match_index + len;
        let print_start = rfind_lf(&string, match_index);
        let print_end = find_lf(&string, match_end);
        print!(
            "{}",
            leben_regex::encode_utf8_string(&string[print_start..match_index])
        );
        print!(
            "\x1b[91m{}\x1b[m",
            leben_regex::encode_utf8_string(&string[match_index..match_end])
        );
        println!(
            "{}",
            leben_regex::encode_utf8_string(&string[match_end..print_end])
        );
    } else {
        println!("No match found!");
    }

    Ok(())
}

fn find_lf(string: &[UnicodeCodepoint], index: usize) -> usize {
    let lf_chars: [UnicodeCodepoint; 2] =
        [UnicodeCodepoint::from('\n'), UnicodeCodepoint::from('\r')];
    string[index..]
        .iter()
        .zip(index..)
        .find(|(c, _)| lf_chars.contains(*c))
        .map_or(string.len(), |(_, i)| i)
}

fn rfind_lf(string: &[UnicodeCodepoint], index: usize) -> usize {
    let lf_chars: [UnicodeCodepoint; 2] =
        [UnicodeCodepoint::from('\n'), UnicodeCodepoint::from('\r')];
    string[..index]
        .iter()
        .zip(0..index)
        .rfind(|(c, _)| lf_chars.contains(*c))
        .map_or(0, |(_, i)| i + 1)
}
