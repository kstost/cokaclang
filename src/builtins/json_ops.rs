use crate::error::CokacError;
use crate::json;
#[cfg(not(target_arch = "wasm32"))]
use crate::runtime::Runtime;
use crate::value::Value;

pub fn builtin_json_parse(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'자료파싱'은 1개의 인수가 필요합니다.".to_string(), line));
    }
    let text = args[0].to_display_string();
    json::json_parse(&text, line)
}

pub fn builtin_json_stringify(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'자료문자열화'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let s = json::json_stringify(&args[0], line)?;
    Ok(Value::String(s))
}

pub fn builtin_json_stringify_pretty(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.is_empty() || args.len() > 2 {
        return Err(CokacError::new("'자료예쁘게문자열화'는 1~2개의 인수가 필요합니다.".to_string(), line));
    }
    let indent = if args.len() > 1 {
        let n = crate::value::value_to_number(&args[1], line)?;
        let n = n as usize;
        if n > 16 { 16 } else { n }
    } else {
        2
    };
    let s = json::json_stringify_pretty(&args[0], indent, line)?;
    Ok(Value::String(s))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn builtin_json_read_file(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'자료읽기'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let path = runtime.resolve_path(&args[0].to_display_string());
    let content = std::fs::read_to_string(&path).map_err(|e| {
        CokacError::new(format!("파일을 읽을 수 없습니다: '{}': {}", path, e), line)
    })?;
    json::json_parse(&content, line)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn builtin_json_write_file(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'자료쓰기'는 2개의 인수가 필요합니다.".to_string(), line));
    }
    let path = runtime.resolve_path(&args[0].to_display_string());
    let s = json::json_stringify(&args[1], line)?;
    std::fs::write(&path, &s).map_err(|e| {
        CokacError::new(format!("파일을 쓸 수 없습니다: '{}': {}", path, e), line)
    })?;
    Ok(Value::Bool(true))
}

#[cfg(not(target_arch = "wasm32"))]
pub fn builtin_json_write_file_pretty(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if args.len() < 2 || args.len() > 3 {
        return Err(CokacError::new("'자료예쁘게쓰기'는 2~3개의 인수가 필요합니다.".to_string(), line));
    }
    let path = runtime.resolve_path(&args[0].to_display_string());
    let indent = if args.len() > 2 {
        let n = crate::value::value_to_number(&args[2], line)?;
        let n = n as usize;
        if n > 16 { 16 } else { n }
    } else {
        2
    };
    let s = json::json_stringify_pretty(&args[1], indent, line)?;
    std::fs::write(&path, &s).map_err(|e| {
        CokacError::new(format!("파일을 쓸 수 없습니다: '{}': {}", path, e), line)
    })?;
    Ok(Value::Bool(true))
}
