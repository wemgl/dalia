use std::borrow::Cow;
use std::fmt::Formatter;

const TOKEN_NAMES: [&str; 8] = [
    "n/a", "<EOF>", "FILE", "LINE", "LBRACK", "RBRACK", "ALIAS", "PATH",
];

pub(crate) const TOKEN_EOF: i32 = 1;
pub(crate) const TOKEN_FILE: i32 = 2;
pub(crate) const TOKEN_LINE: i32 = 3;
pub(crate) const TOKEN_LBRACK: i32 = 4;
pub(crate) const TOKEN_RBRACK: i32 = 5;
pub(crate) const TOKEN_ALIAS: i32 = 6;
pub(crate) const TOKEN_PATH: i32 = 7;

const EOF: char = !0 as char;

/// Token identifies a text and the kind of token it represents.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Token<'a> {
    /// The specific atom this token represents.
    pub(crate) kind: i32,
    /// The particular text associated with this token when it was parsed.
    pub(crate) text: Cow<'a, String>,
}

impl<'a> Token<'a> {
    pub(crate) fn new(kind: i32, text: Cow<'a, String>) -> Self {
        Self { kind, text }
    }
}

impl<'a> std::fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<'{}', {}>", self.text, TOKEN_NAMES[self.kind as usize])
    }
}

/// Cursor allows traversing through an input String character by character while lexing.
#[derive(Debug)]
pub(crate) struct Cursor {
    /// The input String being processed.
    input: String,
    /// A pointer to the current character.
    pointer: usize,
    /// The current character being processed.
    current_char: char,
}

impl Cursor {
    /// Constructs a new Cursor.
    fn new(input: &str, pointer: usize, c: char) -> Self {
        Self {
            input: input.to_string(),
            pointer,
            current_char: c,
        }
    }

    /// Consumes one character moving forward and detects "end of file".
    fn consume(&mut self) {
        self.pointer += 1;
        if self.pointer >= self.input.len() {
            self.current_char = EOF;
        } else if let Some(c) = self.input.chars().nth(self.pointer) {
            self.current_char = c
        }
    }

    /// Checks if the character parameter matches the current one stored by the cursor and
    /// returns the updated current character.
    fn matches(&mut self, c: char) -> Result<char, String> {
        if self.current_char == c {
            self.consume();
            return Ok(self.current_char);
        }
        return Err(format!("expecting {}, but found {}", c, self.current_char));
    }
}

/// Creates and identifies tokens using the underlying cursor.
#[derive(Debug)]
pub(crate) struct Lexer<'a> {
    pub(crate) cursor: Cursor,
    token_names: [&'a str; 8],
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(input: &str, pointer: usize, c: char) -> Self {
        Self {
            cursor: Cursor::new(input, pointer, c),
            token_names: TOKEN_NAMES,
        }
    }

    pub(crate) fn token_names(&self, i: usize) -> String {
        self.token_names[i].to_string()
    }

    fn is_not_end_line(&self) -> bool {
        match self.cursor.current_char {
            '\u{ff}' | '\0' | '\n' => false,
            _ => true,
        }
    }

    fn is_alphanumeric(&self) -> bool {
        self.cursor.current_char.is_ascii_alphanumeric()
    }

    pub(crate) fn next_token(&mut self) -> Result<Token<'a>, String> {
        while self.cursor.current_char != EOF {
            match self.cursor.current_char {
                ' ' | '\t' | '\n' | '\r' => {
                    self.whitespace();
                    continue;
                }
                '[' => {
                    self.cursor.consume();
                    return Ok(Token::new(TOKEN_LBRACK, Cow::Owned("[".into())));
                }
                ']' => {
                    self.cursor.consume();
                    return Ok(Token::new(TOKEN_RBRACK, Cow::Owned("]".into())));
                }
                _ => {
                    if self.is_alphanumeric() {
                        return self.alias();
                    } else if self.is_not_end_line() {
                        return self.path();
                    }
                    return Err(format!("invalid character {}", self.cursor.current_char));
                }
            }
        }

        Ok(Token::new(TOKEN_EOF, Cow::Owned("<EOF>".into())))
    }

    fn whitespace(&mut self) {
        while self.cursor.current_char.is_whitespace() {
            self.cursor.consume()
        }
    }

    fn alias(&mut self) -> Result<Token<'a>, String> {
        let mut a: String = String::new();
        while self.is_alphanumeric() {
            a.push(self.cursor.current_char);
            self.cursor.consume();
        }
        Ok(Token::new(TOKEN_ALIAS, Cow::Owned(a)))
    }

    fn path(&mut self) -> Result<Token<'a>, String> {
        let mut p = String::new();
        while self.is_not_end_line() {
            p.push(self.cursor.current_char);
            self.cursor.consume();
        }
        Ok(Token::new(TOKEN_PATH, Cow::Owned(p)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_display() {
        let tok = Token::new(TOKEN_EOF, Cow::Owned("<EOF>".into()));
        assert_eq!("<'<EOF>', <EOF>>", tok.to_string())
    }

    #[test]
    fn test_create_cursor() {
        let cur = Cursor::new("", 0, !0 as char);
        assert_eq!("".to_string(), cur.input);
        assert_eq!(0, cur.pointer);
        assert_eq!(!0 as char, cur.current_char);
    }

    #[test]
    fn test_cursor_consumes_characters() {
        let mut cur = Cursor::new("test", 0, 'e');
        cur.consume();
        assert_eq!("test".to_string(), cur.input);
        assert_eq!(1, cur.pointer);
        assert_eq!('e', cur.current_char);
    }

    #[test]
    fn test_cursor_consumes_end_of_file() {
        let mut cur = Cursor::new("test", 4, 't');
        cur.consume();
        assert_eq!("test".to_string(), cur.input);
        assert_eq!(5, cur.pointer);
        assert_eq!(!0 as char, cur.current_char);
    }

    #[test]
    fn test_cursor_matches_character_and_consumes() {
        let mut cur = Cursor::new("test", 0, 't');
        match cur.matches('t') {
            Ok(r) => assert_eq!('e', r),
            Err(e) => panic!(e),
        }
    }

    #[test]
    fn test_cursor_does_not_match_character_or_consume() {
        let mut cur = Cursor::new("test", 0, 't');
        match cur.matches('x') {
            Ok(_) => panic!("should not be okay"),
            Err(e) => assert_eq!("expecting x, but found t", e),
        }
    }

    #[test]
    fn test_lexer_gets_token_name() {
        let lexer = Lexer::new("test", 0, 't');
        let token_name = lexer.token_names(2);
        assert_eq!("FILE", token_name);
    }

    #[test]
    fn test_lexer_detects_line_feed_character() {
        let lexer = Lexer::new("\0", 0, '\0');
        assert!(
            !lexer.is_not_end_line(),
            "current character was not a LINE FEED"
        );
    }

    #[test]
    fn test_lexer_does_not_detect_non_line_feed_character() {
        let lexer = Lexer::new("test", 0, 't');
        assert!(lexer.is_not_end_line(), "current character was LINE FEED");
    }

    #[test]
    fn test_lexer_consumes_whitespace() {
        let mut lexer = Lexer::new("   test", 0, ' ');
        lexer.whitespace();
        assert_eq!('t', lexer.cursor.current_char);
    }

    #[test]
    fn test_lexer_can_check_is_alphanumeric() {
        let lexer = Lexer::new("test0123", 0, 't');
        assert!(lexer.is_alphanumeric());
    }

    #[test]
    fn test_lexer_can_check_is_alphanumeric_fails() {
        let lexer = Lexer::new("_", 0, '_');
        assert!(!lexer.is_alphanumeric());
    }

    #[test]
    fn test_lexer_creates_alias_token() {
        let mut lexer = Lexer::new("alias", 0, 'a');
        match lexer.alias() {
            Ok(token) => {
                assert_eq!(TOKEN_ALIAS, token.kind);
                assert_eq!("alias", token.text.as_str());
            }
            Err(_) => panic!("lexer panicked while creating alias"),
        }
    }

    #[test]
    fn test_lexer_creates_path_token() {
        let mut lexer = Lexer::new("/some/absolute/path", 0, '/');
        match lexer.path() {
            Ok(token) => {
                assert_eq!(TOKEN_PATH, token.kind);
                assert_eq!("/some/absolute/path", token.text.as_str());
            }
            Err(_) => panic!("lexer panicked while creating path"),
        }
    }

    #[test]
    fn test_lexer_next_token() {
        let input = r#"[test]/some/absolute/path
        /another/absolute/path
        "#;
        let mut lexer = Lexer::new(input, 0, '[');
        let mut tokens: Vec<Token> = Vec::new();
        while let Ok(t) = lexer.next_token() {
            if t.kind == TOKEN_EOF {
                break;
            }
            tokens.push(t);
        }
        assert_eq!(Token::new(TOKEN_LBRACK, Cow::Owned("[".into())), tokens[0]);
        assert_eq!(
            Token::new(TOKEN_ALIAS, Cow::Owned("test".into())),
            tokens[1]
        );
        assert_eq!(Token::new(TOKEN_RBRACK, Cow::Owned("]".into())), tokens[2]);
        assert_eq!(
            Token::new(TOKEN_PATH, Cow::Owned("/some/absolute/path".into())),
            tokens[3]
        );
        assert_eq!(
            Token::new(TOKEN_PATH, Cow::Owned("/another/absolute/path".into())),
            tokens[4]
        );
    }
}
