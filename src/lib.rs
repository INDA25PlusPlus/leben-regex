mod math;
pub mod regex;
pub mod utf8;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let r = "a(a(b|cd)*|ab)*c".as_bytes();
        regex::Regex::parse(r).unwrap();
    }
}
