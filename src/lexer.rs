use crate::token::{Token, TokenType, keyword_type};

pub fn lex_source(source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut line: i32 = 1;

    while i < len {
        let ch = chars[i];

        // Skip whitespace
        if ch == ' ' || ch == '\t' || ch == '\r' {
            i += 1;
            continue;
        }
        if ch == '\n' {
            line += 1;
            i += 1;
            continue;
        }

        // Comments
        if ch == '#' {
            // Hash comment - skip to end of line
            while i < len && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if ch == '/' && i + 1 < len {
            if chars[i + 1] == '/' {
                // Line comment
                i += 2;
                while i < len && chars[i] != '\n' {
                    i += 1;
                }
                continue;
            }
            if chars[i + 1] == '*' {
                // Block comment
                i += 2;
                let start_line = line;
                let mut found_end = false;
                while i + 1 < len {
                    if chars[i] == '\n' {
                        line += 1;
                    }
                    if chars[i] == '*' && chars[i + 1] == '/' {
                        i += 2;
                        found_end = true;
                        break;
                    }
                    i += 1;
                }
                if !found_end {
                    return Err(format!("[줄 {}] 오류: 종결되지 않은 블록 주석", start_line));
                }
                continue;
            }
        }

        // Strings
        if ch == '"' {
            let start_line = line;
            i += 1;
            let mut s = String::new();
            let mut terminated = false;
            while i < len {
                let c = chars[i];
                if c == '"' {
                    i += 1;
                    terminated = true;
                    break;
                }
                if c == '\n' {
                    line += 1;
                }
                if c == '\\' {
                    i += 1;
                    if i >= len {
                        return Err(format!("[줄 {}] 오류: 종결되지 않은 문자열", start_line));
                    }
                    let esc = chars[i];
                    match esc {
                        'n' => s.push('\n'),
                        'r' => s.push('\r'),
                        't' => s.push('\t'),
                        '\\' => s.push('\\'),
                        '"' => s.push('"'),
                        _ => {
                            return Err(format!(
                                "[줄 {}] 오류: 알 수 없는 이스케이프 시퀀스: \\{}",
                                line, esc
                            ));
                        }
                    }
                    i += 1;
                    continue;
                }
                s.push(c);
                i += 1;
            }
            if !terminated {
                return Err(format!("[줄 {}] 오류: 종결되지 않은 문자열", start_line));
            }
            tokens.push(Token::new(TokenType::StringLit, s, start_line));
            continue;
        }

        // Numbers
        if ch.is_ascii_digit() {
            let start = i;
            while i < len && chars[i].is_ascii_digit() {
                i += 1;
            }
            if i < len && chars[i] == '.' && i + 1 < len && chars[i + 1].is_ascii_digit() {
                i += 1; // skip '.'
                while i < len && chars[i].is_ascii_digit() {
                    i += 1;
                }
            }
            let lexeme: String = chars[start..i].iter().collect();
            let value: f64 = lexeme.parse().unwrap_or(0.0);
            tokens.push(Token::number(lexeme, value, line));
            continue;
        }

        // Identifiers and keywords (ASCII alpha, _, or UTF-8 multibyte)
        if ch.is_alphabetic() || ch == '_' || ch as u32 >= 0x80 {
            let start = i;
            while i < len {
                let c = chars[i];
                if c.is_alphanumeric() || c == '_' || c as u32 >= 0x80 {
                    i += 1;
                } else {
                    break;
                }
            }
            let word: String = chars[start..i].iter().collect();
            if let Some(kw) = keyword_type(&word) {
                tokens.push(Token::new(kw, word, line));
            } else {
                tokens.push(Token::new(TokenType::Identifier, word, line));
            }
            continue;
        }

        // Operators and punctuation
        match ch {
            '+' => { tokens.push(Token::new(TokenType::Plus, "+".into(), line)); i += 1; }
            '-' => { tokens.push(Token::new(TokenType::Minus, "-".into(), line)); i += 1; }
            '*' => { tokens.push(Token::new(TokenType::Star, "*".into(), line)); i += 1; }
            '/' => { tokens.push(Token::new(TokenType::Slash, "/".into(), line)); i += 1; }
            '%' => { tokens.push(Token::new(TokenType::Percent, "%".into(), line)); i += 1; }
            '(' => { tokens.push(Token::new(TokenType::LParen, "(".into(), line)); i += 1; }
            ')' => { tokens.push(Token::new(TokenType::RParen, ")".into(), line)); i += 1; }
            '{' => { tokens.push(Token::new(TokenType::LBrace, "{".into(), line)); i += 1; }
            '}' => { tokens.push(Token::new(TokenType::RBrace, "}".into(), line)); i += 1; }
            '[' => { tokens.push(Token::new(TokenType::LBracket, "[".into(), line)); i += 1; }
            ']' => { tokens.push(Token::new(TokenType::RBracket, "]".into(), line)); i += 1; }
            ',' => { tokens.push(Token::new(TokenType::Comma, ",".into(), line)); i += 1; }
            '.' => { tokens.push(Token::new(TokenType::Dot, ".".into(), line)); i += 1; }
            ':' => { tokens.push(Token::new(TokenType::Colon, ":".into(), line)); i += 1; }
            ';' => { tokens.push(Token::new(TokenType::Semicolon, ";".into(), line)); i += 1; }
            '!' => {
                if i + 1 < len && chars[i + 1] == '=' {
                    tokens.push(Token::new(TokenType::BangEqual, "!=".into(), line));
                    i += 2;
                } else {
                    tokens.push(Token::new(TokenType::Bang, "!".into(), line));
                    i += 1;
                }
            }
            '=' => {
                if i + 1 < len && chars[i + 1] == '=' {
                    tokens.push(Token::new(TokenType::EqualEqual, "==".into(), line));
                    i += 2;
                } else {
                    tokens.push(Token::new(TokenType::Equal, "=".into(), line));
                    i += 1;
                }
            }
            '<' => {
                if i + 1 < len && chars[i + 1] == '=' {
                    tokens.push(Token::new(TokenType::LessEqual, "<=".into(), line));
                    i += 2;
                } else {
                    tokens.push(Token::new(TokenType::Less, "<".into(), line));
                    i += 1;
                }
            }
            '>' => {
                if i + 1 < len && chars[i + 1] == '=' {
                    tokens.push(Token::new(TokenType::GreaterEqual, ">=".into(), line));
                    i += 2;
                } else {
                    tokens.push(Token::new(TokenType::Greater, ">".into(), line));
                    i += 1;
                }
            }
            '&' => {
                if i + 1 < len && chars[i + 1] == '&' {
                    tokens.push(Token::new(TokenType::AndAnd, "&&".into(), line));
                    i += 2;
                } else {
                    return Err(format!("[줄 {}] 오류: 예상치 못한 문자: '&'", line));
                }
            }
            '|' => {
                if i + 1 < len && chars[i + 1] == '|' {
                    tokens.push(Token::new(TokenType::OrOr, "||".into(), line));
                    i += 2;
                } else {
                    return Err(format!("[줄 {}] 오류: 예상치 못한 문자: '|'", line));
                }
            }
            _ => {
                return Err(format!("[줄 {}] 오류: 예상치 못한 문자: '{}'", line, ch));
            }
        }
    }

    tokens.push(Token::eof(line));
    Ok(tokens)
}
