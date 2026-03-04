use crate::error::CokacError;
use crate::value::Value;

fn expect_string(args: &[Value], idx: usize, name: &str, line: i32) -> Result<String, CokacError> {
    match args.get(idx) {
        Some(Value::String(s)) => Ok(s.clone()),
        Some(v) => Ok(v.to_display_string()),
        None => Err(CokacError::new(format!("'{}'에 인수가 부족합니다.", name), line)),
    }
}

pub fn builtin_contains(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'문자포함'은 2개의 인수가 필요합니다.".to_string(), line));
    }
    let text = expect_string(&args, 0, "문자포함", line)?;
    let needle = expect_string(&args, 1, "문자포함", line)?;
    Ok(Value::Bool(text.contains(&needle)))
}

pub fn builtin_replace(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 3 {
        return Err(CokacError::new("'문자치환'은 3개의 인수가 필요합니다.".to_string(), line));
    }
    let text = expect_string(&args, 0, "문자치환", line)?;
    let find = expect_string(&args, 1, "문자치환", line)?;
    let replace = expect_string(&args, 2, "문자치환", line)?;
    if find.is_empty() {
        return Err(CokacError::new("'문자치환' 검색 문자열이 비어 있습니다.".to_string(), line));
    }
    Ok(Value::String(text.replace(&find, &replace)))
}

pub fn builtin_split(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'문자분할'은 2개의 인수가 필요합니다.".to_string(), line));
    }
    let text = expect_string(&args, 0, "문자분할", line)?;
    let delimiter = expect_string(&args, 1, "문자분할", line)?;
    if delimiter.is_empty() {
        return Err(CokacError::new("'문자분할' 구분자가 비어 있습니다.".to_string(), line));
    }
    let parts: Vec<Value> = text.split(&delimiter).map(|s| Value::String(s.to_string())).collect();
    Ok(Value::new_array(parts))
}

pub fn builtin_starts_with(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'문자시작'은 2개의 인수가 필요합니다.".to_string(), line));
    }
    let text = expect_string(&args, 0, "문자시작", line)?;
    let prefix = expect_string(&args, 1, "문자시작", line)?;
    Ok(Value::Bool(text.starts_with(&prefix)))
}

pub fn builtin_ends_with(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'문자끝'은 2개의 인수가 필요합니다.".to_string(), line));
    }
    let text = expect_string(&args, 0, "문자끝", line)?;
    let suffix = expect_string(&args, 1, "문자끝", line)?;
    Ok(Value::Bool(text.ends_with(&suffix)))
}

pub fn builtin_trim(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'문자다듬기'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let text = expect_string(&args, 0, "문자다듬기", line)?;
    Ok(Value::String(text.trim().to_string()))
}

pub fn builtin_to_upper(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'문자대문자'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let text = expect_string(&args, 0, "문자대문자", line)?;
    // ASCII-only uppercasing (matching C behavior)
    let result: String = text.bytes().map(|b| {
        if b >= b'a' && b <= b'z' { (b - 32) as char } else { b as char }
    }).collect();
    Ok(Value::String(result))
}

pub fn builtin_to_lower(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'문자소문자'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let text = expect_string(&args, 0, "문자소문자", line)?;
    let result: String = text.bytes().map(|b| {
        if b >= b'A' && b <= b'Z' { (b + 32) as char } else { b as char }
    }).collect();
    Ok(Value::String(result))
}

pub fn builtin_remove_prefix(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'문자시작제거'는 2개의 인수가 필요합니다.".to_string(), line));
    }
    let text = expect_string(&args, 0, "문자시작제거", line)?;
    let prefix = expect_string(&args, 1, "문자시작제거", line)?;
    if text.starts_with(&prefix) {
        Ok(Value::String(text[prefix.len()..].to_string()))
    } else {
        Ok(Value::String(text))
    }
}

pub fn builtin_remove_suffix(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'문자끝제거'는 2개의 인수가 필요합니다.".to_string(), line));
    }
    let text = expect_string(&args, 0, "문자끝제거", line)?;
    let suffix = expect_string(&args, 1, "문자끝제거", line)?;
    if text.ends_with(&suffix) {
        Ok(Value::String(text[..text.len() - suffix.len()].to_string()))
    } else {
        Ok(Value::String(text))
    }
}

pub fn builtin_repeat(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'문자반복'은 2개의 인수가 필요합니다.".to_string(), line));
    }
    let text = expect_string(&args, 0, "문자반복", line)?;
    let count = crate::value::value_to_number(&args[1], line)?;
    if count < 0.0 {
        return Err(CokacError::new("'문자반복' 횟수는 0 이상이어야 합니다.".to_string(), line));
    }
    let count = count as usize;
    Ok(Value::String(text.repeat(count)))
}
