use std::fs;
use std::path::{Path, PathBuf};

use crate::error::CokacError;
use crate::runtime::Runtime;
use crate::value::Value;

pub fn builtin_dir_list(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    let path = if args.is_empty() {
        ".".to_string()
    } else {
        runtime.resolve_path(&args[0].to_display_string())
    };
    let mut entries: Vec<String> = Vec::new();
    let dir = fs::read_dir(&path).map_err(|e| {
        CokacError::new(format!("디렉토리를 읽을 수 없습니다: '{}': {}", path, e), line)
    })?;
    for entry in dir {
        if let Ok(entry) = entry {
            let name = entry.file_name().to_string_lossy().to_string();
            if name != "." && name != ".." {
                entries.push(name);
            }
        }
    }
    entries.sort();
    let items: Vec<Value> = entries.into_iter().map(Value::String).collect();
    Ok(Value::new_array(items))
}

pub fn builtin_dir_create(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'디렉토리생성'은 1개의 인수가 필요합니다.".to_string(), line));
    }
    let path = runtime.resolve_path(&args[0].to_display_string());
    match fs::create_dir(&path) {
        Ok(_) => Ok(Value::Bool(true)),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            if Path::new(&path).is_dir() {
                Ok(Value::Bool(true))
            } else {
                Err(CokacError::new(
                    format!("경로가 이미 존재하지만 디렉토리가 아닙니다: '{}'", path),
                    line,
                ))
            }
        }
        Err(e) => Err(CokacError::new(
            format!("디렉토리를 생성할 수 없습니다: '{}': {}", path, e),
            line,
        )),
    }
}

pub fn builtin_dir_remove(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'디렉토리삭제'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let path = runtime.resolve_path(&args[0].to_display_string());
    match fs::remove_dir(&path) {
        Ok(_) => Ok(Value::Bool(true)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Value::Bool(false)),
        Err(e) => Err(CokacError::new(
            format!("디렉토리를 삭제할 수 없습니다: '{}': {}", path, e),
            line,
        )),
    }
}

pub fn builtin_dir_remove_recursive(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'디렉토리삭제재귀'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let path = runtime.resolve_path(&args[0].to_display_string());
    match fs::remove_dir_all(&path) {
        Ok(_) => Ok(Value::Bool(true)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Value::Bool(false)),
        Err(e) => Err(CokacError::new(
            format!("디렉토리를 삭제할 수 없습니다: '{}': {}", path, e),
            line,
        )),
    }
}

pub fn builtin_dir_copy(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'디렉토리복사'는 2개의 인수가 필요합니다.".to_string(), line));
    }
    let src = runtime.resolve_path(&args[0].to_display_string());
    let dst = runtime.resolve_path(&args[1].to_display_string());
    copy_dir_recursive(Path::new(&src), Path::new(&dst)).map_err(|e| {
        CokacError::new(format!("디렉토리를 복사할 수 없습니다: '{}' → '{}': {}", src, dst, e), line)
    })?;
    Ok(Value::Bool(true))
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if src.is_file() {
        fs::copy(src, dst)?;
        return Ok(());
    }
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

pub fn builtin_dir_exists(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'디렉토리존재'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let path = runtime.resolve_path(&args[0].to_display_string());
    Ok(Value::Bool(Path::new(&path).is_dir()))
}

pub fn builtin_cwd(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if !args.is_empty() {
        return Err(CokacError::new("'현재디렉토리'는 인수가 필요 없습니다.".to_string(), line));
    }
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());
    Ok(Value::String(cwd))
}

pub fn builtin_path_join(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'경로합치기'는 2개의 인수가 필요합니다.".to_string(), line));
    }
    let base = args[0].to_display_string();
    let child = args[1].to_display_string();
    if child.is_empty() {
        return Ok(Value::String(base));
    }
    if Path::new(&child).is_absolute() {
        return Ok(Value::String(child));
    }
    let joined = PathBuf::from(&base).join(&child);
    Ok(Value::String(joined.to_string_lossy().to_string()))
}

pub fn builtin_abs_path(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'절대경로'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let path = runtime.resolve_path(&args[0].to_display_string());
    match fs::canonicalize(&path) {
        Ok(p) => Ok(Value::String(p.to_string_lossy().to_string())),
        Err(_) => Ok(Value::String(path)),
    }
}

pub fn builtin_basename(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'경로이름'은 1개의 인수가 필요합니다.".to_string(), line));
    }
    let path = args[0].to_display_string();
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() {
        return Ok(Value::String("/".to_string()));
    }
    let name = trimmed.rsplit('/').next().unwrap_or(trimmed);
    Ok(Value::String(name.to_string()))
}

pub fn builtin_dirname(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'상위경로'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let path = args[0].to_display_string();
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() {
        return Ok(Value::String("/".to_string()));
    }
    match trimmed.rfind('/') {
        Some(0) => Ok(Value::String("/".to_string())),
        Some(pos) => Ok(Value::String(trimmed[..pos].to_string())),
        None => Ok(Value::String(".".to_string())),
    }
}

pub fn builtin_extension(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'확장자'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let path = args[0].to_display_string();
    // Get basename first
    let trimmed = path.trim_end_matches('/');
    let basename = if trimmed.is_empty() {
        "/"
    } else {
        trimmed.rsplit('/').next().unwrap_or(trimmed)
    };
    // Find last dot
    match basename.rfind('.') {
        Some(0) => Ok(Value::String(String::new())), // hidden file like .gitignore
        Some(pos) if pos < basename.len() - 1 => {
            Ok(Value::String(basename[pos + 1..].to_string()))
        }
        _ => Ok(Value::String(String::new())),
    }
}

pub fn builtin_normalize(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'경로정규화'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let path = args[0].to_display_string();
    Ok(Value::String(normalize_path(&path)))
}

pub fn builtin_relative(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'상대경로'는 2개의 인수가 필요합니다.".to_string(), line));
    }
    let base = normalize_path(&args[0].to_display_string());
    let target = normalize_path(&args[1].to_display_string());

    let base_abs = base.starts_with('/');
    let target_abs = target.starts_with('/');
    if base_abs != target_abs {
        return Ok(Value::String(target));
    }

    let base_parts: Vec<&str> = base.split('/').filter(|s| !s.is_empty()).collect();
    let target_parts: Vec<&str> = target.split('/').filter(|s| !s.is_empty()).collect();

    // Find common prefix
    let mut common = 0;
    for (a, b) in base_parts.iter().zip(target_parts.iter()) {
        if a == b { common += 1; } else { break; }
    }

    if common == base_parts.len() && common == target_parts.len() {
        return Ok(Value::String(".".to_string()));
    }

    let mut result = Vec::new();
    for _ in common..base_parts.len() {
        result.push("..");
    }
    for &part in &target_parts[common..] {
        result.push(part);
    }
    Ok(Value::String(result.join("/")))
}

pub fn builtin_path_exists(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'경로존재'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let path = runtime.resolve_path(&args[0].to_display_string());
    Ok(Value::Bool(Path::new(&path).exists()))
}

fn normalize_path(path: &str) -> String {
    if path.is_empty() {
        return ".".to_string();
    }
    let is_absolute = path.starts_with('/');
    let parts: Vec<&str> = path.split('/').collect();
    let mut result: Vec<&str> = Vec::new();

    for part in &parts {
        match *part {
            "" | "." => continue,
            ".." => {
                if !result.is_empty() && *result.last().unwrap() != ".." {
                    result.pop();
                } else if !is_absolute {
                    result.push("..");
                }
            }
            other => result.push(other),
        }
    }

    if result.is_empty() {
        if is_absolute { "/".to_string() } else { ".".to_string() }
    } else if is_absolute {
        format!("/{}", result.join("/"))
    } else {
        result.join("/")
    }
}
