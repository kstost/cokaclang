use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;

use crate::error::CokacError;
use crate::runtime::Runtime;
use crate::value::*;

pub fn builtin_shell_exec(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.is_empty() || args.len() > 2 {
        return Err(CokacError::new("'명령실행'은 1~2개의 인수가 필요합니다.".to_string(), line));
    }
    let command = args[0].to_display_string();
    let opts = if args.len() > 1 { Some(&args[1]) } else { None };

    let shell_opts = parse_shell_options(opts, line)?;

    let mut cmd = Command::new("/bin/sh");
    cmd.arg("-c").arg(&command);
    apply_shell_options(&mut cmd, &shell_opts);

    if let Some(ref input) = shell_opts.stdin_text {
        use std::io::Write;
        use std::process::Stdio;
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        let mut child = cmd.spawn().map_err(|e| {
            CokacError::new(format!("명령 실행 실패: {}", e), line)
        })?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(input.as_bytes()).ok();
        }
        let output = child.wait_with_output().map_err(|e| {
            CokacError::new(format!("명령 실행 실패: {}", e), line)
        })?;
        return Ok(Value::String(String::from_utf8_lossy(&output.stdout).to_string()));
    }

    let output = cmd.output().map_err(|e| {
        CokacError::new(format!("명령 실행 실패: {}", e), line)
    })?;

    Ok(Value::String(String::from_utf8_lossy(&output.stdout).to_string()))
}

pub fn builtin_shell_exec_full(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.is_empty() || args.len() > 2 {
        return Err(CokacError::new("'명령실행결과'는 1~2개의 인수가 필요합니다.".to_string(), line));
    }
    let command = args[0].to_display_string();
    let opts = if args.len() > 1 { Some(&args[1]) } else { None };

    let shell_opts = parse_shell_options(opts, line)?;

    let mut cmd = Command::new("/bin/sh");
    cmd.arg("-c").arg(&command);
    apply_shell_options(&mut cmd, &shell_opts);

    use std::process::Stdio;
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    if shell_opts.stdin_text.is_some() {
        cmd.stdin(Stdio::piped());
    }

    // Set up process group for timeout kill
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                libc::setpgid(0, 0);
                Ok(())
            });
        }
    }

    let mut child = cmd.spawn().map_err(|e| {
        CokacError::new(format!("명령 실행 실패: {}", e), line)
    })?;

    if let Some(ref input) = shell_opts.stdin_text {
        use std::io::Write;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(input.as_bytes()).ok();
        }
    }

    if let Some(timeout_sec) = shell_opts.timeout_sec {
        let pid = child.id();
        let (tx, rx) = std::sync::mpsc::channel();
        let timeout = std::time::Duration::from_secs(timeout_sec);

        std::thread::spawn(move || {
            let result = child.wait_with_output();
            let _ = tx.send(result);
        });

        match rx.recv_timeout(timeout) {
            Ok(Ok(output)) => {
                let code = output.status.code().unwrap_or(-1);
                return Ok(make_exec_result(code,
                    &String::from_utf8_lossy(&output.stdout),
                    &String::from_utf8_lossy(&output.stderr)));
            }
            Ok(Err(e)) => {
                return Err(CokacError::new(format!("명령 실행 실패: {}", e), line));
            }
            Err(_) => {
                // Timeout — kill the process group
                unsafe { libc::kill(-(pid as i32), libc::SIGKILL); }
                // Reap the thread
                let _ = rx.recv();
                return Ok(make_exec_result(124, "", "시간 제한 초과"));
            }
        }
    }

    let output = child.wait_with_output().map_err(|e| {
        CokacError::new(format!("명령 실행 실패: {}", e), line)
    })?;

    let code = output.status.code().unwrap_or(-1);
    Ok(make_exec_result(code,
        &String::from_utf8_lossy(&output.stdout),
        &String::from_utf8_lossy(&output.stderr)))
}

fn make_exec_result(code: i32, stdout: &str, stderr: &str) -> Value {
    let obj = ObjectValue::new();
    let obj = Rc::new(RefCell::new(obj));
    {
        let mut o = obj.borrow_mut();
        o.set("코드".to_string(), Value::Number(code as f64));
        o.set("성공".to_string(), Value::Bool(code == 0));
        o.set("표준출력".to_string(), Value::String(stdout.to_string()));
        o.set("표준에러".to_string(), Value::String(stderr.to_string()));
    }
    Value::Object(obj)
}

struct ShellOptions {
    stdin_text: Option<String>,
    cwd: Option<String>,
    timeout_sec: Option<u64>,
    env_vars: Option<Vec<(String, String)>>,
}

fn parse_shell_options(opts: Option<&Value>, line: i32) -> Result<ShellOptions, CokacError> {
    let mut result = ShellOptions {
        stdin_text: None,
        cwd: None,
        timeout_sec: None,
        env_vars: None,
    };

    if let Some(Value::Object(obj)) = opts {
        let obj = obj.borrow();
        if let Some(Value::String(s)) = obj.get("입력") {
            result.stdin_text = Some(s.clone());
        }
        if let Some(Value::String(s)) = obj.get("작업디렉토리") {
            result.cwd = Some(s.clone());
        }
        if let Some(val) = obj.get("시간제한초") {
            let n = crate::value::value_to_number(val, line)?;
            result.timeout_sec = Some(n as u64);
        }
        if let Some(Value::Object(env_obj)) = obj.get("환경") {
            let env_obj = env_obj.borrow();
            let mut vars = Vec::new();
            for (k, v) in env_obj.keys.iter().zip(env_obj.values.iter()) {
                vars.push((k.clone(), v.to_display_string()));
            }
            result.env_vars = Some(vars);
        }
    }
    Ok(result)
}

fn apply_shell_options(cmd: &mut Command, opts: &ShellOptions) {
    if let Some(ref dir) = opts.cwd {
        cmd.current_dir(dir);
    }
    if let Some(ref env_vars) = opts.env_vars {
        for (k, v) in env_vars {
            cmd.env(k, v);
        }
    }
}

pub fn builtin_arg_value(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'인수값'은 1개의 인수가 필요합니다.".to_string(), line));
    }
    let idx = crate::value::value_to_number(&args[0], line)? as usize;
    match runtime.script_argv.get(idx) {
        Some(s) => Ok(Value::String(s.clone())),
        None => Ok(Value::Nil),
    }
}

pub fn builtin_arg_count(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if !args.is_empty() {
        return Err(CokacError::new("'인수개수'는 인수가 필요 없습니다.".to_string(), line));
    }
    Ok(Value::Number(runtime.script_argc as f64))
}

pub fn builtin_arg_list(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if !args.is_empty() {
        return Err(CokacError::new("'인수목록'은 인수가 필요 없습니다.".to_string(), line));
    }
    let items: Vec<Value> = runtime.script_argv.iter().map(|s| Value::String(s.clone())).collect();
    Ok(Value::new_array(items))
}

pub fn builtin_exit(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    let code = if args.is_empty() {
        0
    } else {
        crate::value::value_to_number(&args[0], line)? as i32
    };
    use std::io::Write;
    io::stdout().flush().ok();
    io::stderr().flush().ok();
    std::process::exit(code);
}

pub fn builtin_getenv(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'환경'은 1개의 인수가 필요합니다.".to_string(), line));
    }
    let key = args[0].to_display_string();
    match std::env::var(&key) {
        Ok(val) => Ok(Value::String(val)),
        Err(_) => Ok(Value::Nil),
    }
}

pub fn builtin_env_list(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if !args.is_empty() {
        return Err(CokacError::new("'환경목록'은 인수가 필요 없습니다.".to_string(), line));
    }
    let obj = ObjectValue::new();
    let obj = Rc::new(RefCell::new(obj));
    {
        let mut o = obj.borrow_mut();
        for (key, val) in std::env::vars() {
            o.set(key, Value::String(val));
        }
    }
    Ok(Value::Object(obj))
}

use std::io;
