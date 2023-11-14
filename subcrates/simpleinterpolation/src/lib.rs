//! SimpleInterpolation
//!
//! A dead simple interpolation format
//! `this is an {interpolated} string`
//! Variable names may have `-` `_`, `a-z`, and `A-Z`, any other charc
//!
//! #
use std::{collections::HashMap, fmt::Formatter};

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct Interpolation {
    // The first value is the raw value that will be appended
    // to the final string. The second value will go AFTER this string,
    // but it may be dynamic, and will be empty if unset
    parts: Vec<(String, String)>,
}

impl Interpolation {
    const REASONABLE_INTERPOLATION_PREALLOC_BYTES: usize = 128;

    pub fn new(input: String) -> Result<Self, Error> {
        InterpolationCompiler::compile(input)
    }

    fn output_string(&self) -> String {
        String::with_capacity(
            self.parts
                .iter()
                .map(|v| v.0.len() + Self::REASONABLE_INTERPOLATION_PREALLOC_BYTES)
                .sum(),
        )
    }

    pub fn render(&self, args: &HashMap<String, String>) -> String {
        let mut output = self.output_string();
        for (raw, interpolation_key) in &self.parts {
            output.push_str(raw);
            let interpolation_value = args.get(interpolation_key);
            output.push_str(interpolation_value.unwrap_or(&String::new()));
        }
        output
    }

    pub fn render_transform<T, F: Fn(&T) -> String>(
        &self,
        args: &HashMap<String, T>,
        transform: F,
    ) -> String {
        let mut output = self.output_string();
        for (raw, interpolation_key) in &self.parts {
            output.push_str(raw);
            let interpolation_value = args.get(interpolation_key);
            output.push_str(&interpolation_value.map_or(String::new(), &transform));
        }
        output
    }
}

struct InterpolationCompiler {
    chars: Vec<char>,
    parts: Vec<(String, String)>,
    index: usize,
    next: String,
    escaped: bool,
}

// TODO: figure out why the heck this is outputting extra strings
impl InterpolationCompiler {
    fn compile(input: String) -> Result<Interpolation, Error> {
        let mut compiler = Self {
            chars: input.chars().collect(),
            parts: Vec::new(),
            index: 0,
            next: String::new(),
            escaped: false,
        };

        // for each character, check if the character is a
        while let Some(character) = compiler.chars.get(compiler.index).copied() {
            compiler.handle_char(character)?;
        }

        // Push the final part and return self
        compiler.parts.push((compiler.next, String::new()));
        compiler.parts.shrink_to_fit();
        Ok(Interpolation {
            parts: compiler.parts,
        })
    }

    fn handle_char(&mut self, ch: char) -> Result<(), Error> {
        if self.escaped && ch != '{' && ch != '\\' {
            return Err(Error::InvalidEscape(ch));
        } else if self.escaped {
            self.next.push(ch);
            self.escaped = false;
        } else if ch == '\\' {
            self.escaped = true;
        } else if ch == '{' {
            self.index += 1;
            let ident = self.make_identifier()?;
            let to_push = std::mem::take(&mut self.next);
            self.parts.push((to_push, ident));
        } else {
            self.next.push(ch);
        };
        self.index += 1;
        Ok(())
    }

    #[inline]
    fn valid_ident_char(ch: char) -> bool {
        matches!(ch, 'A'..='Z' | 'a'..='z' | '_' | '-')
    }

    fn make_identifier(&mut self) -> Result<String, Error> {
        let mut identifier = String::new();
        while let Some(identifier_part) = self.chars.get(self.index).copied() {
            if identifier_part == '}' {
                break;
            }
            if self.index >= self.chars.len() {
                return Err(Error::UnclosedIdentifier);
            }
            if !Self::valid_ident_char(identifier_part) {
                return Err(Error::InvalidCharInIdentifier(identifier_part));
            }
            identifier.push(identifier_part);
            self.index += 1;
        }
        Ok(identifier)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Error {
    UnclosedIdentifier,
    InvalidCharInIdentifier(char),
    InvalidEscape(char),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnclosedIdentifier => write!(f, "Unclosed identifier"),
            Self::InvalidCharInIdentifier(c) => {
                write!(f, "Invalid character in identifier: `{c:?}`")
            }
            Self::InvalidEscape(c) => {
                write!(f, "`{c:?}` cannot be escaped, only `{{` and `\\` can")
            }
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn get_example_args() -> HashMap<String, String> {
        let mut hm = HashMap::new();
        hm.insert("interpolation".to_string(), "Interpolation".to_string());
        hm.insert("unused".to_string(), "ERROR".to_string());
        hm
    }
    #[test]
    fn basic() {
        let interpolation =
            Interpolation::new("This is an example string for {interpolation}!".to_string())
                .unwrap();
        println!("{interpolation:?}");
        println!("{}", interpolation.render(&get_example_args()));
    }
    #[test]
    fn escapes() {
        let interpolation = Interpolation::new(
            "This is an example string for \\{interpolation} escapes!".to_string(),
        )
        .unwrap();
        println!("{interpolation:?}");
        println!("{}", interpolation.render(&get_example_args()));
    }
    #[test]
    fn recursive_escapes() {
        let interpolation = Interpolation::new(
            "This is an example string for \\\\{interpolation} recursive escapes!".to_string(),
        )
        .unwrap();
        println!("{interpolation:?}");
        println!("{}", interpolation.render(&get_example_args()));
    }
    #[test]
    fn no_interpolation() {
        let interpolation = Interpolation::new(
            "This is an example string for a lack of interpolation!".to_string(),
        )
        .unwrap();
        println!("{interpolation:?}");
        println!("{}", interpolation.render(&get_example_args()));
    }
}
