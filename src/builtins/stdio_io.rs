#[cfg(not(target_arch = "wasm32"))]
use std::io::{self, Write, BufRead};

use crate::error::CokacError;
use crate::value::Value;

pub fn builtin_input(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() > 1 {
        return Err(CokacError::new("'입력'은 0~1개의 인수가 필요합니다.".to_string(), line));
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(prompt) = args.get(0) {
            print!("{}", prompt.to_display_string());
            io::stdout().flush().ok();
        }
        let mut input = String::new();
        match io::stdin().lock().read_line(&mut input) {
            Ok(0) => Ok(Value::Nil), // EOF
            Ok(_) => {
                let trimmed = input.trim_end_matches('\n').trim_end_matches('\r');
                Ok(Value::String(trimmed.to_string()))
            }
            Err(_) => Ok(Value::Nil),
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = args;
        Err(CokacError::new("WASM 환경에서는 입력을 지원하지 않습니다.".to_string(), line))
    }
}

pub fn builtin_stdin_read(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if !args.is_empty() {
        return Err(CokacError::new("'표준입력읽기'는 인수가 필요 없습니다.".to_string(), line));
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut input = String::new();
        match io::stdin().lock().read_line(&mut input) {
            Ok(0) => Ok(Value::Nil),
            Ok(_) => {
                let trimmed = input.trim_end_matches('\n').trim_end_matches('\r');
                Ok(Value::String(trimmed.to_string()))
            }
            Err(_) => Ok(Value::Nil),
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        Err(CokacError::new("WASM 환경에서는 표준입력읽기를 지원하지 않습니다.".to_string(), line))
    }
}

pub fn builtin_stdout_write(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'표준출력쓰기'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let text = args[0].to_display_string();
    let bytes = text.as_bytes();
    #[cfg(not(target_arch = "wasm32"))]
    {
        io::stdout().write_all(bytes).ok();
        io::stdout().flush().ok();
    }
    #[cfg(target_arch = "wasm32")]
    {
        crate::output::capture_write(&text);
    }
    Ok(Value::Number(bytes.len() as f64))
}

pub fn builtin_stdout_writeln(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() > 1 {
        return Err(CokacError::new("'표준출력줄'은 0~1개의 인수가 필요합니다.".to_string(), line));
    }
    let text = if args.is_empty() {
        String::new()
    } else {
        args[0].to_display_string()
    };
    let full = format!("{}\n", text);
    #[cfg(not(target_arch = "wasm32"))]
    {
        io::stdout().write_all(full.as_bytes()).ok();
        io::stdout().flush().ok();
    }
    #[cfg(target_arch = "wasm32")]
    {
        crate::output::capture_write(&full);
    }
    Ok(Value::Number(full.len() as f64))
}

pub fn builtin_stderr_write(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'표준에러쓰기'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let text = args[0].to_display_string();
    let bytes = text.as_bytes();
    #[cfg(not(target_arch = "wasm32"))]
    {
        io::stderr().write_all(bytes).ok();
        io::stderr().flush().ok();
    }
    #[cfg(target_arch = "wasm32")]
    {
        crate::output::capture_write(&format!("[에러] {}", text));
    }
    Ok(Value::Number(bytes.len() as f64))
}

pub fn builtin_stderr_writeln(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() > 1 {
        return Err(CokacError::new("'표준에러줄'은 0~1개의 인수가 필요합니다.".to_string(), line));
    }
    let text = if args.is_empty() {
        String::new()
    } else {
        args[0].to_display_string()
    };
    let full = format!("{}\n", text);
    #[cfg(not(target_arch = "wasm32"))]
    {
        io::stderr().write_all(full.as_bytes()).ok();
        io::stderr().flush().ok();
    }
    #[cfg(target_arch = "wasm32")]
    {
        crate::output::capture_write(&format!("[에러] {}", full));
    }
    Ok(Value::Number(full.len() as f64))
}
