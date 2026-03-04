use std::collections::HashMap;
use crate::value::Value;

#[derive(Debug, Clone)]
struct Variable {
    value: Value,
    is_const: bool,
}

#[derive(Debug)]
pub struct Environment {
    vars: HashMap<String, Variable>,
    pub parent: Option<Box<Environment>>,
    pub loop_depth: i32,
    pub function_depth: i32,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            vars: HashMap::new(),
            parent: None,
            loop_depth: 0,
            function_depth: 0,
        }
    }

    pub fn with_parent(parent: Environment) -> Self {
        Environment {
            vars: HashMap::new(),
            parent: Some(Box::new(parent)),
            loop_depth: 0,
            function_depth: 0,
        }
    }

    pub fn define(&mut self, name: String, value: Value, is_const: bool) -> Result<(), String> {
        if self.vars.contains_key(&name) {
            return Err(format!("변수 '{}'이(가) 이미 정의되어 있습니다.", name));
        }
        if is_const {
            value.freeze();
        }
        self.vars.insert(name, Variable { value, is_const });
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(var) = self.vars.get(name) {
            return Some(var.value.clone());
        }
        if let Some(ref parent) = self.parent {
            return parent.get(name);
        }
        None
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<bool, String> {
        if let Some(var) = self.vars.get_mut(name) {
            if var.is_const {
                return Err(format!("상수 '{}'에 값을 재할당할 수 없습니다.", name));
            }
            var.value = value;
            return Ok(true);
        }
        if let Some(ref mut parent) = self.parent {
            return parent.assign(name, value);
        }
        Ok(false) // not found
    }

    /// Take this environment's parent, leaving None in its place
    pub fn take_parent(&mut self) -> Option<Environment> {
        self.parent.take().map(|b| *b)
    }
}

impl Clone for Environment {
    fn clone(&self) -> Self {
        Environment {
            vars: self.vars.clone(),
            parent: self.parent.clone(),
            loop_depth: self.loop_depth,
            function_depth: self.function_depth,
        }
    }
}
