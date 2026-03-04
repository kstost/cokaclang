use std::cell::RefCell;
use std::rc::Rc;
use std::fmt;

use crate::ast::StmtId;
use crate::environment::Environment;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Bool(bool),
    String(String),
    Function(FunctionValue),
    Array(Rc<RefCell<ArrayValue>>),
    Object(Rc<RefCell<ObjectValue>>),
    Task(Rc<RefCell<TaskValue>>),
    Nil,
}

#[derive(Debug, Clone)]
pub struct FunctionValue {
    pub name: String,
    pub params: Vec<String>,
    pub body: StmtId,
    pub is_builtin: bool,
    pub is_async: bool,
    pub arena_index: usize,
    pub closure_env: Option<Rc<RefCell<Environment>>>,
}

#[derive(Debug, Clone)]
pub struct ArrayValue {
    pub items: Vec<Value>,
    pub frozen: bool,
}

#[derive(Debug, Clone)]
pub struct ObjectValue {
    pub keys: Vec<String>,
    pub values: Vec<Value>,
    pub frozen: bool,
}

#[derive(Debug, Clone)]
pub struct TaskValue {
    pub completed: bool,
    pub failed: bool,
    pub cancelled: bool,
    pub result: Option<Value>,
    pub error_message: Option<String>,
    pub error_code: Option<String>,
    pub error_line: i32,
    pub error_stack: Vec<String>,
}

impl TaskValue {
    pub fn new() -> Self {
        TaskValue {
            completed: false,
            failed: false,
            cancelled: false,
            result: None,
            error_message: None,
            error_code: None,
            error_line: 0,
            error_stack: Vec::new(),
        }
    }

    pub fn complete_success(&mut self, value: Value) {
        if !self.completed {
            self.completed = true;
            self.result = Some(value);
        }
    }

    pub fn complete_error(&mut self, message: String, code: String, line: i32, stack: Vec<String>) {
        if !self.completed {
            self.completed = true;
            self.failed = true;
            self.error_message = Some(message);
            self.error_code = Some(code);
            self.error_line = line;
            self.error_stack = stack;
        }
    }

    pub fn cancel(&mut self) -> bool {
        if !self.completed {
            self.completed = true;
            self.cancelled = true;
            self.failed = true;
            self.error_message = Some("작업이 취소되었습니다.".to_string());
            self.error_code = Some("E_TASK_CANCELLED".to_string());
            true
        } else {
            false
        }
    }
}

impl ArrayValue {
    pub fn new() -> Self {
        ArrayValue {
            items: Vec::new(),
            frozen: false,
        }
    }

    pub fn from_items(items: Vec<Value>) -> Self {
        ArrayValue {
            items,
            frozen: false,
        }
    }
}

impl ObjectValue {
    pub fn new() -> Self {
        ObjectValue {
            keys: Vec::new(),
            values: Vec::new(),
            frozen: false,
        }
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        for (i, k) in self.keys.iter().enumerate() {
            if k == key {
                return Some(&self.values[i]);
            }
        }
        None
    }

    pub fn set(&mut self, key: String, value: Value) {
        for (i, k) in self.keys.iter().enumerate() {
            if k == &key {
                self.values[i] = value;
                return;
            }
        }
        self.keys.push(key);
        self.values.push(value);
    }

    pub fn has(&self, key: &str) -> bool {
        self.keys.iter().any(|k| k == key)
    }

    pub fn delete(&mut self, key: &str) -> bool {
        for (i, k) in self.keys.iter().enumerate() {
            if k == key {
                self.keys.remove(i);
                self.values.remove(i);
                return true;
            }
        }
        false
    }

    pub fn count(&self) -> usize {
        self.keys.len()
    }
}

impl Value {
    pub fn number(n: f64) -> Self {
        Value::Number(n)
    }
    pub fn bool_val(b: bool) -> Self {
        Value::Bool(b)
    }
    pub fn string(s: String) -> Self {
        Value::String(s)
    }
    pub fn nil() -> Self {
        Value::Nil
    }
    pub fn new_array(items: Vec<Value>) -> Self {
        Value::Array(Rc::new(RefCell::new(ArrayValue::from_items(items))))
    }
    pub fn new_object() -> Self {
        Value::Object(Rc::new(RefCell::new(ObjectValue::new())))
    }
    pub fn new_task() -> Rc<RefCell<TaskValue>> {
        Rc::new(RefCell::new(TaskValue::new()))
    }
    pub fn make_function(name: String, params: Vec<String>, body: StmtId, is_async: bool) -> Self {
        Value::Function(FunctionValue {
            name,
            params,
            body,
            is_builtin: false,
            is_async,
            arena_index: 0,
            closure_env: None,
        })
    }
    pub fn make_function_with_arena(name: String, params: Vec<String>, body: StmtId, is_async: bool, arena_index: usize) -> Self {
        Value::Function(FunctionValue {
            name,
            params,
            body,
            is_builtin: false,
            is_async,
            arena_index,
            closure_env: None,
        })
    }
    pub fn make_builtin(name: String) -> Self {
        Value::Function(FunctionValue {
            name,
            params: Vec::new(),
            body: 0,
            is_builtin: true,
            is_async: false,
            arena_index: 0,
            closure_env: None,
        })
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Number(n) => n.abs() > 1e-9,
            Value::String(s) => !s.is_empty(),
            Value::Function(_) => true,
            Value::Array(a) => a.borrow().items.len() > 0,
            Value::Object(o) => o.borrow().count() > 0,
            Value::Task(t) => {
                let t = t.borrow();
                t.completed && !t.failed
            }
            Value::Nil => false,
        }
    }

    pub fn equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => (a - b).abs() < 1e-9,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Function(a), Value::Function(b)) => {
                if a.is_builtin && b.is_builtin {
                    a.name == b.name
                } else {
                    a.body == b.body && a.name == b.name
                }
            }
            (Value::Array(a), Value::Array(b)) => {
                let a = a.borrow();
                let b = b.borrow();
                if a.items.len() != b.items.len() {
                    return false;
                }
                a.items.iter().zip(b.items.iter()).all(|(x, y)| x.equals(y))
            }
            (Value::Object(a), Value::Object(b)) => {
                let a = a.borrow();
                let b = b.borrow();
                if a.count() != b.count() {
                    return false;
                }
                for (i, key) in a.keys.iter().enumerate() {
                    match b.get(key) {
                        Some(bv) => {
                            if !a.values[i].equals(bv) {
                                return false;
                            }
                        }
                        None => return false,
                    }
                }
                true
            }
            (Value::Task(a), Value::Task(b)) => {
                let a = a.borrow();
                let b = b.borrow();
                a.completed == b.completed
                    && a.failed == b.failed
                    && match (&a.result, &b.result) {
                        (Some(ra), Some(rb)) => ra.equals(rb),
                        (None, None) => true,
                        _ => false,
                    }
            }
            _ => false,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "숫자",
            Value::Bool(_) => "불리언",
            Value::String(_) => "문자열",
            Value::Function(_) => "함수",
            Value::Array(_) => "배열",
            Value::Object(_) => "객체",
            Value::Task(_) => "작업",
            Value::Nil => "없음",
        }
    }

    pub fn to_display_string(&self) -> String {
        match self {
            Value::Number(n) => format_number(*n),
            Value::Bool(b) => if *b { "참".to_string() } else { "거짓".to_string() },
            Value::String(s) => s.clone(),
            Value::Nil => "nil".to_string(),
            Value::Function(f) => {
                if f.is_builtin {
                    format!("<내장함수 {}>", f.name)
                } else if f.name.is_empty() {
                    "<함수 익명>".to_string()
                } else {
                    format!("<함수 {}>", f.name)
                }
            }
            Value::Task(t) => {
                let t = t.borrow();
                if !t.completed {
                    "<작업 대기>".to_string()
                } else if t.cancelled {
                    "<작업 취소>".to_string()
                } else if t.failed {
                    "<작업 실패>".to_string()
                } else {
                    "<작업 완료>".to_string()
                }
            }
            Value::Array(a) => {
                let a = a.borrow();
                let items: Vec<String> = a.items.iter().map(|v| v.to_display_string()).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Object(o) => {
                let o = o.borrow();
                let entries: Vec<String> = o
                    .keys
                    .iter()
                    .zip(o.values.iter())
                    .map(|(k, v)| format!("{}: {}", k, v.to_display_string()))
                    .collect();
                format!("{{{}}}", entries.join(", "))
            }
        }
    }

    pub fn freeze(&self) {
        match self {
            Value::Array(a) => {
                let mut a = a.borrow_mut();
                a.frozen = true;
                for item in &a.items {
                    item.freeze();
                }
            }
            Value::Object(o) => {
                let mut o = o.borrow_mut();
                o.frozen = true;
                for val in &o.values {
                    val.freeze();
                }
            }
            _ => {}
        }
    }

    pub fn is_frozen(&self) -> bool {
        match self {
            Value::Array(a) => a.borrow().frozen,
            Value::Object(o) => o.borrow().frozen,
            _ => false,
        }
    }

    pub fn deep_copy(&self) -> Value {
        match self {
            Value::Array(a) => {
                let a = a.borrow();
                let items: Vec<Value> = a.items.iter().map(|v| v.deep_copy()).collect();
                let mut arr = ArrayValue::from_items(items);
                arr.frozen = a.frozen;
                Value::Array(Rc::new(RefCell::new(arr)))
            }
            Value::Object(o) => {
                let o = o.borrow();
                let mut obj = ObjectValue::new();
                for (k, v) in o.keys.iter().zip(o.values.iter()) {
                    obj.keys.push(k.clone());
                    obj.values.push(v.deep_copy());
                }
                obj.frozen = o.frozen;
                Value::Object(Rc::new(RefCell::new(obj)))
            }
            other => other.clone(),
        }
    }
}

pub fn format_number(n: f64) -> String {
    if n == n.trunc() && n.abs() < 1e15 && !n.is_nan() && !n.is_infinite() {
        // Integer-like: format without decimal point
        let i = n as i64;
        format!("{}", i)
    } else {
        // Use %.15g-like formatting
        let s = format!("{:.15e}", n);
        // Parse mantissa and exponent
        if let Some(epos) = s.find('e') {
            let mantissa = &s[..epos];
            let exp: i32 = s[epos + 1..].parse().unwrap_or(0);
            // Trim trailing zeros from mantissa
            let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
            if exp == 0 {
                mantissa.to_string()
            } else {
                // Use Rust's default formatting which gives %.15g-like output
                format!("{}", n)
            }
        } else {
            format!("{}", n)
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_display_string())
    }
}

pub fn value_to_number(val: &Value, line: i32) -> Result<f64, crate::error::CokacError> {
    match val {
        Value::Number(n) => Ok(*n),
        Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
        Value::String(s) => {
            s.trim().parse::<f64>().map_err(|_| {
                crate::error::CokacError::new(
                    format!("'{}' 값을 숫자로 변환할 수 없습니다.", s),
                    line,
                )
            })
        }
        _ => Err(crate::error::CokacError::new(
            format!("'{}' 타입을 숫자로 변환할 수 없습니다.", val.type_name()),
            line,
        )),
    }
}

pub fn value_sort_compare(a: &Value, b: &Value) -> Option<std::cmp::Ordering> {
    match (a, b) {
        (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
        (Value::String(a), Value::String(b)) => Some(a.cmp(b)),
        _ => None,
    }
}
