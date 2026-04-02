use lsp_types::*;

mod lexer;

use lexer::{Token, TokenKind, Lexer};

// ── Position utilities ──────────────────────────────────────────────

fn offset_to_position(text: &str, offset: usize) -> Position {
    let offset = offset.min(text.len());
    let mut line = 0u32;
    let mut col = 0u32;
    for (i, ch) in text.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    Position::new(line, col)
}

fn position_to_offset(text: &str, pos: Position) -> usize {
    let mut line = 0u32;
    let mut col = 0u32;
    for (i, ch) in text.char_indices() {
        if line == pos.line && col == pos.character {
            return i;
        }
        if ch == '\n' {
            if line == pos.line {
                return i;
            }
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    text.len()
}

// ── Binding analysis ────────────────────────────────────────────────

#[derive(Debug)]
struct BindingDef {
    name: String,
    kind: BindingKind,
    name_start: usize,
    name_end: usize,
}

#[derive(Debug, Clone, Copy)]
enum BindingKind {
    Def,
    Let,
    Val,
    Src,
}

impl BindingKind {
    fn keyword(&self) -> &'static str {
        match self {
            BindingKind::Def => "def",
            BindingKind::Let => "let",
            BindingKind::Val => "val",
            BindingKind::Src => "src",
        }
    }

    fn description(&self) -> &'static str {
        match self {
            BindingKind::Def => "Execute later, collect later (function definition)",
            BindingKind::Let => "Execute now, collect now (eager binding)",
            BindingKind::Val => "Execute now, collect now (alias for let)",
            BindingKind::Src => "Execute now, collect later (lazy collection)",
        }
    }
}

fn find_bindings(text: &str) -> Vec<BindingDef> {
    let tokens: Vec<Token> = Lexer::new(text).collect();
    let mut bindings = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        let kind = match tokens[i].kind {
            TokenKind::Def => Some(BindingKind::Def),
            TokenKind::Let => Some(BindingKind::Let),
            TokenKind::Val => Some(BindingKind::Val),
            TokenKind::Src => Some(BindingKind::Src),
            _ => None,
        };

        if let Some(kind) = kind {
            // Next non-whitespace token should be the name
            if i + 1 < tokens.len() && tokens[i + 1].kind == TokenKind::Ident {
                let name_tok = &tokens[i + 1];
                bindings.push(BindingDef {
                    name: name_tok.text.to_string(),
                    kind,
                    name_start: name_tok.start,
                    name_end: name_tok.end,
                });
                i += 2;
                continue;
            }
        }
        i += 1;
    }

    bindings
}

// ── Diagnostics (lightweight parse checking) ────────────────────────

pub fn diagnose(text: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let tokens: Vec<Token> = Lexer::new(text).collect();

    // Check for unmatched brackets
    check_brackets(&tokens, text, &mut diagnostics);

    // Check for basic structure errors
    check_structure(&tokens, text, &mut diagnostics);

    diagnostics
}

fn check_brackets(tokens: &[Token], text: &str, diagnostics: &mut Vec<Diagnostic>) {
    let mut stack: Vec<(TokenKind, usize)> = Vec::new();

    for tok in tokens {
        match tok.kind {
            TokenKind::LBrace | TokenKind::LParen | TokenKind::LBracket => {
                stack.push((tok.kind, tok.start));
            }
            TokenKind::RBrace => {
                if let Some((open, _)) = stack.last() {
                    if *open == TokenKind::LBrace {
                        stack.pop();
                    } else {
                        diagnostics.push(Diagnostic {
                            range: Range::new(
                                offset_to_position(text, tok.start),
                                offset_to_position(text, tok.end),
                            ),
                            severity: Some(DiagnosticSeverity::ERROR),
                            source: Some("dust-lsp".to_string()),
                            message: format!("Mismatched '}}', expected closing for '{}'", match open {
                                TokenKind::LParen => "(",
                                TokenKind::LBracket => "[",
                                _ => "{",
                            }),
                            ..Default::default()
                        });
                    }
                } else {
                    diagnostics.push(Diagnostic {
                        range: Range::new(
                            offset_to_position(text, tok.start),
                            offset_to_position(text, tok.end),
                        ),
                        severity: Some(DiagnosticSeverity::ERROR),
                        source: Some("dust-lsp".to_string()),
                        message: "Unexpected '}'".to_string(),
                        ..Default::default()
                    });
                }
            }
            TokenKind::RParen => {
                if let Some((open, _)) = stack.last() {
                    if *open == TokenKind::LParen {
                        stack.pop();
                    } else {
                        diagnostics.push(Diagnostic {
                            range: Range::new(
                                offset_to_position(text, tok.start),
                                offset_to_position(text, tok.end),
                            ),
                            severity: Some(DiagnosticSeverity::ERROR),
                            source: Some("dust-lsp".to_string()),
                            message: "Mismatched ')'".to_string(),
                            ..Default::default()
                        });
                    }
                } else {
                    diagnostics.push(Diagnostic {
                        range: Range::new(
                            offset_to_position(text, tok.start),
                            offset_to_position(text, tok.end),
                        ),
                        severity: Some(DiagnosticSeverity::ERROR),
                        source: Some("dust-lsp".to_string()),
                        message: "Unexpected ')'".to_string(),
                        ..Default::default()
                    });
                }
            }
            TokenKind::RBracket => {
                if let Some((open, _)) = stack.last() {
                    if *open == TokenKind::LBracket {
                        stack.pop();
                    } else {
                        diagnostics.push(Diagnostic {
                            range: Range::new(
                                offset_to_position(text, tok.start),
                                offset_to_position(text, tok.end),
                            ),
                            severity: Some(DiagnosticSeverity::ERROR),
                            source: Some("dust-lsp".to_string()),
                            message: "Mismatched ']'".to_string(),
                            ..Default::default()
                        });
                    }
                } else {
                    diagnostics.push(Diagnostic {
                        range: Range::new(
                            offset_to_position(text, tok.start),
                            offset_to_position(text, tok.end),
                        ),
                        severity: Some(DiagnosticSeverity::ERROR),
                        source: Some("dust-lsp".to_string()),
                        message: "Unexpected ']'".to_string(),
                        ..Default::default()
                    });
                }
            }
            _ => {}
        }
    }

    // Report unclosed brackets
    for (kind, start) in stack {
        let bracket = match kind {
            TokenKind::LBrace => "{",
            TokenKind::LParen => "(",
            TokenKind::LBracket => "[",
            _ => "?",
        };
        diagnostics.push(Diagnostic {
            range: Range::new(
                offset_to_position(text, start),
                offset_to_position(text, start + 1),
            ),
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some("dust-lsp".to_string()),
            message: format!("Unclosed '{}'", bracket),
            ..Default::default()
        });
    }
}

fn check_structure(tokens: &[Token], text: &str, diagnostics: &mut Vec<Diagnostic>) {
    let mut i = 0;
    while i < tokens.len() {
        let tok = &tokens[i];

        // Check: `def name =` / `let name =` / `val name =` / `src name =` patterns
        match tok.kind {
            TokenKind::Def | TokenKind::Let | TokenKind::Val | TokenKind::Src => {
                let kw = tok.text;
                // Must be followed by an identifier
                if i + 1 >= tokens.len() {
                    diagnostics.push(Diagnostic {
                        range: Range::new(
                            offset_to_position(text, tok.start),
                            offset_to_position(text, tok.end),
                        ),
                        severity: Some(DiagnosticSeverity::ERROR),
                        source: Some("dust-lsp".to_string()),
                        message: format!("'{}' must be followed by a name", kw),
                        ..Default::default()
                    });
                } else if tokens[i + 1].kind != TokenKind::Ident {
                    diagnostics.push(Diagnostic {
                        range: Range::new(
                            offset_to_position(text, tokens[i + 1].start),
                            offset_to_position(text, tokens[i + 1].end),
                        ),
                        severity: Some(DiagnosticSeverity::ERROR),
                        source: Some("dust-lsp".to_string()),
                        message: format!("Expected identifier after '{}'", kw),
                        ..Default::default()
                    });
                } else if i + 2 < tokens.len() && tokens[i + 2].kind != TokenKind::Eq {
                    // For def, the name can be followed by params before =
                    if tok.kind != TokenKind::Def {
                        diagnostics.push(Diagnostic {
                            range: Range::new(
                                offset_to_position(text, tokens[i + 2].start),
                                offset_to_position(text, tokens[i + 2].end),
                            ),
                            severity: Some(DiagnosticSeverity::WARNING),
                            source: Some("dust-lsp".to_string()),
                            message: format!("Expected '=' after '{}' binding name", kw),
                            ..Default::default()
                        });
                    }
                }
            }
            _ => {}
        }

        i += 1;
    }
}

// ── Hover ───────────────────────────────────────────────────────────

pub fn hover(text: &str, pos: Position) -> Option<Hover> {
    let offset = position_to_offset(text, pos);
    let token = token_at_offset(text, offset)?;

    // Keyword hover
    let keyword_info = match token.kind {
        TokenKind::Def => Some("**def** — Define a binding. Execute later, collect later (function definition)."),
        TokenKind::Let => Some("**let** — Eager binding. Execute now, collect now."),
        TokenKind::Val => Some("**val** — Alias for `let`. Execute now, collect now."),
        TokenKind::Src => Some("**src** — Source binding. Execute now, collect later."),
        TokenKind::Include => Some("**include** — Include and execute another Dust file."),
        TokenKind::IncludeStr => Some("**include_str** — Include the string content of a file into a binding."),
        _ => None,
    };

    if let Some(info) = keyword_info {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: info.to_string(),
            }),
            range: Some(Range::new(
                offset_to_position(text, token.start),
                offset_to_position(text, token.end),
            )),
        });
    }

    // Variable hover ($name)
    if token.kind == TokenKind::Variable {
        let var_name = &token.text[1..]; // strip $
        let bindings = find_bindings(text);
        for b in &bindings {
            if b.name == var_name {
                let info = format!(
                    "**${}** — `{}` binding\n\n{}",
                    var_name,
                    b.kind.keyword(),
                    b.kind.description(),
                );
                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: info,
                    }),
                    range: Some(Range::new(
                        offset_to_position(text, token.start),
                        offset_to_position(text, token.end),
                    )),
                });
            }
        }
    }

    // System command hover (!name)
    if token.kind == TokenKind::SystemCmd {
        let cmd_name = &token.text[1..]; // strip !
        let info = format!("**!{}** — System command invocation", cmd_name);
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: info,
            }),
            range: Some(Range::new(
                offset_to_position(text, token.start),
                offset_to_position(text, token.end),
            )),
        });
    }

    // Identifier hover (could be a binding reference)
    if token.kind == TokenKind::Ident {
        let bindings = find_bindings(text);
        for b in &bindings {
            if b.name == token.text {
                let info = format!(
                    "**{}** — `{}` binding\n\n{}",
                    b.name,
                    b.kind.keyword(),
                    b.kind.description(),
                );
                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: info,
                    }),
                    range: Some(Range::new(
                        offset_to_position(text, token.start),
                        offset_to_position(text, token.end),
                    )),
                });
            }
        }
    }

    None
}

// ── Go to definition ────────────────────────────────────────────────

pub fn goto_definition(text: &str, uri: &Url, pos: Position) -> Option<Location> {
    let offset = position_to_offset(text, pos);
    let token = token_at_offset(text, offset)?;

    let name = match token.kind {
        TokenKind::Variable => &token.text[1..], // strip $
        TokenKind::Ident => token.text,
        _ => return None,
    };

    let bindings = find_bindings(text);
    for b in &bindings {
        if b.name == name {
            return Some(Location {
                uri: uri.clone(),
                range: Range::new(
                    offset_to_position(text, b.name_start),
                    offset_to_position(text, b.name_end),
                ),
            });
        }
    }

    None
}

// ── Token lookup ────────────────────────────────────────────────────

fn token_at_offset<'a>(text: &'a str, offset: usize) -> Option<Token<'a>> {
    let lexer = Lexer::new(text);
    for tok in lexer {
        if tok.start <= offset && offset < tok.end {
            return Some(tok);
        }
    }
    None
}
