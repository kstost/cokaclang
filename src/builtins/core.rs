use crate::error::CokacError;
use crate::value::{Value, value_to_number};

pub fn builtin_length(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'길이'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let len = match &args[0] {
        Value::String(s) => s.len(),
        Value::Array(a) => a.borrow().items.len(),
        Value::Object(o) => o.borrow().count(),
        other => other.to_display_string().len(),
    };
    Ok(Value::Number(len as f64))
}

pub fn builtin_assert(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.is_empty() || args.len() > 2 {
        return Err(CokacError::new("'단언'은 1~2개의 인수가 필요합니다.".to_string(), line));
    }
    if !args[0].is_truthy() {
        let msg = if args.len() > 1 {
            args[1].to_display_string()
        } else {
            "단언 실패".to_string()
        };
        return Err(CokacError::new(msg, line));
    }
    Ok(Value::Bool(true))
}

pub fn builtin_to_string(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'문자열'은 1개의 인수가 필요합니다.".to_string(), line));
    }
    Ok(Value::String(args[0].to_display_string()))
}

pub fn builtin_to_bool(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'불린'은 1개의 인수가 필요합니다.".to_string(), line));
    }
    Ok(Value::Bool(args[0].is_truthy()))
}

pub fn builtin_to_number(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'숫자'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let n = value_to_number(&args[0], line)?;
    Ok(Value::Number(n))
}

pub fn builtin_abs(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'절댓값'은 1개의 인수가 필요합니다.".to_string(), line));
    }
    let n = value_to_number(&args[0], line)?;
    Ok(Value::Number(n.abs()))
}

pub fn builtin_integer(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'정수'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let n = value_to_number(&args[0], line)?;
    Ok(Value::Number(n.trunc()))
}

pub fn builtin_max(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() < 2 {
        return Err(CokacError::new("'최대'는 2개 이상의 인수가 필요합니다.".to_string(), line));
    }
    let mut max = value_to_number(&args[0], line)?;
    for arg in &args[1..] {
        let n = value_to_number(arg, line)?;
        if n > max { max = n; }
    }
    Ok(Value::Number(max))
}

pub fn builtin_min(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() < 2 {
        return Err(CokacError::new("'최소'는 2개 이상의 인수가 필요합니다.".to_string(), line));
    }
    let mut min = value_to_number(&args[0], line)?;
    for arg in &args[1..] {
        let n = value_to_number(arg, line)?;
        if n < min { min = n; }
    }
    Ok(Value::Number(min))
}

pub fn builtin_type(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'타입'은 1개의 인수가 필요합니다.".to_string(), line));
    }
    Ok(Value::String(args[0].type_name().to_string()))
}
