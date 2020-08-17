mod parser {
    mod lexer;

    #[derive(Debug)]
    struct ParseError {
        msg: String,
    }

    impl std::fmt::Display for ParseError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "parse error: {}", self.msg)
        }
    }

    impl std::error::Error for ParseError {}

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

        fn consume(&mut self) -> Result<(), ParseError> {
            match self.input.next_token() {
                Ok(t) => {
                    self.lookahead = t;
                    Ok(())
                }
                Err(e) => Err(ParseError { msg: e }),
            }
        }

        fn matches(&mut self, k: i32) -> Result<(), ParseError> {
            if self.lookahead.kind == k {
                return match self.consume() {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                };
            }
            let msg = format!(
                "expecting {}; found {}",
                self.input.token_names(k as usize),
                self.lookahead
            );
            Err(ParseError { msg })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_create_parser() {
            let p = Parser::new("/some/absolute/path\n");
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
            let mut p = Parser::new("[alias]/some/absolute/path\n");
            let _ = p.consume();
            assert_eq!(lexer::Token::new(lexer::TOKEN_ALIAS, "alias"), p.lookahead);
        }

        #[test]
        fn test_parser_matches() {
            let mut p = Parser::new("[alias]/some/absolute/path\n");
            let _ = p.matches(lexer::TOKEN_LBRACK);
            assert_eq!(lexer::Token::new(lexer::TOKEN_ALIAS, "alias"), p.lookahead);
        }

        #[test]
        fn test_parser_does_not_match() {
            let mut p = Parser::new("[alias]/some/absolute/path\n");
            if let Err(pe) = p.matches(lexer::TOKEN_RBRACK) {
                assert_eq!("expecting RBRACK; found <'[', LBRACK>", pe.msg);
            }
        }
    }
}
