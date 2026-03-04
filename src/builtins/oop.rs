use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::AstArena;
use crate::environment::Environment;
use crate::error::CokacError;
use crate::evaluator::Evaluator;
use crate::value::*;

pub fn builtin_create_class(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() < 2 || args.len() > 3 {
        return Err(CokacError::new("'클래스생성'은 2~3개의 인수가 필요합니다.".to_string(), line));
    }
    let name = args[0].to_display_string();
    let methods = match &args[1] {
        Value::Object(_) => args[1].clone(),
        _ => return Err(CokacError::new("두 번째 인수(메서드)는 객체여야 합니다.".to_string(), line)),
    };
    let parent = if args.len() > 2 {
        match &args[2] {
            Value::Object(o) => {
                if !o.borrow().has("__클래스객체") {
                    return Err(CokacError::new("세 번째 인수는 클래스 객체여야 합니다.".to_string(), line));
                }
                Some(args[2].clone())
            }
            Value::Nil => None,
            _ => return Err(CokacError::new("세 번째 인수는 클래스 객체여야 합니다.".to_string(), line)),
        }
    } else {
        None
    };

    let class_obj = ObjectValue::new();
    let class_obj = Rc::new(RefCell::new(class_obj));
    {
        let mut o = class_obj.borrow_mut();
        o.set("__클래스객체".to_string(), Value::Bool(true));
        o.set("이름".to_string(), Value::String(name));
        o.set("메서드".to_string(), methods);
        match parent {
            Some(p) => o.set("부모".to_string(), p),
            None => o.set("부모".to_string(), Value::Nil),
        }
    }
    Ok(Value::Object(class_obj))
}

pub fn builtin_create_instance(
    args: Vec<Value>,
    eval: &mut Evaluator,
    arena: &AstArena,
    env: &mut Environment,
    line: i32,
) -> Result<Value, CokacError> {
    if args.is_empty() || args.len() > 2 {
        return Err(CokacError::new("'인스턴스생성'은 1~2개의 인수가 필요합니다.".to_string(), line));
    }
    let class = match &args[0] {
        Value::Object(o) => {
            if !o.borrow().has("__클래스객체") {
                return Err(CokacError::new("첫 번째 인수는 클래스 객체여야 합니다.".to_string(), line));
            }
            args[0].clone()
        }
        _ => return Err(CokacError::new("첫 번째 인수는 클래스 객체여야 합니다.".to_string(), line)),
    };

    let init_args = if args.len() > 1 {
        match &args[1] {
            Value::Array(arr) => arr.borrow().items.clone(),
            _ => return Err(CokacError::new("두 번째 인수는 배열이어야 합니다.".to_string(), line)),
        }
    } else {
        Vec::new()
    };

    // Create instance object
    let instance = ObjectValue::new();
    let instance = Rc::new(RefCell::new(instance));
    instance.borrow_mut().set("__클래스".to_string(), class.clone());
    let instance_val = Value::Object(instance.clone());

    // Call 초기화 (init) method if present
    if let Some(init_method) = find_method_in_class(&class, "초기화") {
        let mut call_args = vec![instance_val.clone()];
        call_args.extend(init_args);
        eval.invoke_callable(init_method, call_args, arena, env, line)?;
    }

    Ok(instance_val)
}

pub fn builtin_method_call(
    args: Vec<Value>,
    eval: &mut Evaluator,
    arena: &AstArena,
    env: &mut Environment,
    line: i32,
) -> Result<Value, CokacError> {
    if args.len() < 2 || args.len() > 3 {
        return Err(CokacError::new("'메서드호출'은 2~3개의 인수가 필요합니다.".to_string(), line));
    }

    let instance = &args[0];
    let method_name = args[1].to_display_string();

    let class = match instance {
        Value::Object(obj) => {
            let obj = obj.borrow();
            match obj.get("__클래스") {
                Some(c) => c.clone(),
                None => return Err(CokacError::new(
                    "객체에 '__클래스' 속성이 없습니다. 인스턴스가 아닙니다.".to_string(),
                    line,
                )),
            }
        }
        _ => return Err(CokacError::new("첫 번째 인수는 인스턴스 객체여야 합니다.".to_string(), line)),
    };

    let method = find_method_in_class(&class, &method_name).ok_or_else(|| {
        CokacError::new(format!("메서드 '{}'을(를) 찾을 수 없습니다.", method_name), line)
    })?;

    let call_args_extra = if args.len() > 2 {
        match &args[2] {
            Value::Array(arr) => arr.borrow().items.clone(),
            _ => return Err(CokacError::new("세 번째 인수는 배열이어야 합니다.".to_string(), line)),
        }
    } else {
        Vec::new()
    };

    let mut call_args = vec![instance.clone()];
    call_args.extend(call_args_extra);
    eval.invoke_callable(method, call_args, arena, env, line)
}

pub fn builtin_is_class(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'클래스확인'은 1개의 인수가 필요합니다.".to_string(), line));
    }
    let is_class = match &args[0] {
        Value::Object(obj) => obj.borrow().has("__클래스객체"),
        _ => false,
    };
    Ok(Value::Bool(is_class))
}

pub fn builtin_inherits(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'상속확인'은 2개의 인수가 필요합니다.".to_string(), line));
    }

    let target_class = match &args[0] {
        Value::Object(obj) => {
            let obj = obj.borrow();
            match obj.get("__클래스") {
                Some(c) => c.clone(),
                None => {
                    if obj.has("__클래스객체") {
                        args[0].clone()
                    } else {
                        return Ok(Value::Bool(false));
                    }
                }
            }
        }
        _ => return Ok(Value::Bool(false)),
    };

    let parent_class = match &args[1] {
        Value::Object(obj) => {
            if obj.borrow().has("__클래스객체") {
                args[1].clone()
            } else {
                return Ok(Value::Bool(false));
            }
        }
        _ => return Ok(Value::Bool(false)),
    };

    // Walk up the class chain
    let mut current = Some(target_class);
    while let Some(cls) = current {
        if class_identity_match(&cls, &parent_class) {
            return Ok(Value::Bool(true));
        }
        current = match &cls {
            Value::Object(obj) => {
                let obj = obj.borrow();
                match obj.get("부모") {
                    Some(Value::Object(_)) => Some(obj.get("부모").unwrap().clone()),
                    _ => None,
                }
            }
            _ => None,
        };
    }

    Ok(Value::Bool(false))
}

fn find_method_in_class(class: &Value, method_name: &str) -> Option<Value> {
    let mut current = Some(class.clone());
    while let Some(cls) = current {
        if let Value::Object(obj) = &cls {
            let obj = obj.borrow();
            if let Some(Value::Object(methods)) = obj.get("메서드") {
                let methods = methods.borrow();
                if let Some(method) = methods.get(method_name) {
                    return Some(method.clone());
                }
            }
            // Walk up to parent
            current = match obj.get("부모") {
                Some(p @ Value::Object(_)) => Some(p.clone()),
                _ => None,
            };
        } else {
            break;
        }
    }
    None
}

fn class_identity_match(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Object(a), Value::Object(b)) => Rc::ptr_eq(a, b),
        _ => false,
    }
}
