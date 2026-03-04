use crate::value::Value;

#[derive(Debug, Clone)]
struct Variable {
    value: Value,
    is_const: bool,
}

#[derive(Debug)]
pub struct Environment {
    vars: Vec<(String, Variable)>,
    pub parent: Option<Box<Environment>>,
    pub loop_depth: i32,
    pub function_depth: i32,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            vars: Vec::new(),
            parent: None,
            loop_depth: 0,
            function_depth: 0,
        }
    }

    pub fn with_parent(parent: Environment) -> Self {
        Environment {
            vars: Vec::new(),
            parent: Some(Box::new(parent)),
            loop_depth: 0,
            function_depth: 0,
        }
    }

    pub fn for_function(parent: Environment, capacity: usize) -> Self {
        Environment {
            vars: Vec::with_capacity(capacity),
            parent: Some(Box::new(parent)),
            loop_depth: 0,
            function_depth: 0,
        }
    }

    /// Set up this environment for a non-closure function call (in-place, zero-alloc with pool).
    /// Saves current state into a pooled Box as the parent scope.
    pub fn prepare_call(&mut self, capacity: usize, pool: &mut Vec<Box<Environment>>) {
        let mut parent_box = pool.pop()
            .unwrap_or_else(|| Box::new(Environment::new()));
        // Move current state into the parent box, get the pooled env's state
        std::mem::swap(self, &mut *parent_box);
        // self now has the pooled env's (cleared) vars Vec; parent_box has caller's state
        self.vars.clear();
        if capacity > self.vars.capacity() {
            self.vars.reserve(capacity);
        }
        self.parent = Some(parent_box);
        self.loop_depth = 0;
        self.function_depth = 0;
    }

    /// Restore after a non-closure function call. Returns caller's state from parent.
    /// Drains param name strings to the name pool for reuse.
    pub fn finish_call(&mut self, pool: &mut Vec<Box<Environment>>, name_pool: &mut Vec<String>) {
        if let Some(mut parent_box) = self.parent.take() {
            // Swap caller's state back into self
            std::mem::swap(self, &mut *parent_box);
            // Rescue param name strings before dropping
            for (name, _) in parent_box.vars.drain(..) {
                name_pool.push(name);
            }
            parent_box.parent = None;
            parent_box.loop_depth = 0;
            parent_box.function_depth = 0;
            pool.push(parent_box);
        }
    }

    /// Create a function environment using a pooled Box for the parent.
    /// Used for closure calls where we need a separate env (not in-place).
    pub fn for_function_pooled(parent: Environment, capacity: usize, pool: &mut Vec<Box<Environment>>) -> Self {
        let mut parent_box = pool.pop()
            .unwrap_or_else(|| Box::new(Environment::new()));
        *parent_box = parent;
        Environment {
            vars: Vec::with_capacity(capacity),
            parent: Some(parent_box),
            loop_depth: 0,
            function_depth: 0,
        }
    }

    /// Take parent and return the Box to the pool. Drains param names for reuse.
    pub fn take_parent_pooled(&mut self, pool: &mut Vec<Box<Environment>>, name_pool: &mut Vec<String>) -> Option<Environment> {
        // Rescue param name strings before this env is dropped
        for (name, _) in self.vars.drain(..) {
            name_pool.push(name);
        }
        self.parent.take().map(|mut b| {
            let env = std::mem::replace(&mut *b, Environment::new());
            pool.push(b);
            env
        })
    }

    pub fn define(&mut self, name: String, value: Value, is_const: bool) -> Result<(), String> {
        for (k, _) in &self.vars {
            if k == &name {
                return Err(format!("변수 '{}'이(가) 이미 정의되어 있습니다.", name));
            }
        }
        if is_const {
            value.freeze();
        }
        self.vars.push((name, Variable { value, is_const }));
        Ok(())
    }

    /// Fast path for function parameters — no duplicate check needed.
    #[inline(always)]
    pub fn define_param(&mut self, name: String, value: Value) {
        self.vars.push((name, Variable { value, is_const: false }));
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        for (k, var) in self.vars.iter().rev() {
            if k == name {
                return Some(var.value.clone());
            }
        }
        if let Some(ref parent) = self.parent {
            return parent.get(name);
        }
        None
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<bool, String> {
        for (k, var) in self.vars.iter_mut().rev() {
            if k == name {
                if var.is_const {
                    return Err(format!("상수 '{}'에 값을 재할당할 수 없습니다.", name));
                }
                var.value = value;
                return Ok(true);
            }
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
