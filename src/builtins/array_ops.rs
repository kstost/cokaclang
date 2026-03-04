use crate::ast::AstArena;
use crate::environment::Environment;
use crate::error::CokacError;
use crate::evaluator::{Evaluator, value_to_index};
use crate::value::*;

pub fn builtin_push(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'배열추가'는 2개의 인수가 필요합니다.".to_string(), line));
    }
    match &args[0] {
        Value::Array(arr) => {
            let mut arr = arr.borrow_mut();
            if arr.frozen {
                return Err(CokacError::new("불변 배열을 수정할 수 없습니다.".to_string(), line));
            }
            arr.items.push(args[1].clone());
            Ok(Value::Number(arr.items.len() as f64))
        }
        _ => Err(CokacError::new("'배열추가'의 첫 번째 인수는 배열이어야 합니다.".to_string(), line)),
    }
}

pub fn builtin_insert(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 3 {
        return Err(CokacError::new("'배열삽입'은 3개의 인수가 필요합니다.".to_string(), line));
    }
    match &args[0] {
        Value::Array(arr) => {
            let mut arr = arr.borrow_mut();
            if arr.frozen {
                return Err(CokacError::new("불변 배열을 수정할 수 없습니다.".to_string(), line));
            }
            let idx = value_to_index(&args[1], arr.items.len() + 1, true, line)?;
            arr.items.insert(idx, args[2].clone());
            Ok(Value::Bool(true))
        }
        _ => Err(CokacError::new("'배열삽입'의 첫 번째 인수는 배열이어야 합니다.".to_string(), line)),
    }
}

pub fn builtin_remove(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'배열삭제'는 2개의 인수가 필요합니다.".to_string(), line));
    }
    match &args[0] {
        Value::Array(arr) => {
            let mut arr = arr.borrow_mut();
            if arr.frozen {
                return Err(CokacError::new("불변 배열을 수정할 수 없습니다.".to_string(), line));
            }
            if arr.items.is_empty() {
                return Err(CokacError::new("빈 배열에서 삭제할 수 없습니다.".to_string(), line));
            }
            let idx = value_to_index(&args[1], arr.items.len(), false, line)?;
            let removed = arr.items.remove(idx);
            Ok(removed)
        }
        _ => Err(CokacError::new("'배열삭제'의 첫 번째 인수는 배열이어야 합니다.".to_string(), line)),
    }
}

pub fn builtin_pop(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'배열꺼내기'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    match &args[0] {
        Value::Array(arr) => {
            let mut arr = arr.borrow_mut();
            if arr.frozen {
                return Err(CokacError::new("불변 배열을 수정할 수 없습니다.".to_string(), line));
            }
            arr.items.pop().ok_or_else(|| {
                CokacError::new("빈 배열에서 꺼낼 수 없습니다.".to_string(), line)
            })
        }
        _ => Err(CokacError::new("'배열꺼내기'의 인수는 배열이어야 합니다.".to_string(), line)),
    }
}

pub fn builtin_slice(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 3 {
        return Err(CokacError::new("'배열슬라이스'는 3개의 인수가 필요합니다.".to_string(), line));
    }
    match &args[0] {
        Value::Array(arr) => {
            let arr = arr.borrow();
            let len = arr.items.len();
            let start = value_to_index(&args[1], len + 1, true, line)?;
            let end = value_to_index(&args[2], len + 1, true, line)?;
            if start > end {
                return Err(CokacError::new("슬라이스 시작이 끝보다 클 수 없습니다.".to_string(), line));
            }
            let items: Vec<Value> = arr.items[start..end].to_vec();
            Ok(Value::new_array(items))
        }
        _ => Err(CokacError::new("'배열슬라이스'의 첫 번째 인수는 배열이어야 합니다.".to_string(), line)),
    }
}

pub fn builtin_concat(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'배열합치기'는 2개의 인수가 필요합니다.".to_string(), line));
    }
    match (&args[0], &args[1]) {
        (Value::Array(a), Value::Array(b)) => {
            let a = a.borrow();
            let b = b.borrow();
            let mut items = a.items.clone();
            items.extend(b.items.clone());
            Ok(Value::new_array(items))
        }
        _ => Err(CokacError::new("'배열합치기'의 인수들은 배열이어야 합니다.".to_string(), line)),
    }
}

pub fn builtin_sort(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'배열정렬'은 1개의 인수가 필요합니다.".to_string(), line));
    }
    match &args[0] {
        Value::Array(arr) => {
            let mut arr = arr.borrow_mut();
            if arr.frozen {
                return Err(CokacError::new("불변 배열을 정렬할 수 없습니다.".to_string(), line));
            }
            // Selection sort to match C behavior
            let n = arr.items.len();
            for i in 0..n {
                for j in (i + 1)..n {
                    let cmp = value_sort_compare(&arr.items[i], &arr.items[j]);
                    match cmp {
                        Some(std::cmp::Ordering::Greater) => {
                            arr.items.swap(i, j);
                        }
                        None => {
                            return Err(CokacError::new(
                                "정렬할 수 없는 타입이 포함되어 있습니다.".to_string(),
                                line,
                            ));
                        }
                        _ => {}
                    }
                }
            }
            drop(arr);
            Ok(args[0].clone())
        }
        _ => Err(CokacError::new("'배열정렬'의 인수는 배열이어야 합니다.".to_string(), line)),
    }
}

pub fn builtin_join(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'배열문자열합치기'는 2개의 인수가 필요합니다.".to_string(), line));
    }
    match &args[0] {
        Value::Array(arr) => {
            let arr = arr.borrow();
            let delimiter = args[1].to_display_string();
            let parts: Vec<String> = arr.items.iter().map(|v| v.to_display_string()).collect();
            Ok(Value::String(parts.join(&delimiter)))
        }
        _ => Err(CokacError::new("'배열문자열합치기'의 첫 번째 인수는 배열이어야 합니다.".to_string(), line)),
    }
}

pub fn builtin_map(
    args: Vec<Value>,
    eval: &mut Evaluator,
    arena: &AstArena,
    env: &mut Environment,
    line: i32,
) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'배열맵'은 2개의 인수가 필요합니다.".to_string(), line));
    }
    let items = match &args[0] {
        Value::Array(arr) => arr.borrow().items.clone(),
        _ => return Err(CokacError::new("'배열맵'의 첫 번째 인수는 배열이어야 합니다.".to_string(), line)),
    };
    let callback = args[1].clone();
    let mut result = Vec::with_capacity(items.len());
    for (i, item) in items.into_iter().enumerate() {
        let val = eval.invoke_callable(
            callback.clone(),
            vec![item, Value::Number(i as f64)],
            arena,
            env,
            line,
        )?;
        result.push(val);
    }
    Ok(Value::new_array(result))
}

pub fn builtin_filter(
    args: Vec<Value>,
    eval: &mut Evaluator,
    arena: &AstArena,
    env: &mut Environment,
    line: i32,
) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'배열필터'는 2개의 인수가 필요합니다.".to_string(), line));
    }
    let items = match &args[0] {
        Value::Array(arr) => arr.borrow().items.clone(),
        _ => return Err(CokacError::new("'배열필터'의 첫 번째 인수는 배열이어야 합니다.".to_string(), line)),
    };
    let callback = args[1].clone();
    let mut result = Vec::new();
    for (i, item) in items.into_iter().enumerate() {
        let val = eval.invoke_callable(
            callback.clone(),
            vec![item.clone(), Value::Number(i as f64)],
            arena,
            env,
            line,
        )?;
        if val.is_truthy() {
            result.push(item);
        }
    }
    Ok(Value::new_array(result))
}

pub fn builtin_reduce(
    args: Vec<Value>,
    eval: &mut Evaluator,
    arena: &AstArena,
    env: &mut Environment,
    line: i32,
) -> Result<Value, CokacError> {
    if args.len() != 3 {
        return Err(CokacError::new("'배열리듀스'는 3개의 인수가 필요합니다.".to_string(), line));
    }
    let items = match &args[0] {
        Value::Array(arr) => arr.borrow().items.clone(),
        _ => return Err(CokacError::new("'배열리듀스'의 첫 번째 인수는 배열이어야 합니다.".to_string(), line)),
    };
    let callback = args[1].clone();
    let mut acc = args[2].clone();
    for (i, item) in items.into_iter().enumerate() {
        acc = eval.invoke_callable(
            callback.clone(),
            vec![acc, item, Value::Number(i as f64)],
            arena,
            env,
            line,
        )?;
    }
    Ok(acc)
}
