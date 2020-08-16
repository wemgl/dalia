use std::fmt::{Display, Formatter};
use std::result;

const TOKEN_NAMES: [&str; 9] = [
    "n/a", "<EOF>", "FILE", "LINE", "LBRACK", "RBRACK", "ALIAS", "PATH", "FSLASH",
];

const TOKEN_EOF: i32 = 1;
const TOKEN_FILE: i32 = 2;
const TOKEN_LINE: i32 = 3;
const TOKEN_LBRACK: i32 = 4;
const TOKEN_RBRACK: i32 = 5;
const TOKEN_ALIAS: i32 = 6;
const TOKEN_PATH: i32 = 7;
const TOKEN_FLASH: i32 = 8;

const EOF: char = !0 as char;

/// Token identifies a text and the kind of token it represents.
struct Token {
    /// The specific atom this token represents.
    kind: i32,
    /// The particular text associated with this token when it was parsed.
    text: &'static str,
}

impl Token {
    fn new(kind: i32, text: &'static str) -> Token {
        Token { kind, text }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<'{}', {}>", self.text, TOKEN_NAMES[self.kind as usize])
    }
}

/// Cursor allows traversing through an input String character by character while lexing.
struct Cursor {
    /// The input String being processed.
    input: String,
    /// A pointer to the current character.
    pointer: usize,
    /// The current character being processed.
    current_char: char,
}

impl Cursor {
    /// Constructs a new Cursor.
    fn new(input: &str, pointer: usize, c: char) -> Cursor {
        Cursor {
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
        } else {
            if let Some(c) = self.input.chars().nth(self.pointer) {
                self.current_char = c
            }
        }
    }

    fn matches(&mut self, c: char) -> result::Result<char, String> {
        if self.current_char == c {
            self.consume();
            return Ok(self.current_char);
        }
        return Err(format!("expecting {}, but found {}", c, self.current_char));
    }
}

struct Lexer<'a> {
    cursor: Cursor,
    token_names: [&'a str; 9],
}

impl<'a> Lexer<'a> {
    fn new(input: &str, pointer: usize, c: char) -> Lexer {
        Lexer {
            cursor: Cursor {
                input: input.to_string(),
                pointer,
                current_char: c,
            },
            token_names: TOKEN_NAMES,
        }
    }

    fn token_names(&self, i: usize) -> String {
        self.token_names[i].to_string()
    }

    fn is_not_nul(&self) -> bool {
        return self.cursor.current_char != '\0';
    }

    fn whitespace(&mut self) {
        while self.cursor.current_char == ' ' ||
            self.cursor.current_char == '\t' ||
            self.cursor.current_char == '\n' ||
            self.cursor.current_char == '\r' {
            self.cursor.consume()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_display() {
        let tok = Token::new(TOKEN_EOF, "<EOF>");
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
            Err(e) => panic!(e)
        }
    }

    #[test]
    fn test_cursor_does_not_match_character_or_consume() {
        let mut cur = Cursor::new("test", 0, 't');
        match cur.matches('x') {
            Ok(_) => panic!("should not be okay"),
            Err(e) => assert_eq!("expecting x, but found t", e)
        }
    }

    #[test]
    fn test_lexer_gets_token_name() {
        let lexer = Lexer::new("test", 0, 't');
        let token_name = lexer.token_names(2);
        assert_eq!("FILE", token_name);
    }

    #[test]
    fn test_lexer_detects_non_nul_characters() {
        let lexer = Lexer::new("test", 0, 't');
        assert!(lexer.is_not_nul(), "current character was NUL");
    }

    #[test]
    fn test_lexer_detects_nul_characters() {
        let lexer = Lexer::new("test", 0, '\0');
        assert!(!lexer.is_not_nul(), "current character was not NUL");
    }

    #[test]
    fn test_lexer_consumes_whitespace() {
        let mut lexer = Lexer::new("   test", 0, ' ');
        lexer.whitespace();
        assert_eq!('t', lexer.cursor.current_char);
    }
}
