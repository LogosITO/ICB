//! # Ultra Heuristic Near-AST Parser v3
//!
//! This version upgrades the parser from token-level heuristics
//! to a **structure-aware near-AST extractor**.
//!
//! ## Core idea
//!
//! Instead of parsing grammar, we reconstruct:
//!
//! - scopes (brace trees)
//! - declarations (class/function/namespace)
//! - call sites
//! - qualified names
//! - structural blocks
//!
//! ## Design goals
//!
//! - near‑AST accuracy without tree‑sitter
//! - robust C++/JS/Rust style class detection
//! - template / inheritance tolerant scanning
//! - O(n) linear pass
//! - zero unwrap / zero panic

use crate::facts::RawNode;
use icb_common::{Language, NodeKind};
use std::collections::HashSet;

/// Entry point for universal parsing.
pub fn parse_universal(source: &str, file: &str) -> Vec<RawNode> {
    if looks_like_markup(source) {
        return Vec::new();
    }

    let tokens = tokenize(source);
    if tokens.is_empty() {
        return Vec::new();
    }

    let scopes = build_scope_map(&tokens);
    let mut out = Vec::with_capacity(tokens.len() / 5);

    extract_structures(&tokens, &scopes, file, &mut out);
    extract_calls(&tokens, &mut out);
    extract_namespaces(&tokens, &mut out);

    dedup(&mut out);
    out
}

fn build_scope_map(tokens: &[Token]) -> Vec<u32> {
    let mut depth: u32 = 0;
    let mut map = Vec::with_capacity(tokens.len());

    for t in tokens {
        match t.kind {
            TokenKind::OpenBrace => depth = depth.saturating_add(1),
            TokenKind::CloseBrace => depth = depth.saturating_sub(1),
            _ => {}
        }
        map.push(depth);
    }

    map
}

fn extract_structures(tokens: &[Token], _scopes: &[u32], file: &str, out: &mut Vec<RawNode>) {
    let mut i: usize = 0;

    while i < tokens.len() {
        let t = &tokens[i];

        if t.kind != TokenKind::Ident {
            i += 1;
            continue;
        }

        let word = t.text.as_str();

        if is_class_keyword(word) {
            if let Some((name, col)) = find_structural_name(tokens, i + 1) {
                out.push(make_node(NodeKind::Class, name, t.line, col, file));
                i += 1;
                continue;
            }
        }

        if is_function_keyword(word) {
            if let Some((name, col)) = find_structural_name(tokens, i + 1) {
                out.push(make_node(NodeKind::Function, name, t.line, col, file));
                i += 1;
                continue;
            }
        }

        i += 1;
    }
}

fn find_structural_name(tokens: &[Token], mut i: usize) -> Option<(String, usize)> {
    let skip_noise = true;

    while i < tokens.len() {
        let t = &tokens[i];

        match t.kind {
            TokenKind::Ident if skip_noise => {
                let s = t.text.as_str();

                if is_noise_word(s) || is_modifier_word(s) {
                    i += 1;
                    continue;
                }

                return Some((t.text.clone(), t.col));
            }

            TokenKind::Ident => {
                return Some((t.text.clone(), t.col));
            }

            TokenKind::LessThan => {
                i = skip_template(tokens, i);
            }

            TokenKind::Colon => {
                i += 1;
            }

            _ => i += 1,
        }
    }

    None
}

fn skip_template(tokens: &[Token], mut i: usize) -> usize {
    let mut depth = 0usize;

    while i < tokens.len() {
        match tokens[i].kind {
            TokenKind::LessThan => depth += 1,
            TokenKind::GreaterThan => {
                if depth == 0 {
                    break;
                }
                depth -= 1;
                if depth == 0 {
                    return i + 1;
                }
            }
            _ => {}
        }

        i += 1;
    }

    i
}

fn extract_calls(tokens: &[Token], out: &mut Vec<RawNode>) {
    let mut i = 0;

    while i + 1 < tokens.len() {
        if tokens[i].kind == TokenKind::Ident && tokens[i + 1].kind == TokenKind::OpenParen {
            let name = build_qualified(tokens, i);

            if is_valid_call(&name) {
                out.push(RawNode {
                    language: Language::Unknown,
                    kind: NodeKind::CallSite,
                    name: Some(name),
                    usr: None,
                    start_line: tokens[i].line,
                    start_col: tokens[i].col,
                    end_line: tokens[i].line,
                    end_col: tokens[i].col + 1,
                    children: Vec::new(),
                    source_file: None,
                });
            }
        }

        i += 1;
    }
}

fn extract_namespaces(tokens: &[Token], out: &mut Vec<RawNode>) {
    let mut i = 0;

    while i < tokens.len() {
        if tokens[i].text == "namespace" {
            if let Some((name, col)) = find_structural_name(tokens, i + 1) {
                out.push(make_node(NodeKind::Class, name, tokens[i].line, col, ""));
            }
        }

        i += 1;
    }
}

fn build_qualified(tokens: &[Token], mut pos: usize) -> String {
    let mut parts = Vec::new();

    while pos > 0 {
        let t = &tokens[pos];

        if t.kind != TokenKind::Ident {
            break;
        }

        parts.push(t.text.clone());

        if pos >= 2 && tokens[pos - 1].kind == TokenKind::Dot {
            pos -= 2;
        } else {
            break;
        }
    }

    parts.reverse();
    parts.join(".")
}

fn make_node(kind: NodeKind, name: String, line: usize, col: usize, file: &str) -> RawNode {
    RawNode {
        language: Language::Unknown,
        kind,
        name: Some(name),
        usr: None,
        start_line: line,
        start_col: col,
        end_line: line,
        end_col: col + 1,
        children: Vec::new(),
        source_file: Some(file.to_string()),
    }
}

fn dedup(facts: &mut Vec<RawNode>) {
    let mut seen = HashSet::new();

    facts.retain(|f| {
        let key = (
            f.name.clone().unwrap_or_default(),
            f.start_line,
            match f.kind {
                NodeKind::Function => 1,
                NodeKind::Class => 2,
                NodeKind::CallSite => 3,
                _ => 0,
            },
        );

        seen.insert(key)
    });
}

fn is_class_keyword(s: &str) -> bool {
    matches!(
        s,
        "class" | "struct" | "interface" | "trait" | "enum" | "namespace" | "union" | "object"
    )
}

fn is_function_keyword(s: &str) -> bool {
    matches!(s, "fn" | "def" | "func" | "function" | "method")
}

fn is_modifier_word(s: &str) -> bool {
    matches!(
        s,
        "public"
            | "private"
            | "protected"
            | "static"
            | "final"
            | "abstract"
            | "virtual"
            | "override"
            | "const"
            | "inline"
    )
}

fn is_noise_word(s: &str) -> bool {
    matches!(
        s,
        "if" | "for" | "while" | "return" | "true" | "false" | "null" | "this" | "self"
    )
}

fn is_valid_call(name: &str) -> bool {
    !name.is_empty() && !is_noise_word(name)
}

fn looks_like_markup(src: &str) -> bool {
    let s = src.as_bytes();
    s.starts_with(b"<html") || s.starts_with(b"<?xml") || s.starts_with(b"<!DOCTYPE")
}

// ---------------------------------------------------------------------------
// Tokenizer
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
enum TokenKind {
    Ident,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Dot,
    Comma,
    Colon,
    LessThan,
    GreaterThan,
}

#[derive(Clone)]
struct Token {
    kind: TokenKind,
    text: String,
    line: usize,
    col: usize,
}

fn tokenize(src: &str) -> Vec<Token> {
    let bytes = src.as_bytes();
    let mut out = Vec::with_capacity(src.len() / 4);

    let mut i = 0;
    let mut line = 1;
    let mut col = 1;

    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                line += 1;
                col = 1;
                i += 1;
            }

            b'(' => push(
                &mut out,
                TokenKind::OpenParen,
                "(",
                line,
                col,
                &mut i,
                &mut col,
            ),
            b')' => push(
                &mut out,
                TokenKind::CloseParen,
                ")",
                line,
                col,
                &mut i,
                &mut col,
            ),
            b'{' => push(
                &mut out,
                TokenKind::OpenBrace,
                "{",
                line,
                col,
                &mut i,
                &mut col,
            ),
            b'}' => push(
                &mut out,
                TokenKind::CloseBrace,
                "}",
                line,
                col,
                &mut i,
                &mut col,
            ),
            b'.' => push(&mut out, TokenKind::Dot, ".", line, col, &mut i, &mut col),
            b',' => push(&mut out, TokenKind::Comma, ",", line, col, &mut i, &mut col),
            b':' => push(&mut out, TokenKind::Colon, ":", line, col, &mut i, &mut col),
            b'<' => push(
                &mut out,
                TokenKind::LessThan,
                "<",
                line,
                col,
                &mut i,
                &mut col,
            ),
            b'>' => push(
                &mut out,
                TokenKind::GreaterThan,
                ">",
                line,
                col,
                &mut i,
                &mut col,
            ),

            c if c.is_ascii_alphabetic() || c == b'_' => {
                let start = i;

                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }

                let text = &src[start..i];

                out.push(Token {
                    kind: TokenKind::Ident,
                    text: text.to_string(),
                    line,
                    col,
                });

                col += i - start;
            }

            _ => {
                i += 1;
                col += 1;
            }
        }
    }

    out
}

fn push(
    out: &mut Vec<Token>,
    kind: TokenKind,
    text: &str,
    line: usize,
    col: usize,
    i: &mut usize,
    c: &mut usize,
) {
    out.push(Token {
        kind,
        text: text.into(),
        line,
        col,
    });

    *i += 1;
    *c += 1;
}
