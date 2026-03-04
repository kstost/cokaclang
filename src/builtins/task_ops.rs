use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::AstArena;
use crate::environment::Environment;
use crate::error::CokacError;
use crate::evaluator::Evaluator;
use crate::runtime::Runtime;
use crate::value::*;

pub fn builtin_task_done(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'작업완료'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    match &args[0] {
        Value::Task(t) => Ok(Value::Bool(t.borrow().completed)),
        _ => Err(CokacError::new("인수가 작업 타입이 아닙니다.".to_string(), line)),
    }
}

pub fn builtin_task_failed(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'작업실패'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    match &args[0] {
        Value::Task(t) => {
            let t = t.borrow();
            Ok(Value::Bool(t.completed && t.failed))
        }
        _ => Err(CokacError::new("인수가 작업 타입이 아닙니다.".to_string(), line)),
    }
}

pub fn builtin_task_error(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'작업오류'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    match &args[0] {
        Value::Task(t) => {
            let t = t.borrow();
            if t.completed && t.failed {
                match &t.error_message {
                    Some(msg) => Ok(Value::String(msg.clone())),
                    None => Ok(Value::Nil),
                }
            } else {
                Ok(Value::Nil)
            }
        }
        _ => Err(CokacError::new("인수가 작업 타입이 아닙니다.".to_string(), line)),
    }
}

pub fn builtin_task_error_code(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'작업오류코드'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    match &args[0] {
        Value::Task(t) => {
            let t = t.borrow();
            if t.completed && t.failed {
                match &t.error_code {
                    Some(code) => Ok(Value::String(code.clone())),
                    None => Ok(Value::Nil),
                }
            } else {
                Ok(Value::Nil)
            }
        }
        _ => Err(CokacError::new("인수가 작업 타입이 아닙니다.".to_string(), line)),
    }
}

pub fn builtin_task_cancel(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'작업취소'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    match &args[0] {
        Value::Task(t) => Ok(Value::Bool(t.borrow_mut().cancel())),
        _ => Err(CokacError::new("인수가 작업 타입이 아닙니다.".to_string(), line)),
    }
}

pub fn builtin_task_state(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'작업상태'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    match &args[0] {
        Value::Task(t) => {
            let t = t.borrow();
            let state = if !t.completed {
                "대기"
            } else if t.cancelled {
                "취소"
            } else if t.failed {
                "실패"
            } else {
                "완료"
            };
            Ok(Value::String(state.to_string()))
        }
        _ => Err(CokacError::new("인수가 작업 타입이 아닙니다.".to_string(), line)),
    }
}

pub fn builtin_task_result(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'작업결과'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    match &args[0] {
        Value::Task(t) => {
            let t = t.borrow();
            if t.completed && !t.failed {
                Ok(t.result.clone().unwrap_or(Value::Nil))
            } else {
                Ok(Value::Nil)
            }
        }
        _ => Err(CokacError::new("인수가 작업 타입이 아닙니다.".to_string(), line)),
    }
}

pub fn builtin_task_all(args: Vec<Value>, eval: &mut Evaluator, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'작업모두'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    match &args[0] {
        Value::Array(arr) => {
            let arr = arr.borrow();
            let mut deps = Vec::new();
            for item in &arr.items {
                match item {
                    Value::Task(t) => deps.push(t.clone()),
                    _ => return Err(CokacError::new("'작업모두' 배열의 요소는 작업이어야 합니다.".to_string(), line)),
                }
            }
            let task = Value::new_task();
            if !eval.runtime.try_enqueue_task(&task, line) {
                return Ok(Value::Task(task));
            }
            eval.runtime.async_jobs.push(crate::runtime::AsyncJob {
                task: task.clone(),
                kind: crate::runtime::AsyncJobKind::All { deps },
            });
            eval.runtime.mark_async_enqueued();
            Ok(Value::Task(task))
        }
        _ => Err(CokacError::new("'작업모두'의 인수는 배열이어야 합니다.".to_string(), line)),
    }
}

pub fn builtin_task_race(args: Vec<Value>, eval: &mut Evaluator, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'작업경주'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    match &args[0] {
        Value::Array(arr) => {
            let arr = arr.borrow();
            let mut deps = Vec::new();
            for item in &arr.items {
                match item {
                    Value::Task(t) => deps.push(t.clone()),
                    _ => return Err(CokacError::new("'작업경주' 배열의 요소는 작업이어야 합니다.".to_string(), line)),
                }
            }
            let task = Value::new_task();
            if !eval.runtime.try_enqueue_task(&task, line) {
                return Ok(Value::Task(task));
            }
            eval.runtime.async_jobs.push(crate::runtime::AsyncJob {
                task: task.clone(),
                kind: crate::runtime::AsyncJobKind::Race { deps },
            });
            eval.runtime.mark_async_enqueued();
            Ok(Value::Task(task))
        }
        _ => Err(CokacError::new("'작업경주'의 인수는 배열이어야 합니다.".to_string(), line)),
    }
}

pub fn builtin_await_timeout(
    args: Vec<Value>,
    eval: &mut Evaluator,
    arena: &AstArena,
    env: &mut Environment,
    line: i32,
) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'대기최대'는 2개의 인수가 필요합니다.".to_string(), line));
    }
    let task = match &args[0] {
        Value::Task(t) => t.clone(),
        _ => return Err(CokacError::new("첫 번째 인수가 작업 타입이 아닙니다.".to_string(), line)),
    };
    let timeout_ms = crate::value::value_to_number(&args[1], line)? as u64;

    #[cfg(not(target_arch = "wasm32"))]
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
    #[cfg(target_arch = "wasm32")]
    let deadline_ms = js_sys::Date::now() + timeout_ms as f64;

    loop {
        {
            let t = task.borrow();
            if t.completed {
                if t.failed {
                    let msg = t.error_message.clone().unwrap_or_else(|| "작업 실패".to_string());
                    return Err(CokacError::new(msg, line));
                }
                return Ok(t.result.clone().unwrap_or(Value::Nil));
            }
        }
        eval.runtime.async_wait_calls += 1;
        #[cfg(not(target_arch = "wasm32"))]
        {
            if std::time::Instant::now() >= deadline {
                eval.runtime.async_wait_timeouts += 1;
                return Err(CokacError::new(
                    format!("시간 초과: {}ms 내에 작업이 완료되지 않았습니다.", timeout_ms),
                    line,
                ));
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            if js_sys::Date::now() >= deadline_ms {
                eval.runtime.async_wait_timeouts += 1;
                return Err(CokacError::new(
                    format!("시간 초과: {}ms 내에 작업이 완료되지 않았습니다.", timeout_ms),
                    line,
                ));
            }
        }
        eval.drive_async(arena, env)?;
        // drive_async can block while running a job; enforce timeout even if the task completed late.
        #[cfg(not(target_arch = "wasm32"))]
        {
            if std::time::Instant::now() >= deadline {
                eval.runtime.async_wait_timeouts += 1;
                let t = task.borrow();
                if !t.completed {
                    return Err(CokacError::new(
                        format!("시간 초과: {}ms 내에 작업이 완료되지 않았습니다.", timeout_ms),
                        line,
                    ));
                }
                return Err(CokacError::new(
                    format!("시간 초과: {}ms 제한을 초과해 작업이 완료되었습니다.", timeout_ms),
                    line,
                ));
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            if js_sys::Date::now() >= deadline_ms {
                eval.runtime.async_wait_timeouts += 1;
                let t = task.borrow();
                if !t.completed {
                    return Err(CokacError::new(
                        format!("시간 초과: {}ms 내에 작업이 완료되지 않았습니다.", timeout_ms),
                        line,
                    ));
                }
                return Err(CokacError::new(
                    format!("시간 초과: {}ms 제한을 초과해 작업이 완료되었습니다.", timeout_ms),
                    line,
                ));
            }
        }
    }
}

pub fn builtin_async_queue_length(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if !args.is_empty() {
        return Err(CokacError::new("'비동기큐길이'는 인수가 필요 없습니다.".to_string(), line));
    }
    Ok(Value::Number(runtime.async_jobs.len() as f64))
}

pub fn builtin_async_stats(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if !args.is_empty() {
        return Err(CokacError::new("'비동기통계'는 인수가 필요 없습니다.".to_string(), line));
    }
    let obj = ObjectValue::new();
    let obj = Rc::new(RefCell::new(obj));
    {
        let mut o = obj.borrow_mut();
        o.set("큐길이".to_string(), Value::Number(runtime.async_jobs.len() as f64));
        o.set("최대큐길이".to_string(), Value::Number(runtime.async_max_queue as f64));
        o.set("누적등록".to_string(), Value::Number(runtime.async_enqueued as f64));
        o.set("누적완료".to_string(), Value::Number(runtime.async_completed as f64));
        o.set("누적실패".to_string(), Value::Number(runtime.async_failed as f64));
        o.set("누적취소".to_string(), Value::Number(runtime.async_cancelled as f64));
        o.set("누적재큐".to_string(), Value::Number(runtime.async_requeued as f64));
        o.set("누적백프레셔".to_string(), Value::Number(runtime.async_backpressure as f64));
        o.set("루프틱".to_string(), Value::Number(runtime.async_loop_ticks as f64));
        o.set("대기호출".to_string(), Value::Number(runtime.async_wait_calls as f64));
        o.set("대기타임아웃".to_string(), Value::Number(runtime.async_wait_timeouts as f64));
    }
    Ok(Value::Object(obj))
}
