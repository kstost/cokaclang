use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

use crate::error::CokacError;
use crate::runtime::Runtime;
use crate::value::*;

pub fn builtin_file_read(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'파일읽기'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let path = runtime.resolve_path(&args[0].to_display_string());
    match fs::read_to_string(&path) {
        Ok(content) => Ok(Value::String(content)),
        Err(e) => {
            let msg = if e.kind() == std::io::ErrorKind::NotFound {
                format!("파일을 찾을 수 없습니다: '{}'", path)
            } else {
                format!("파일을 읽을 수 없습니다: '{}': {}", path, e)
            };
            Err(CokacError::new(msg, line))
        }
    }
}

pub fn builtin_file_read_lines(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'파일읽기줄들'은 1개의 인수가 필요합니다.".to_string(), line));
    }
    let path = runtime.resolve_path(&args[0].to_display_string());
    match fs::read_to_string(&path) {
        Ok(content) => {
            let lines: Vec<Value> = content
                .split('\n')
                .map(|l| Value::String(l.trim_end_matches('\r').to_string()))
                .collect();
            Ok(Value::new_array(lines))
        }
        Err(e) => Err(CokacError::new(
            format!("파일을 읽을 수 없습니다: '{}': {}", path, e),
            line,
        )),
    }
}

pub fn builtin_file_write(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'파일쓰기'는 2개의 인수가 필요합니다.".to_string(), line));
    }
    let path = runtime.resolve_path(&args[0].to_display_string());
    let content = args[1].to_display_string();
    fs::write(&path, &content).map_err(|e| {
        CokacError::new(format!("파일을 쓸 수 없습니다: '{}': {}", path, e), line)
    })?;
    Ok(Value::Bool(true))
}

pub fn builtin_file_write_lines(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'파일쓰기줄들'은 2개의 인수가 필요합니다.".to_string(), line));
    }
    let path = runtime.resolve_path(&args[0].to_display_string());
    match &args[1] {
        Value::Array(arr) => {
            let arr = arr.borrow();
            let lines: Vec<String> = arr.items.iter().map(|v| v.to_display_string()).collect();
            let content = lines.join("\n");
            fs::write(&path, &content).map_err(|e| {
                CokacError::new(format!("파일을 쓸 수 없습니다: '{}': {}", path, e), line)
            })?;
            Ok(Value::Bool(true))
        }
        _ => Err(CokacError::new("'파일쓰기줄들'의 두 번째 인수는 배열이어야 합니다.".to_string(), line)),
    }
}

pub fn builtin_file_append(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'파일추가'는 2개의 인수가 필요합니다.".to_string(), line));
    }
    let path = runtime.resolve_path(&args[0].to_display_string());
    let content = args[1].to_display_string();
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| CokacError::new(format!("파일을 열 수 없습니다: '{}': {}", path, e), line))?;
    file.write_all(content.as_bytes())
        .map_err(|e| CokacError::new(format!("파일에 추가할 수 없습니다: '{}': {}", path, e), line))?;
    Ok(Value::Bool(true))
}

pub fn builtin_file_copy(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'파일복사'는 2개의 인수가 필요합니다.".to_string(), line));
    }
    let src = runtime.resolve_path(&args[0].to_display_string());
    let dst = runtime.resolve_path(&args[1].to_display_string());
    fs::copy(&src, &dst).map_err(|e| {
        CokacError::new(format!("파일을 복사할 수 없습니다: '{}' → '{}': {}", src, dst, e), line)
    })?;
    Ok(Value::Bool(true))
}

pub fn builtin_file_move(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'파일이동'은 2개의 인수가 필요합니다.".to_string(), line));
    }
    let src = runtime.resolve_path(&args[0].to_display_string());
    let dst = runtime.resolve_path(&args[1].to_display_string());
    if !std::path::Path::new(&src).exists() {
        return Ok(Value::Bool(false));
    }
    match fs::rename(&src, &dst) {
        Ok(_) => Ok(Value::Bool(true)),
        Err(e) => {
            // Cross-device fallback (EXDEV)
            if e.raw_os_error() == Some(18) {
                fs::copy(&src, &dst).map_err(|e| {
                    CokacError::new(format!("파일을 이동할 수 없습니다: '{}' → '{}': {}", src, dst, e), line)
                })?;
                let _ = fs::remove_file(&src);
                Ok(Value::Bool(true))
            } else {
                Err(CokacError::new(format!("파일을 이동할 수 없습니다: '{}' → '{}': {}", src, dst, e), line))
            }
        }
    }
}

pub fn builtin_file_exists(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'파일존재'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let path = runtime.resolve_path(&args[0].to_display_string());
    Ok(Value::Bool(std::path::Path::new(&path).exists()))
}

pub fn builtin_file_delete(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'파일삭제'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let path = runtime.resolve_path(&args[0].to_display_string());
    match fs::remove_file(&path) {
        Ok(_) => Ok(Value::Bool(true)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Value::Bool(false)),
        Err(e) => Err(CokacError::new(
            format!("파일을 삭제할 수 없습니다: '{}': {}", path, e),
            line,
        )),
    }
}

pub fn builtin_file_info(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'파일정보'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let path = runtime.resolve_path(&args[0].to_display_string());
    let meta = fs::metadata(&path).map_err(|e| {
        CokacError::new(format!("파일 정보를 가져올 수 없습니다: '{}': {}", path, e), line)
    })?;
    let obj = ObjectValue::new();
    let obj = Rc::new(RefCell::new(obj));
    {
        let mut o = obj.borrow_mut();
        o.set("경로".to_string(), Value::String(path));
        o.set("크기".to_string(), Value::Number(meta.len() as f64));
        o.set("디렉토리".to_string(), Value::Bool(meta.is_dir()));
        o.set("파일".to_string(), Value::Bool(meta.is_file()));
        let mtime = meta.modified()
            .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as f64)
            .unwrap_or(0.0);
        o.set("수정시각".to_string(), Value::Number(mtime));
    }
    Ok(Value::Object(obj))
}

pub fn builtin_file_size(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'파일크기'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let path = runtime.resolve_path(&args[0].to_display_string());
    let meta = fs::metadata(&path).map_err(|e| {
        CokacError::new(format!("파일 정보를 가져올 수 없습니다: '{}': {}", path, e), line)
    })?;
    Ok(Value::Number(meta.len() as f64))
}

pub fn builtin_file_mtime(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'파일수정시각'은 1개의 인수가 필요합니다.".to_string(), line));
    }
    let path = runtime.resolve_path(&args[0].to_display_string());
    let meta = fs::metadata(&path).map_err(|e| {
        CokacError::new(format!("파일 정보를 가져올 수 없습니다: '{}': {}", path, e), line)
    })?;
    let mtime = meta.modified()
        .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() as f64)
        .unwrap_or(0.0);
    Ok(Value::Number(mtime))
}
