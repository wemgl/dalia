mod lexer;

// const CONFIG: std::collections::HashMap<String, String> = std::collections::HashMap::new();

#[derive(Debug)]
struct Parser<'a> {
    /// The lexer responsible for returning tokenized input.
    input: lexer::Lexer<'a>,
    /// The current lookahead token used used by this parser.
    lookahead: lexer::Token,
}

impl<'a> Parser<'a> {
    fn new(input: &str) -> Self {
        if input.trim().len() == 0 {
            panic!("no input provided")
        }
        let c = input.chars().nth(0).unwrap();
        let mut lex = lexer::Lexer::new(input, 0, c);
        match lex.next_token() {
            Ok(tok) => {
                return Self {
                    input: lex,
                    lookahead: tok,
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
            if let Err(e) = self.consume() {
                return Err(e);
            }
            return Ok(());
        }
        let msg = format!(
            "expecting {}; found {}",
            self.input.token_names(k as usize),
            self.lookahead
        );
        Err(msg)
    }

    fn file(&mut self) -> Result<(), String> {
        loop {
            if let Err(e) = self.line() {
                return Err(e);
            }
            if self.lookahead.kind == lexer::TOKEN_EOF {
                return match self.matches(lexer::TOKEN_EOF) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                };
            }
        }
    }

    pub fn line(&mut self) -> Result<(), String> {
        if self.lookahead.kind == lexer::TOKEN_LBRACK {
            if let Err(e) = self.matches(lexer::TOKEN_LBRACK) {
                return Err(e);
            }
            if let Err(e) = self.alias() {
                return Err(e);
            }
            if let Err(e) = self.matches(lexer::TOKEN_RBRACK) {
                return Err(e);
            }
        }
        if let Err(e) = self.path() {
            return Err(e);
        }
        Ok(())
    }

    fn alias(&mut self) -> Result<(), String> {
        if let Err(e) = self.matches(lexer::TOKEN_ALIAS) {
            return Err(e);
        }
        Ok(())
    }

    fn path(&mut self) -> Result<(), String> {
        if self.lookahead.kind == lexer::TOKEN_PATH {
            if let Err(e) = self.matches(lexer::TOKEN_PATH) {
                return Err(e);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_parser() {
        let p = Parser::new("/some/absolute/path");
        assert_eq!(
            lexer::Token::new(lexer::TOKEN_PATH, "/some/absolute/path"),
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
        assert_eq!(lexer::Token::new(lexer::TOKEN_ALIAS, "alias"), p.lookahead);
    }

    #[test]
    fn test_parser_matches() {
        let mut p = Parser::new("[alias]/some/absolute/path");
        let _ = p.matches(lexer::TOKEN_LBRACK);
        assert_eq!(lexer::Token::new(lexer::TOKEN_ALIAS, "alias"), p.lookahead);
    }

    #[test]
    fn test_parser_does_not_match() {
        let mut p = Parser::new("[alias]/some/absolute/path");
        if let Err(e) = p.matches(lexer::TOKEN_RBRACK) {
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
}
