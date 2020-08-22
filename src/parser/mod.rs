use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;

use lexer::{Lexer, TOKEN_ALIAS, TOKEN_EOF, TOKEN_LBRACK, TOKEN_PATH, TOKEN_RBRACK};

mod lexer;

#[derive(Debug)]
pub struct Parser<'a> {
    /// The lexer responsible for returning tokenized input.
    input: Lexer<'a>,
    /// The current lookahead token used used by this parser.
    lookahead: lexer::Token<'a>,
    intrep: HashMap<String, String>,
}

impl<'a> Parser<'a> {
    pub fn new(s: &str) -> Self {
        if s.trim().len() == 0 {
            panic!("no input provided")
        }
        let c = s.chars().nth(0).unwrap();
        let mut input = Lexer::new(s, 0, c);
        match input.next_token() {
            Ok(lookahead) => {
                return Self {
                    input,
                    lookahead,
                    intrep: HashMap::new(),
                };
            }
            Err(e) => panic!("couldn't create new parser: {}", e),
        }
    }

    pub fn aliases(&self) -> HashMap<String, String> {
        self.intrep.to_owned()
    }

    fn consume(&mut self) -> Result<(), String> {
        self.lookahead = self.input.next_token()?;
        Ok(())
    }

    fn matches(&mut self, k: i32) -> Result<(), String> {
        if self.lookahead.kind == k {
            return self.consume();
        }
        Err(format!(
            "expecting {}; found {}",
            self.input.token_names(k as usize),
            self.lookahead
        ))
    }

    fn file(&mut self) -> Result<(), String> {
        loop {
            self.line()?;
            if self.lookahead.kind == TOKEN_EOF {
                return self.matches(TOKEN_EOF);
            }
        }
    }

    pub fn process_input(&mut self) -> Result<(), String> {
        self.file()
    }

    pub fn line(&mut self) -> Result<(), String> {
        let mut alias: Option<Cow<String>> = None;
        if self.lookahead.kind == TOKEN_LBRACK {
            self.matches(TOKEN_LBRACK)?;

            alias = Some(self.lookahead.text.to_owned());
            self.alias()?;

            self.matches(TOKEN_RBRACK)?
        }
        let path: Option<Cow<String>> = Some(self.lookahead.text.to_owned());
        self.path()?;
        self.add_config(alias, path)?;
        Ok(())
    }

    fn add_config(
        &mut self,
        alias: Option<Cow<String>>,
        path: Option<Cow<String>>,
    ) -> Result<(), String> {
        match alias {
            Some(a) => {
                self.intrep.insert(
                    a.to_owned().parse().unwrap(),
                    path.unwrap().to_owned().parse().unwrap(),
                );
            }
            None => {
                self.insert_alias_from_path(path);
            }
        }
        Ok(())
    }

    fn insert_alias_from_path(&mut self, path: Option<Cow<String>>) -> Option<String> {
        let dir = path?.into_owned();
        let file_stem = Path::new(&dir).file_stem()?;
        let alias = file_stem.to_str()?;
        self.intrep.insert(alias.to_lowercase().into(), dir)
    }

    fn alias(&mut self) -> Result<(), String> {
        self.matches(TOKEN_ALIAS)
    }

    fn path(&mut self) -> Result<(), String> {
        self.matches(TOKEN_PATH)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;

    #[test]
    fn test_create_parser() {
        let p = Parser::new("/some/absolute/path");
        assert_eq!(
            lexer::Token::new(TOKEN_PATH, Cow::Owned("/some/absolute/path".into())),
            p.lookahead
        );
    }

    #[test]
    #[should_panic]
    fn test_create_parser_panics() {
        Parser::new("");
    }

    #[test]
    #[should_panic]
    fn test_create_parser_panics_with_empty_str() {
        Parser::new("    ");
    }

    #[test]
    fn test_parser_consume() {
        let mut p = Parser::new("[alias]/some/absolute/path");
        let _ = p.consume();
        assert_eq!(
            lexer::Token::new(TOKEN_ALIAS, Cow::Owned("alias".into())),
            p.lookahead
        );
    }

    #[test]
    fn test_parser_matches() {
        let mut p = Parser::new("[alias]/some/absolute/path");
        let _ = p.matches(TOKEN_LBRACK);
        assert_eq!(
            lexer::Token::new(TOKEN_ALIAS, Cow::Owned("alias".into())),
            p.lookahead
        );
    }

    #[test]
    fn test_parser_does_not_match() {
        let mut p = Parser::new("[alias]/some/absolute/path");
        if let Err(e) = p.matches(TOKEN_RBRACK) {
            assert_eq!("expecting RBRACK; found <'[', LBRACK>", e);
        }
    }

    #[test]
    fn test_parse_file_with_alias_config() -> Result<(), String> {
        let mut p = Parser::new("[alias]/some/absolute/path");
        p.file()?;
        Ok(())
    }

    #[test]
    fn test_parse_file_with_single_path() -> Result<(), String> {
        let mut p = Parser::new("/some/absolute/path");
        p.file()?;
        Ok(())
    }

    #[test]
    fn test_parse_fails_with_invalid_path() {
        let input = "some/absolute/path";
        let mut p = Parser::new(input);
        let result: Result<(), String> = p.file();
        assert_eq!(result.unwrap_err(), "expecting PATH; found <'some', ALIAS>")
    }

    #[test]
    fn test_parse_complex_file() -> Result<(), String> {
        let mut p = Parser::new(
            r#"[alias]/another/absolute/path
        /yet/another/path
        "#,
        );
        p.file()?;
        assert!(!p.intrep.is_empty());
        assert_eq!(2, p.intrep.len());
        assert_eq!("/another/absolute/path", p.intrep.get("alias").unwrap());
        assert_eq!("/yet/another/path", p.intrep.get("path").unwrap());
        Ok(())
    }

    #[test]
    fn test_parsed_alias_is_lowercase() -> Result<(), String> {
        let mut p = Parser::new("/absolute/Path");
        p.file()?;
        assert_eq!("/absolute/Path", p.intrep.get("path").unwrap().as_str());
        Ok(())
    }

    #[test]
    fn test_parsed_alias_with_tilde() -> Result<(), String> {
        let mut p = Parser::new(
            r#"
        ~/absolute/Path
        [another-path]~/absolute/Path
        "#,
        );
        p.file()?;
        assert!(!p.intrep.is_empty());
        assert_eq!("~/absolute/Path", p.intrep.get("path").unwrap().as_str());
        assert_eq!(
            "~/absolute/Path",
            p.intrep.get("another-path").unwrap().as_str()
        );
        Ok(())
    }
}
