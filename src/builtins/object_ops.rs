use crate::error::CokacError;
use crate::value::*;
use std::cell::RefCell;
use std::rc::Rc;

pub fn builtin_has(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'객체가짐'은 2개의 인수가 필요합니다.".to_string(), line));
    }
    match &args[0] {
        Value::Object(obj) => {
            let key = args[1].to_display_string();
            Ok(Value::Bool(obj.borrow().has(&key)))
        }
        _ => Err(CokacError::new("'객체가짐'의 첫 번째 인수는 객체여야 합니다.".to_string(), line)),
    }
}

pub fn builtin_set(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 3 {
        return Err(CokacError::new("'객체설정'은 3개의 인수가 필요합니다.".to_string(), line));
    }
    match &args[0] {
        Value::Object(obj) => {
            let mut obj = obj.borrow_mut();
            if obj.frozen {
                return Err(CokacError::new("불변 객체를 수정할 수 없습니다.".to_string(), line));
            }
            let key = args[1].to_display_string();
            obj.set(key, args[2].clone());
            Ok(args[2].clone())
        }
        _ => Err(CokacError::new("'객체설정'의 첫 번째 인수는 객체여야 합니다.".to_string(), line)),
    }
}

pub fn builtin_delete(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'객체삭제'는 2개의 인수가 필요합니다.".to_string(), line));
    }
    match &args[0] {
        Value::Object(obj) => {
            let mut obj = obj.borrow_mut();
            if obj.frozen {
                return Err(CokacError::new("불변 객체를 수정할 수 없습니다.".to_string(), line));
            }
            let key = args[1].to_display_string();
            Ok(Value::Bool(obj.delete(&key)))
        }
        _ => Err(CokacError::new("'객체삭제'의 첫 번째 인수는 객체여야 합니다.".to_string(), line)),
    }
}

pub fn builtin_keys(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'객체키들'은 1개의 인수가 필요합니다.".to_string(), line));
    }
    match &args[0] {
        Value::Object(obj) => {
            let obj = obj.borrow();
            let keys: Vec<Value> = obj.keys.iter().map(|k| Value::String(k.clone())).collect();
            Ok(Value::new_array(keys))
        }
        _ => Err(CokacError::new("'객체키들'의 인수는 객체여야 합니다.".to_string(), line)),
    }
}

pub fn builtin_values(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'객체값들'은 1개의 인수가 필요합니다.".to_string(), line));
    }
    match &args[0] {
        Value::Object(obj) => {
            let obj = obj.borrow();
            let vals: Vec<Value> = obj.values.clone();
            Ok(Value::new_array(vals))
        }
        _ => Err(CokacError::new("'객체값들'의 인수는 객체여야 합니다.".to_string(), line)),
    }
}

pub fn builtin_clone(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'객체복사'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    match &args[0] {
        Value::Object(obj) => {
            let obj = obj.borrow();
            let mut new_obj = ObjectValue::new();
            for (k, v) in obj.keys.iter().zip(obj.values.iter()) {
                new_obj.keys.push(k.clone());
                new_obj.values.push(v.clone());
            }
            Ok(Value::Object(Rc::new(RefCell::new(new_obj))))
        }
        _ => Err(CokacError::new("'객체복사'의 인수는 객체여야 합니다.".to_string(), line)),
    }
}

pub fn builtin_merge(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'객체합치기'는 2개의 인수가 필요합니다.".to_string(), line));
    }
    match (&args[0], &args[1]) {
        (Value::Object(target), Value::Object(source)) => {
            let mut target = target.borrow_mut();
            if target.frozen {
                return Err(CokacError::new("불변 객체를 수정할 수 없습니다.".to_string(), line));
            }
            let source = source.borrow();
            for (k, v) in source.keys.iter().zip(source.values.iter()) {
                target.set(k.clone(), v.clone());
            }
            drop(target);
            drop(source);
            Ok(args[0].clone())
        }
        _ => Err(CokacError::new("'객체합치기'의 인수들은 객체여야 합니다.".to_string(), line)),
    }
}
