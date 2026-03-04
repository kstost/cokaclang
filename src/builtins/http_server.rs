use std::cell::RefCell;
use std::io::{Read, Write, BufRead, BufReader};
use std::net::{TcpListener, TcpStream};
use std::rc::Rc;

use crate::error::CokacError;
use crate::value::*;

pub fn builtin_server_listen(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.is_empty() || args.len() > 3 {
        return Err(CokacError::new("'서버열기'는 1~3개의 인수가 필요합니다.".to_string(), line));
    }
    let port = crate::value::value_to_number(&args[0], line)? as u16;
    if port == 0 {
        return Err(CokacError::new("포트 번호는 1 이상이어야 합니다.".to_string(), line));
    }

    let host = if args.len() > 1 {
        match &args[1] {
            Value::Nil => "0.0.0.0".to_string(),
            v => v.to_display_string(),
        }
    } else {
        "0.0.0.0".to_string()
    };

    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr).map_err(|e| {
        CokacError::new(format!("서버를 열 수 없습니다 ({}): {}", addr, e), line)
    })?;

    // Store the raw fd as a number (Linux-specific but portable enough)
    use std::os::unix::io::IntoRawFd;
    let fd = listener.into_raw_fd();
    Ok(Value::Number(fd as f64))
}

pub fn builtin_accept_request(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'요청받기'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let fd = crate::value::value_to_number(&args[0], line)? as i32;

    use std::os::unix::io::FromRawFd;
    let listener = unsafe { TcpListener::from_raw_fd(fd) };

    let (stream, addr) = listener.accept().map_err(|e| {
        CokacError::new(format!("요청 수락 실패: {}", e), line)
    })?;

    // Don't drop the listener (it would close the fd)
    std::mem::forget(listener);

    // Set timeouts
    stream.set_read_timeout(Some(std::time::Duration::from_secs(30))).ok();
    stream.set_write_timeout(Some(std::time::Duration::from_secs(30))).ok();

    // Read the request
    let mut reader = BufReader::new(&stream);
    let mut request_text = String::new();

    // Read request line and headers
    loop {
        let mut line_buf = String::new();
        match reader.read_line(&mut line_buf) {
            Ok(0) => break,
            Ok(_) => {
                request_text.push_str(&line_buf);
                if line_buf == "\r\n" || line_buf == "\n" {
                    break;
                }
            }
            Err(_) => break,
        }
    }

    // Parse request line
    let first_line = request_text.lines().next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    let method = parts.get(0).unwrap_or(&"GET").to_string();
    let path = parts.get(1).unwrap_or(&"/").to_string();
    let version = parts.get(2).unwrap_or(&"HTTP/1.1").to_string();

    // Parse headers
    let headers_obj = ObjectValue::new();
    let headers_rc = Rc::new(RefCell::new(headers_obj));
    let mut content_length: usize = 0;
    for line_str in request_text.lines().skip(1) {
        if line_str.is_empty() || line_str == "\r" {
            break;
        }
        if let Some(colon_pos) = line_str.find(':') {
            let key = line_str[..colon_pos].trim().to_string();
            let val = line_str[colon_pos + 1..].trim().to_string();
            if key.to_lowercase() == "content-length" {
                content_length = val.parse().unwrap_or(0);
            }
            headers_rc.borrow_mut().set(key, Value::String(val));
        }
    }

    // Read body if content-length > 0
    let body = if content_length > 0 {
        let mut body_buf = vec![0u8; content_length];
        reader.read_exact(&mut body_buf).ok();
        String::from_utf8_lossy(&body_buf).to_string()
    } else {
        String::new()
    };

    // Store the connection fd
    use std::os::unix::io::IntoRawFd;
    let conn_fd = stream.into_raw_fd();

    let obj = ObjectValue::new();
    let obj = Rc::new(RefCell::new(obj));
    {
        let mut o = obj.borrow_mut();
        o.set("연결".to_string(), Value::Number(conn_fd as f64));
        o.set("메서드".to_string(), Value::String(method));
        o.set("경로".to_string(), Value::String(path));
        o.set("버전".to_string(), Value::String(version));
        o.set("헤더".to_string(), Value::Object(headers_rc));
        o.set("본문".to_string(), Value::String(body));
        o.set("원격주소".to_string(), Value::String(addr.ip().to_string()));
    }
    Ok(Value::Object(obj))
}

pub fn builtin_send_response(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() < 3 || args.len() > 4 {
        return Err(CokacError::new("'응답보내기'는 3~4개의 인수가 필요합니다.".to_string(), line));
    }
    let fd = crate::value::value_to_number(&args[0], line)? as i32;
    let status = crate::value::value_to_number(&args[1], line)? as u16;
    let body = args[2].to_display_string();

    let status_text = match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        500 => "Internal Server Error",
        _ => "Unknown",
    };

    let mut response = format!("HTTP/1.1 {} {}\r\nConnection: close\r\nContent-Length: {}\r\n",
        status, status_text, body.len());

    // Add custom headers
    if let Some(Value::Object(headers)) = args.get(3) {
        let headers = headers.borrow();
        for (key, val) in headers.keys.iter().zip(headers.values.iter()) {
            response.push_str(&format!("{}: {}\r\n", key, val.to_display_string()));
        }
    }

    response.push_str("\r\n");
    response.push_str(&body);

    use std::os::unix::io::FromRawFd;
    let mut stream = unsafe { TcpStream::from_raw_fd(fd) };
    let result = stream.write_all(response.as_bytes());
    stream.flush().ok();
    // Don't close - let 연결닫기 handle it
    std::mem::forget(stream);

    Ok(Value::Bool(result.is_ok()))
}

pub fn builtin_close_connection(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'연결닫기'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let fd = crate::value::value_to_number(&args[0], line)? as i32;
    let result = unsafe { libc::close(fd) };
    Ok(Value::Bool(result == 0))
}
