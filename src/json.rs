use crate::error::CokacError;
use crate::value::{Value, ArrayValue, ObjectValue};
use std::cell::RefCell;
use std::rc::Rc;

struct JsonParser<'a> {
    source: &'a [u8],
    pos: usize,
    line: i32,
}

impl<'a> JsonParser<'a> {
    fn new(source: &'a str, line: i32) -> Self {
        JsonParser {
            source: source.as_bytes(),
            pos: 0,
            line,
        }
    }

    fn fail(&self, msg: &str) -> CokacError {
        CokacError::new(format!("JSON 파싱 오류: {}", msg), self.line)
    }

    fn peek(&self) -> Option<u8> {
        if self.pos < self.source.len() {
            Some(self.source[self.pos])
        } else {
            None
        }
    }

    fn advance(&mut self) -> u8 {
        let ch = self.source[self.pos];
        self.pos += 1;
        ch
    }

    fn skip_ws(&mut self) {
        while self.pos < self.source.len() {
            match self.source[self.pos] {
                b' ' | b'\t' | b'\r' | b'\n' => { self.pos += 1; }
                _ => break,
            }
        }
    }

    fn parse_value(&mut self, depth: usize) -> Result<Value, CokacError> {
        if depth > 256 {
            return Err(self.fail("JSON 중첩 깊이가 너무 깊습니다."));
        }
        self.skip_ws();
        match self.peek() {
            None => Err(self.fail("예상치 못한 JSON 입력 끝")),
            Some(b'"') => self.parse_string().map(Value::String),
            Some(b'{') => self.parse_object(depth),
            Some(b'[') => self.parse_array(depth),
            Some(b'-') | Some(b'0'..=b'9') => self.parse_number(),
            Some(b't') => {
                self.expect_literal(b"true")?;
                Ok(Value::Bool(true))
            }
            Some(b'f') => {
                self.expect_literal(b"false")?;
                Ok(Value::Bool(false))
            }
            Some(b'n') => {
                self.expect_literal(b"null")?;
                Ok(Value::Nil)
            }
            Some(c) => Err(self.fail(&format!("알 수 없는 JSON 값: '{}'", c as char))),
        }
    }

    fn expect_literal(&mut self, expected: &[u8]) -> Result<(), CokacError> {
        for &b in expected {
            match self.peek() {
                Some(c) if c == b => { self.advance(); }
                _ => return Err(self.fail(&format!("'{}' 기대됨", std::str::from_utf8(expected).unwrap_or("?")))),
            }
        }
        Ok(())
    }

    fn parse_string(&mut self) -> Result<String, CokacError> {
        self.advance(); // skip opening "
        let mut s = String::new();
        loop {
            if self.pos >= self.source.len() {
                return Err(self.fail("종결되지 않은 JSON 문자열"));
            }
            let ch = self.advance();
            if ch == b'"' {
                return Ok(s);
            }
            if ch < 0x20 {
                return Err(self.fail("JSON 문자열에 제어 문자가 포함됨"));
            }
            if ch == b'\\' {
                if self.pos >= self.source.len() {
                    return Err(self.fail("종결되지 않은 JSON 이스케이프"));
                }
                let esc = self.advance();
                match esc {
                    b'"' => s.push('"'),
                    b'\\' => s.push('\\'),
                    b'/' => s.push('/'),
                    b'b' => s.push('\u{0008}'),
                    b'f' => s.push('\u{000C}'),
                    b'n' => s.push('\n'),
                    b'r' => s.push('\r'),
                    b't' => s.push('\t'),
                    b'u' => {
                        let cp = self.parse_hex4()?;
                        // Check for surrogate pair
                        if (0xD800..=0xDBFF).contains(&cp) {
                            // High surrogate, expect \uXXXX for low surrogate
                            if self.pos + 1 < self.source.len()
                                && self.source[self.pos] == b'\\'
                                && self.source[self.pos + 1] == b'u'
                            {
                                self.pos += 2;
                                let low = self.parse_hex4()?;
                                if (0xDC00..=0xDFFF).contains(&low) {
                                    let full = 0x10000 + ((cp as u32 - 0xD800) << 10) + (low as u32 - 0xDC00);
                                    if let Some(c) = char::from_u32(full) {
                                        s.push(c);
                                    } else {
                                        return Err(self.fail("잘못된 유니코드 서로게이트 쌍"));
                                    }
                                } else {
                                    return Err(self.fail("잘못된 유니코드 서로게이트 쌍"));
                                }
                            } else {
                                return Err(self.fail("잘못된 유니코드 서로게이트 쌍"));
                            }
                        } else if (0xDC00..=0xDFFF).contains(&cp) {
                            return Err(self.fail("잘못된 유니코드 서로게이트"));
                        } else {
                            if let Some(c) = char::from_u32(cp as u32) {
                                s.push(c);
                            } else {
                                return Err(self.fail("잘못된 유니코드 코드포인트"));
                            }
                        }
                    }
                    _ => return Err(self.fail(&format!("알 수 없는 JSON 이스케이프: \\{}", esc as char))),
                }
            } else {
                // Regular UTF-8 byte
                // Re-read as a UTF-8 character sequence
                let start = self.pos - 1;
                // Determine UTF-8 sequence length
                let byte_count = if ch < 0x80 {
                    1
                } else if ch < 0xE0 {
                    2
                } else if ch < 0xF0 {
                    3
                } else {
                    4
                };
                let end = start + byte_count;
                if end > self.source.len() {
                    return Err(self.fail("잘못된 UTF-8 시퀀스"));
                }
                // We already consumed one byte, consume the rest
                for _ in 1..byte_count {
                    self.advance();
                }
                if let Ok(str_slice) = std::str::from_utf8(&self.source[start..end]) {
                    s.push_str(str_slice);
                } else {
                    return Err(self.fail("잘못된 UTF-8 시퀀스"));
                }
            }
        }
    }

    fn parse_hex4(&mut self) -> Result<u16, CokacError> {
        let mut val: u16 = 0;
        for _ in 0..4 {
            if self.pos >= self.source.len() {
                return Err(self.fail("불완전한 \\uXXXX 이스케이프"));
            }
            let ch = self.advance();
            let digit = match ch {
                b'0'..=b'9' => ch - b'0',
                b'a'..=b'f' => ch - b'a' + 10,
                b'A'..=b'F' => ch - b'A' + 10,
                _ => return Err(self.fail("잘못된 \\uXXXX 이스케이프")),
            };
            val = val * 16 + digit as u16;
        }
        Ok(val)
    }

    fn parse_number(&mut self) -> Result<Value, CokacError> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.advance();
        }
        if self.peek() == Some(b'0') {
            self.advance();
        } else if matches!(self.peek(), Some(b'1'..=b'9')) {
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.advance();
            }
        } else {
            return Err(self.fail("잘못된 JSON 숫자"));
        }
        if self.peek() == Some(b'.') {
            self.advance();
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.fail("소수점 뒤에 숫자가 필요합니다."));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.advance();
            }
        }
        if matches!(self.peek(), Some(b'e') | Some(b'E')) {
            self.advance();
            if matches!(self.peek(), Some(b'+') | Some(b'-')) {
                self.advance();
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.fail("지수 뒤에 숫자가 필요합니다."));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.advance();
            }
        }
        let num_str = std::str::from_utf8(&self.source[start..self.pos])
            .map_err(|_| self.fail("잘못된 숫자 인코딩"))?;
        let n: f64 = num_str.parse().map_err(|_| self.fail("숫자 변환 실패"))?;
        Ok(Value::Number(n))
    }

    fn parse_array(&mut self, depth: usize) -> Result<Value, CokacError> {
        self.advance(); // skip [
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.advance();
            return Ok(Value::new_array(items));
        }
        loop {
            let val = self.parse_value(depth + 1)?;
            items.push(val);
            self.skip_ws();
            match self.peek() {
                Some(b',') => { self.advance(); }
                Some(b']') => { self.advance(); break; }
                _ => return Err(self.fail("배열에서 ',' 또는 ']' 기대됨")),
            }
        }
        Ok(Value::new_array(items))
    }

    fn parse_object(&mut self, depth: usize) -> Result<Value, CokacError> {
        self.advance(); // skip {
        let mut obj = ObjectValue::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.advance();
            return Ok(Value::Object(Rc::new(RefCell::new(obj))));
        }
        loop {
            self.skip_ws();
            if self.peek() != Some(b'"') {
                return Err(self.fail("객체 키는 문자열이어야 합니다."));
            }
            let key = self.parse_string()?;
            self.skip_ws();
            if self.peek() != Some(b':') {
                return Err(self.fail("객체에서 ':' 기대됨"));
            }
            self.advance();
            let val = self.parse_value(depth + 1)?;
            obj.set(key, val);
            self.skip_ws();
            match self.peek() {
                Some(b',') => { self.advance(); }
                Some(b'}') => { self.advance(); break; }
                _ => return Err(self.fail("객체에서 ',' 또는 '}' 기대됨")),
            }
        }
        Ok(Value::Object(Rc::new(RefCell::new(obj))))
    }
}

pub fn json_parse(text: &str, line: i32) -> Result<Value, CokacError> {
    let mut parser = JsonParser::new(text, line);
    let value = parser.parse_value(0)?;
    parser.skip_ws();
    if parser.pos < parser.source.len() {
        return Err(parser.fail("JSON 끝 뒤에 추가 문자가 있습니다."));
    }
    Ok(value)
}

// ----- Serialization -----

pub fn json_stringify(value: &Value, line: i32) -> Result<String, CokacError> {
    json_stringify_impl(value, false, 2, line)
}

pub fn json_stringify_pretty(value: &Value, indent: usize, line: i32) -> Result<String, CokacError> {
    json_stringify_impl(value, true, indent, line)
}

fn json_stringify_impl(value: &Value, pretty: bool, indent_step: usize, line: i32) -> Result<String, CokacError> {
    let mut output = String::new();
    let mut visited: Vec<*const ()> = Vec::new();
    stringify_value(value, &mut output, pretty, indent_step, 0, line, &mut visited)?;
    Ok(output)
}

fn stringify_value(
    value: &Value,
    out: &mut String,
    pretty: bool,
    indent_step: usize,
    depth: usize,
    line: i32,
    visited: &mut Vec<*const ()>,
) -> Result<(), CokacError> {
    match value {
        Value::Nil => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => {
            if n.is_nan() || n.is_infinite() {
                return Err(CokacError::new(
                    "NaN/Infinity는 JSON으로 직렬화할 수 없습니다.".to_string(),
                    line,
                ));
            }
            out.push_str(&crate::value::format_number(*n));
        }
        Value::String(s) => {
            json_escape_string(s, out);
        }
        Value::Array(arr) => {
            let ptr = &*arr.borrow() as *const ArrayValue as *const ();
            if visited.contains(&ptr) {
                return Err(CokacError::new(
                    "JSON 직렬화 중 순환 참조가 감지되었습니다.".to_string(),
                    line,
                ));
            }
            visited.push(ptr);
            let arr = arr.borrow();
            if arr.items.is_empty() {
                out.push_str("[]");
            } else {
                out.push('[');
                for (i, item) in arr.items.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    if pretty {
                        out.push('\n');
                        for _ in 0..(depth + 1) * indent_step {
                            out.push(' ');
                        }
                    }
                    stringify_value(item, out, pretty, indent_step, depth + 1, line, visited)?;
                }
                if pretty {
                    out.push('\n');
                    for _ in 0..depth * indent_step {
                        out.push(' ');
                    }
                }
                out.push(']');
            }
            visited.pop();
        }
        Value::Object(obj) => {
            let ptr = &*obj.borrow() as *const ObjectValue as *const ();
            if visited.contains(&ptr) {
                return Err(CokacError::new(
                    "JSON 직렬화 중 순환 참조가 감지되었습니다.".to_string(),
                    line,
                ));
            }
            visited.push(ptr);
            let obj = obj.borrow();
            if obj.count() == 0 {
                out.push_str("{}");
            } else {
                out.push('{');
                for (i, (key, val)) in obj.keys.iter().zip(obj.values.iter()).enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    if pretty {
                        out.push('\n');
                        for _ in 0..(depth + 1) * indent_step {
                            out.push(' ');
                        }
                    }
                    json_escape_string(key, out);
                    out.push(':');
                    if pretty { out.push(' '); }
                    stringify_value(val, out, pretty, indent_step, depth + 1, line, visited)?;
                }
                if pretty {
                    out.push('\n');
                    for _ in 0..depth * indent_step {
                        out.push(' ');
                    }
                }
                out.push('}');
            }
            visited.pop();
        }
        Value::Function(_) => {
            return Err(CokacError::new(
                "함수를 JSON으로 직렬화할 수 없습니다.".to_string(),
                line,
            ));
        }
        Value::Task(_) => {
            return Err(CokacError::new(
                "작업을 JSON으로 직렬화할 수 없습니다.".to_string(),
                line,
            ));
        }
    }
    Ok(())
}

fn json_escape_string(s: &str, out: &mut String) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000C}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}
