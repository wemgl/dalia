#[cfg(test)]
extern crate temp_testdir;

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;

use crate::lexer::{
    Lexer, Token, TOKEN_ALIAS, TOKEN_EOF, TOKEN_GLOB, TOKEN_LBRACK, TOKEN_PATH, TOKEN_RBRACK,
};

#[derive(Debug)]
pub struct Parser<'a> {
    /// The lexer responsible for returning tokenized input.
    input: Lexer<'a>,
    /// The current lookahead token used by this parser.
    lookahead: Token<'a>,
    /// The internal representation of a parsed configuration file.
    int_rep: HashMap<String, String>,
}

impl<'a> Parser<'a> {
    pub fn new(s: &str) -> Self {
        if s.trim().is_empty() {
            panic!("no config file found to parse")
        }
        let c = s.chars().next().unwrap();
        let mut input = Lexer::new(s, 0, c);
        match input.next_token() {
            Ok(lookahead) => Self {
                input,
                lookahead,
                int_rep: HashMap::new(),
            },
            Err(e) => panic!("couldn't create new parser: {}", e),
        }
    }

    pub fn aliases(&self) -> HashMap<String, String> {
        self.int_rep.to_owned()
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
        let mut is_glob: bool = false;
        if self.lookahead.kind == TOKEN_LBRACK {
            self.matches(TOKEN_LBRACK)?;

            if self.lookahead.kind == TOKEN_GLOB {
                is_glob = true;
                self.glob()?;
            } else if self.lookahead.kind == TOKEN_ALIAS {
                alias = Some(self.lookahead.text.to_owned());
                self.alias()?;
            }

            self.matches(TOKEN_RBRACK)?
        }
        let path: Option<Cow<String>> = Some(self.lookahead.text.to_owned());
        self.path()?;
        if is_glob {
            self.expand_glob_paths(path);
        } else {
            self.add_path_alias(alias, path);
        }
        Ok(())
    }

    fn add_path_alias(&mut self, alias: Option<Cow<String>>, path: Option<Cow<String>>) {
        match alias {
            Some(a) => {
                self.int_rep.insert(
                    a.to_owned().parse().unwrap(),
                    path.unwrap().to_owned().parse().unwrap(),
                );
            }
            None => {
                self.insert_alias_from_path(path);
            }
        }
    }

    fn expand_glob_paths(&mut self, path: Option<Cow<String>>) {
        let dir: String = path.unwrap().parse().unwrap();
        let paths = std::fs::read_dir(dir).unwrap();
        for path in paths {
            if let Ok(entry) = path {
                if entry.metadata().unwrap().is_file() {
                    continue;
                }
                self.insert_alias_from_path(Some(Cow::Owned(
                    entry.path().to_str().unwrap().to_string(),
                )));
            }
        }
    }

    fn insert_alias_from_path(&mut self, path: Option<Cow<String>>) -> Option<String> {
        let dir = path?.into_owned();
        let file_stem = Path::new(&dir).file_stem()?;
        let alias = file_stem.to_str()?;
        self.int_rep.insert(alias.to_lowercase(), dir)
    }

    fn alias(&mut self) -> Result<(), String> {
        self.matches(TOKEN_ALIAS)
    }

    fn glob(&mut self) -> Result<(), String> {
        self.matches(TOKEN_GLOB)
    }

    fn path(&mut self) -> Result<(), String> {
        self.matches(TOKEN_PATH)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::fs::create_dir;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_create_parser() {
        let p = Parser::new("/some/absolute/path");
        assert_eq!(
            Token::new(TOKEN_PATH, Cow::Owned("/some/absolute/path".into())),
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
            Token::new(TOKEN_ALIAS, Cow::Owned("alias".into())),
            p.lookahead
        );
    }

    #[test]
    fn test_parser_matches() {
        let mut p = Parser::new("[alias]/some/absolute/path");
        let _ = p.matches(TOKEN_LBRACK);
        assert_eq!(
            Token::new(TOKEN_ALIAS, Cow::Owned("alias".into())),
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
        assert!(!p.int_rep.is_empty());
        assert_eq!(2, p.int_rep.len());
        assert_eq!("/another/absolute/path", p.int_rep.get("alias").unwrap());
        assert_eq!("/yet/another/path", p.int_rep.get("path").unwrap());
        Ok(())
    }

    #[test]
    fn test_parsed_alias_is_lowercase() -> Result<(), String> {
        let mut p = Parser::new("/absolute/Path");
        p.file()?;
        assert_eq!("/absolute/Path", p.int_rep.get("path").unwrap().as_str());
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
        assert!(!p.int_rep.is_empty());
        assert_eq!("~/absolute/Path", p.int_rep.get("path").unwrap().as_str());
        assert_eq!(
            "~/absolute/Path",
            p.int_rep.get("another-path").unwrap().as_str()
        );
        Ok(())
    }

    #[test]
    fn test_parse_glob_asterisk() -> Result<(), String> {
        let temp = temp_testdir::TempDir::default();
        let file_path = PathBuf::from(temp.as_ref());

        let path1 = format!("{}/one", file_path.to_str().unwrap());
        if let Err(e) = create_dir(&path1) {
            return Err(format!("couldn't create temp dir one: {}", e));
        }

        let path2 = format!("{}/two", file_path.to_str().unwrap());
        if let Err(e) = create_dir(&path2) {
            return Err(format!("couldn't create temp dir two: {}", e));
        }

        let path3 = format!("{}/three", file_path.to_str().unwrap());
        if let Err(e) = create_dir(&path3) {
            return Err(format!("couldn't create temp dir three: {}", e));
        }

        let glob_path = format!("[*]{}", file_path.to_str().unwrap());
        let mut p = Parser::new(glob_path.as_str());

        p.file()?;

        assert!(!p.int_rep.is_empty());
        assert_eq!(3, p.int_rep.len());
        assert_eq!(path1, p.int_rep.get("one").unwrap().to_string());
        assert_eq!(path2, p.int_rep.get("two").unwrap().to_string());
        assert_eq!(path3, p.int_rep.get("three").unwrap().to_string());

        Ok(())
    }
}
