/// Lightweight Dust lexer for the LSP server.
///
/// Token kinds mirror the original lex crate's token types.
/// This lexer is intentionally standalone so the LSP crate can build
/// without pulling in the full dusk workspace.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // Keywords
    Def,
    Let,
    Val,
    Src,
    Include,
    IncludeStr,
    New,
    If,
    ForEach,
    In,

    // Identifiers & literals
    Ident,
    Natural,
    String,
    RawString,

    // Paths
    AbsPath,
    RelPath,
    HomePath,

    // Options
    LongOpt,
    ShortOpt,

    // Variables & references
    Variable,   // $name
    Slice,      // $name[...]
    SystemCmd,  // !name
    Deref,      // *name

    // Operators & punctuation
    Eq,
    Semi,
    Comma,
    At,
    Lt,
    Gt,
    Bang,
    Dollar,
    Star,
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,
    RBracket,

    // Comments
    Comment,

    // Unknown / error token
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub text: &'a str,
    pub start: usize,
    pub end: usize,
}

pub struct Lexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn peek(&self) -> Option<u8> {
        self.input.as_bytes().get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<u8> {
        self.input.as_bytes().get(self.pos + offset).copied()
    }

    fn advance(&mut self, n: usize) {
        self.pos += n;
    }

    fn skip_whitespace(&mut self) {
        while let Some(b) = self.peek() {
            if b.is_ascii_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn eat_comment(&mut self) -> Option<Token<'a>> {
        if self.peek() != Some(b'#') {
            return None;
        }
        let start = self.pos;
        while let Some(b) = self.peek() {
            if b == b'\n' {
                break;
            }
            self.pos += 1;
        }
        Some(Token {
            kind: TokenKind::Comment,
            text: &self.input[start..self.pos],
            start,
            end: self.pos,
        })
    }

    fn eat_raw_string(&mut self) -> Option<Token<'a>> {
        // Supports: r"...", r#"..."#, r##"..."##, r###"..."###, etc.
        if self.peek() != Some(b'r') {
            return None;
        }

        let start = self.pos;
        let mut off = 1; // skip 'r'

        // Count opening hashes
        let mut hash_count = 0usize;
        while self.peek_at(off) == Some(b'#') {
            hash_count += 1;
            off += 1;
        }

        // Must have opening quote after r[#*]
        if self.peek_at(off) != Some(b'"') {
            return None;
        }
        off += 1; // skip opening "

        self.advance(off);

        // Now find closing: "[#]{hash_count}
        loop {
            if self.pos >= self.input.len() {
                // Unterminated raw string - consume rest
                return Some(Token {
                    kind: TokenKind::RawString,
                    text: &self.input[start..self.pos],
                    start,
                    end: self.pos,
                });
            }

            if self.peek() == Some(b'"') {
                // Check if followed by exactly hash_count '#'s
                let mut matched = true;
                for i in 1..=hash_count {
                    if self.peek_at(i) != Some(b'#') {
                        matched = false;
                        break;
                    }
                }
                if matched {
                    self.advance(1 + hash_count); // closing " + hashes
                    return Some(Token {
                        kind: TokenKind::RawString,
                        text: &self.input[start..self.pos],
                        start,
                        end: self.pos,
                    });
                }
            }

            self.advance(1);
        }
    }

    fn eat_string(&mut self, quote: u8) -> Option<Token<'a>> {
        if self.peek() != Some(quote) {
            return None;
        }
        let start = self.pos;
        self.advance(1);
        while let Some(b) = self.peek() {
            if b == b'\\' {
                self.advance(2);
                continue;
            }
            if b == quote {
                self.advance(1);
                return Some(Token {
                    kind: TokenKind::String,
                    text: &self.input[start..self.pos],
                    start,
                    end: self.pos,
                });
            }
            self.advance(1);
        }
        // Unterminated string
        Some(Token {
            kind: TokenKind::String,
            text: &self.input[start..self.pos],
            start,
            end: self.pos,
        })
    }

    fn eat_variable_or_slice(&mut self) -> Option<Token<'a>> {
        if self.peek() != Some(b'$') {
            return None;
        }
        let start = self.pos;
        self.advance(1); // skip $

        // Must be followed by an ident start
        if !self.peek().map(is_ident_init).unwrap_or(false) {
            return Some(Token {
                kind: TokenKind::Dollar,
                text: &self.input[start..self.pos],
                start,
                end: self.pos,
            });
        }

        // Eat identifier part
        while self.peek().map(is_ident_rest).unwrap_or(false) {
            self.advance(1);
        }

        // Check for slice: $name[...]
        if self.peek() == Some(b'[') {
            let _name_end = self.pos;
            self.advance(1); // skip [
            let mut depth = 1;
            while depth > 0 {
                match self.peek() {
                    Some(b'[') => { depth += 1; self.advance(1); }
                    Some(b']') => { depth -= 1; self.advance(1); }
                    Some(_) => { self.advance(1); }
                    None => break,
                }
            }
            return Some(Token {
                kind: TokenKind::Slice,
                text: &self.input[start..self.pos],
                start,
                end: self.pos,
            });
        }

        Some(Token {
            kind: TokenKind::Variable,
            text: &self.input[start..self.pos],
            start,
            end: self.pos,
        })
    }

    fn eat_system_cmd(&mut self) -> Option<Token<'a>> {
        if self.peek() != Some(b'!') {
            return None;
        }
        // Must be followed by ident
        if !self.peek_at(1).map(is_ident_init).unwrap_or(false) {
            return None;
        }
        let start = self.pos;
        self.advance(1); // skip !
        while self.peek().map(is_ident_rest).unwrap_or(false) {
            self.advance(1);
        }
        Some(Token {
            kind: TokenKind::SystemCmd,
            text: &self.input[start..self.pos],
            start,
            end: self.pos,
        })
    }

    fn eat_deref(&mut self) -> Option<Token<'a>> {
        if self.peek() != Some(b'*') {
            return None;
        }
        if !self.peek_at(1).map(is_ident_init).unwrap_or(false) {
            return None;
        }
        let start = self.pos;
        self.advance(1); // skip *
        while self.peek().map(is_ident_rest).unwrap_or(false) {
            self.advance(1);
        }
        Some(Token {
            kind: TokenKind::Deref,
            text: &self.input[start..self.pos],
            start,
            end: self.pos,
        })
    }

    fn eat_number(&mut self) -> Option<Token<'a>> {
        if !self.peek().map(|b| b.is_ascii_digit()).unwrap_or(false) {
            return None;
        }
        let start = self.pos;
        while self.peek().map(|b| b.is_ascii_digit()).unwrap_or(false) {
            self.advance(1);
        }
        Some(Token {
            kind: TokenKind::Natural,
            text: &self.input[start..self.pos],
            start,
            end: self.pos,
        })
    }

    fn eat_path(&mut self) -> Option<Token<'a>> {
        let start = self.pos;
        let b = self.peek()?;

        // Absolute path: /...
        if b == b'/' && self.peek_at(1).map(is_ident_rest).unwrap_or(false) {
            self.advance(1);
            while self.peek().map(is_ident_rest).unwrap_or(false) {
                self.advance(1);
            }
            return Some(Token {
                kind: TokenKind::AbsPath,
                text: &self.input[start..self.pos],
                start,
                end: self.pos,
            });
        }

        // Relative path: ./ or ../
        if b == b'.' {
            let next = self.peek_at(1);
            if next == Some(b'/') || (next == Some(b'.') && self.peek_at(2) == Some(b'/')) {
                while self.peek().map(|c| is_ident_rest(c) || c == b'.').unwrap_or(false) {
                    self.advance(1);
                }
                return Some(Token {
                    kind: TokenKind::RelPath,
                    text: &self.input[start..self.pos],
                    start,
                    end: self.pos,
                });
            }
        }

        // Home path: ~/...
        if b == b'~' && self.peek_at(1) == Some(b'/') {
            self.advance(1);
            while self.peek().map(is_ident_rest).unwrap_or(false) {
                self.advance(1);
            }
            return Some(Token {
                kind: TokenKind::HomePath,
                text: &self.input[start..self.pos],
                start,
                end: self.pos,
            });
        }

        None
    }

    fn eat_long_opt(&mut self) -> Option<Token<'a>> {
        if self.peek() == Some(b'-') && self.peek_at(1) == Some(b'-')
            && self.peek_at(2).map(is_ident_rest).unwrap_or(false)
        {
            let start = self.pos;
            self.advance(2);
            while self.peek().map(is_ident_rest).unwrap_or(false) {
                self.advance(1);
            }
            return Some(Token {
                kind: TokenKind::LongOpt,
                text: &self.input[start..self.pos],
                start,
                end: self.pos,
            });
        }
        None
    }

    fn eat_short_opt(&mut self) -> Option<Token<'a>> {
        if self.peek() == Some(b'-')
            && self.peek_at(1).map(|b| b.is_ascii_alphanumeric()).unwrap_or(false)
        {
            let start = self.pos;
            self.advance(1);
            while self.peek().map(is_ident_rest).unwrap_or(false) {
                self.advance(1);
            }
            return Some(Token {
                kind: TokenKind::ShortOpt,
                text: &self.input[start..self.pos],
                start,
                end: self.pos,
            });
        }
        None
    }

    fn eat_ident_or_keyword(&mut self) -> Option<Token<'a>> {
        if !self.peek().map(is_ident_init).unwrap_or(false) {
            return None;
        }
        let start = self.pos;
        while self.peek().map(is_ident_rest).unwrap_or(false) {
            self.advance(1);
        }
        let text = &self.input[start..self.pos];
        let kind = match text {
            "def" => TokenKind::Def,
            "let" => TokenKind::Let,
            "val" => TokenKind::Val,
            "src" => TokenKind::Src,
            "include_str" => TokenKind::IncludeStr,
            "include" => TokenKind::Include,
            "new" => TokenKind::New,
            "if" => TokenKind::If,
            "for_each" => TokenKind::ForEach,
            "in" => TokenKind::In,
            _ => TokenKind::Ident,
        };
        Some(Token {
            kind,
            text,
            start,
            end: self.pos,
        })
    }

    fn eat_punct(&mut self) -> Option<Token<'a>> {
        let start = self.pos;
        let kind = match self.peek()? {
            b'=' => TokenKind::Eq,
            b';' => TokenKind::Semi,
            b',' => TokenKind::Comma,
            b'@' => TokenKind::At,
            b'<' => TokenKind::Lt,
            b'>' => TokenKind::Gt,
            b'!' => TokenKind::Bang,
            b'*' => TokenKind::Star,
            b'{' => TokenKind::LBrace,
            b'}' => TokenKind::RBrace,
            b'(' => TokenKind::LParen,
            b')' => TokenKind::RParen,
            b'[' => TokenKind::LBracket,
            b']' => TokenKind::RBracket,
            _ => return None,
        };
        self.advance(1);
        Some(Token {
            kind,
            text: &self.input[start..self.pos],
            start,
            end: self.pos,
        })
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Token<'a>> {
        loop {
            self.skip_whitespace();

            if self.pos >= self.input.len() {
                return None;
            }

            // Comments — skip them but still return them as tokens
            // (for bracket matching we need to skip them)
            if let Some(tok) = self.eat_comment() {
                return Some(tok);
            }

            // Raw strings first (r#"..."#)
            if let Some(tok) = self.eat_raw_string() {
                return Some(tok);
            }

            // Double-quoted strings
            if let Some(tok) = self.eat_string(b'"') {
                return Some(tok);
            }

            // Single-quoted strings
            if let Some(tok) = self.eat_string(b'\'') {
                return Some(tok);
            }

            // Variables ($name) and slices ($name[...])
            if let Some(tok) = self.eat_variable_or_slice() {
                return Some(tok);
            }

            // System commands (!name)
            if let Some(tok) = self.eat_system_cmd() {
                return Some(tok);
            }

            // Dereferences (*name)
            if let Some(tok) = self.eat_deref() {
                return Some(tok);
            }

            // Numbers
            if let Some(tok) = self.eat_number() {
                return Some(tok);
            }

            // Long options (--name)
            if let Some(tok) = self.eat_long_opt() {
                return Some(tok);
            }

            // Short options (-n)
            if let Some(tok) = self.eat_short_opt() {
                return Some(tok);
            }

            // Paths (/, ./, ../, ~/)
            if let Some(tok) = self.eat_path() {
                return Some(tok);
            }

            // Identifiers / keywords
            if let Some(tok) = self.eat_ident_or_keyword() {
                return Some(tok);
            }

            // Punctuation
            if let Some(tok) = self.eat_punct() {
                return Some(tok);
            }

            // Unknown character — skip it
            let start = self.pos;
            self.advance(1);
            return Some(Token {
                kind: TokenKind::Unknown,
                text: &self.input[start..self.pos],
                start,
                end: self.pos,
            });
        }
    }
}

fn is_ident_init(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn is_ident_rest(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b':' || b == b'.'
        || b == b',' || b == b'/' || b == b'+' || b == b'-' || b == b'='
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_lex() {
        let input = r#"def build = !cargo build --bin xs-compile $args;"#;
        let tokens: Vec<Token> = Lexer::new(input).collect();
        assert_eq!(tokens[0].kind, TokenKind::Def);
        assert_eq!(tokens[1].kind, TokenKind::Ident);
        assert_eq!(tokens[1].text, "build");
        assert_eq!(tokens[2].kind, TokenKind::Eq);
        assert_eq!(tokens[3].kind, TokenKind::SystemCmd);
        assert_eq!(tokens[3].text, "!cargo");
    }

    #[test]
    fn test_variable_and_slice() {
        let input = "$args[0;]";
        let tokens: Vec<Token> = Lexer::new(input).collect();
        assert_eq!(tokens[0].kind, TokenKind::Slice);
        assert_eq!(tokens[0].text, "$args[0;]");
    }

    #[test]
    fn test_strings() {
        let input = r#""hello world" 'raw'"#;
        let tokens: Vec<Token> = Lexer::new(input).collect();
        assert_eq!(tokens[0].kind, TokenKind::String);
        assert_eq!(tokens[1].kind, TokenKind::String);
    }

    #[test]
    fn test_comment() {
        let input = "# this is a comment\ndef x = 1;";
        let tokens: Vec<Token> = Lexer::new(input).collect();
        assert_eq!(tokens[0].kind, TokenKind::Comment);
        assert_eq!(tokens[1].kind, TokenKind::Def);
    }

    #[test]
    fn test_paths() {
        let input = "./target/debug/xs-compile /usr/bin ../parent ~/home";
        let tokens: Vec<Token> = Lexer::new(input).collect();
        assert_eq!(tokens[0].kind, TokenKind::RelPath);
        assert_eq!(tokens[1].kind, TokenKind::AbsPath);
        assert_eq!(tokens[2].kind, TokenKind::RelPath);
        assert_eq!(tokens[3].kind, TokenKind::HomePath);
    }
}
