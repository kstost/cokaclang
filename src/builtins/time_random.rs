use crate::error::CokacError;
use crate::value::Value;

pub fn builtin_current_time(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if !args.is_empty() {
        return Err(CokacError::new("'현재시간'은 인수가 필요 없습니다.".to_string(), line));
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as f64;
    Ok(Value::Number(now))
}

pub fn builtin_time_string(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() > 1 {
        return Err(CokacError::new("'시간문자열'은 0~1개의 인수가 필요합니다.".to_string(), line));
    }
    let timestamp = if args.len() == 1 {
        crate::value::value_to_number(&args[0], line)? as i64
    } else {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
    };

    // Format as "YYYY-MM-DD HH:MM:SS" in local time
    
    let output = std::process::Command::new("date")
        .arg("-d")
        .arg(format!("@{}", timestamp))
        .arg("+%Y-%m-%d %H:%M:%S")
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            Ok(Value::String(s))
        }
        _ => {
            // Fallback: just return timestamp as string
            Ok(Value::String(format!("{}", timestamp)))
        }
    }
}

pub fn builtin_sleep(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'대기밀리초'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let ms = crate::value::value_to_number(&args[0], line)?;
    if ms < 0.0 {
        return Err(CokacError::new("'대기밀리초' 값은 0 이상이어야 합니다.".to_string(), line));
    }
    std::thread::sleep(std::time::Duration::from_millis(ms as u64));
    Ok(Value::Bool(true))
}

pub fn builtin_random(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if !args.is_empty() {
        return Err(CokacError::new("'난수'는 인수가 필요 없습니다.".to_string(), line));
    }
    use rand::Rng;
    let val: f64 = rand::thread_rng().gen();
    Ok(Value::Number(val))
}

pub fn builtin_random_int(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'난수정수'는 2개의 인수가 필요합니다.".to_string(), line));
    }
    let mut min = crate::value::value_to_number(&args[0], line)? as i64;
    let mut max = crate::value::value_to_number(&args[1], line)? as i64;
    if min > max {
        std::mem::swap(&mut min, &mut max);
    }
    use rand::Rng;
    let val = rand::thread_rng().gen_range(min..=max);
    Ok(Value::Number(val as f64))
}
