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

    fn consume(&mut self) -> Result<(), String> {
        match self.input.next_token() {
            Ok(t) => {
                self.lookahead = t;
                Ok(())
            }
            Err(e) => Err(e),
        }
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

    pub fn file(&mut self) -> Result<(), String> {
        loop {
            if let Err(e) = self.line() {
                return Err(e);
            }
            if self.lookahead.kind == TOKEN_EOF {
                return self.matches(TOKEN_EOF);
            }
        }
    }

    pub fn line(&mut self) -> Result<(), String> {
        let mut alias: Option<Cow<String>> = None;
        if self.lookahead.kind == TOKEN_LBRACK {
            if let Err(e) = self.matches(TOKEN_LBRACK) {
                return Err(e);
            }
            alias = Some(self.lookahead.text.to_owned());
            if let Err(e) = self.alias() {
                return Err(e);
            }
            if let Err(e) = self.matches(TOKEN_RBRACK) {
                return Err(e);
            }
        }
        let path: Option<Cow<String>> = Some(self.lookahead.text.to_owned());
        if let Err(e) = self.path() {
            return Err(e);
        }
        if let Err(e) = self.add_config(alias, path) {
            return Err(e);
        }
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
                if let Some(p) = path {
                    let dir = p.into_owned();
                    match Path::new(&dir).file_stem() {
                        Some(file_stem) => {
                            self.intrep
                                .insert(file_stem.to_str().unwrap().to_lowercase().into(), dir);
                        }
                        None => {
                            return Err("missing file stem in path".into());
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn alias(&mut self) -> Result<(), String> {
        if self.lookahead.kind == TOKEN_ALIAS {
            if let Err(e) = self.matches(TOKEN_ALIAS) {
                return Err(e);
            }
        }
        Ok(())
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
    fn test_parse_file_with_alias_config() {
        let mut p = Parser::new("[alias]/some/absolute/path");
        match p.file() {
            Ok(_) => assert!(true),
            Err(e) => panic!(format!("test failed: {:?}", e)),
        }
    }

    #[test]
    fn test_parse_file_with_single_path() {
        let mut p = Parser::new("/some/absolute/path");
        match p.file() {
            Ok(_) => assert!(true),
            Err(e) => panic!(format!("test failed: {:?}", e)),
        }
    }

    #[test]
    fn test_parse_complex_file() {
        let mut p = Parser::new(
            r#"[alias]/another/absolute/path
        /yet/another/path
        "#,
        );
        match p.file() {
            Ok(_) => assert!(true),
            Err(e) => panic!(format!("test failed: {:?}", e)),
        }
    }

    #[test]
    fn test_parsed_alias_is_lowercase() {
        let mut p = Parser::new("/absolute/Path");
        match p.file() {
            Ok(_) => assert_eq!("/absolute/Path", p.intrep.get("path").unwrap().as_str()),
            Err(e) => panic!(format!("test failed: {:?}", e)),
        }
    }

    #[test]
    fn test_parsed_alias_with_tilde() {
        let mut p = Parser::new(
            r#"
        ~/absolute/Path
        [another-path]~/absolute/Path
        "#,
        );
        match p.file() {
            Ok(_) => {
                assert!(!p.intrep.is_empty());
                assert_eq!("~/absolute/Path", p.intrep.get("path").unwrap().as_str());
                assert_eq!(
                    "~/absolute/Path",
                    p.intrep.get("another-path").unwrap().as_str()
                )
            }
            Err(e) => panic!(format!("test failed: {:?}", e)),
        }
    }
}
