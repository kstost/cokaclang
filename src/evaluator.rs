use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::*;
use crate::environment::Environment;
use crate::error::CokacError;
use crate::runtime::Runtime;
use crate::token::TokenType;
use crate::value::*;
use crate::builtins;
use crate::lexer;
use crate::parser::Parser;

#[derive(Debug, Clone)]
pub enum ExecSignal {
    Normal,
    Break,
    Continue,
    Return(Value),
}

pub struct Evaluator<'a> {
    pub runtime: &'a mut Runtime,
    stmt_depth: usize,
    expr_depth: usize,
    max_stmt_depth: usize,
    max_expr_depth: usize,
}

impl<'a> Evaluator<'a> {
    pub fn new(runtime: &'a mut Runtime) -> Self {
        Evaluator {
            runtime,
            stmt_depth: 0,
            expr_depth: 0,
            max_stmt_depth: read_depth_limit("COKAC_MAX_EVAL_STMT_DEPTH", 2048),
            max_expr_depth: read_depth_limit("COKAC_MAX_EVAL_EXPR_DEPTH", 4096),
        }
    }

    fn resolve_arena_index_for(&self, arena: &AstArena) -> usize {
        if self.runtime.current_arena_index > 0 {
            let idx = self.runtime.current_arena_index;
            if let Some(cur) = self.runtime.loaded_arenas.get(idx - 1) {
                if std::ptr::eq(cur.as_ref(), arena) {
                    return idx;
                }
            }
        }
        for (i, a) in self.runtime.loaded_arenas.iter().enumerate() {
            if std::ptr::eq(a.as_ref(), arena) {
                return i + 1;
            }
        }
        0
    }

    fn exec_function_body_with_arena(
        &mut self,
        func_body: StmtId,
        func_arena_index: usize,
        main_arena: &AstArena,
        env: &mut Environment,
        line: i32,
    ) -> Result<ExecSignal, CokacError> {
        if func_arena_index == 0 {
            if func_body >= main_arena.stmts.len() {
                return Err(CokacError::new(
                    "함수 본문 인덱스가 유효하지 않습니다. (main arena)".to_string(),
                    line,
                ));
            }
            return self.exec_stmt(func_body, main_arena, env);
        }
        let idx = func_arena_index - 1;
        let module_arena = self.runtime.loaded_arenas.get(idx).cloned().ok_or_else(|| {
            CokacError::new(
                format!("함수 arena 인덱스가 유효하지 않습니다: {}", func_arena_index),
                line,
            )
        })?;
        if func_body >= module_arena.stmts.len() {
            return Err(CokacError::new(
                "함수 본문 인덱스가 유효하지 않습니다. (module arena)".to_string(),
                line,
            ));
        }
        self.exec_stmt(func_body, &module_arena, env)
    }

    pub fn exec_stmts(
        &mut self,
        stmts: &[StmtId],
        arena: &AstArena,
        env: &mut Environment,
    ) -> Result<ExecSignal, CokacError> {
        for &stmt_id in stmts {
            let signal = self.exec_stmt(stmt_id, arena, env)?;
            match signal {
                ExecSignal::Normal => {}
                other => return Ok(other),
            }
        }
        Ok(ExecSignal::Normal)
    }

    pub fn exec_stmt(
        &mut self,
        stmt_id: StmtId,
        arena: &AstArena,
        env: &mut Environment,
    ) -> Result<ExecSignal, CokacError> {
        if stmt_id >= arena.stmts.len() {
            return Err(CokacError::new(
                format!(
                    "내부 오류: 유효하지 않은 문장 인덱스 {} (문장 수: {})",
                    stmt_id,
                    arena.stmts.len()
                ),
                0,
            ));
        }
        let stmt = arena.get_stmt(stmt_id).clone();
        self.stmt_depth += 1;
        if self.stmt_depth > self.max_stmt_depth {
            self.stmt_depth -= 1;
            return Err(CokacError::new(
                format!(
                    "실행 깊이 제한({})을 초과했습니다. 재귀 또는 중첩 블록을 줄이거나 COKAC_MAX_EVAL_STMT_DEPTH 값을 조정하세요.",
                    self.max_stmt_depth
                ),
                stmt.line,
            ));
        }
        let _stmt_depth_guard = DepthGuard::new(&mut self.stmt_depth);

        match stmt.kind {
            StmtKind::Let { ref name, initializer, is_const } => {
                let val = self.eval_expr(initializer, arena, env)?;
                env.define(name.clone(), val, is_const).map_err(|msg| {
                    CokacError::new(msg, stmt.line)
                })?;
                Ok(ExecSignal::Normal)
            }
            StmtKind::Assign { ref name, value } => {
                let val = self.eval_expr(value, arena, env)?;
                match env.assign(name, val) {
                    Ok(true) => Ok(ExecSignal::Normal),
                    Ok(false) => Err(CokacError::new(
                        format!("정의되지 않은 변수: '{}'", name),
                        stmt.line,
                    )),
                    Err(msg) => Err(CokacError::new(msg, stmt.line)),
                }
            }
            StmtKind::IndexAssign { target, index, value } => {
                let val = self.eval_expr(value, arena, env)?;
                let target_val = self.eval_expr(target, arena, env)?;
                let idx_val = self.eval_expr(index, arena, env)?;
                match target_val {
                    Value::Array(arr) => {
                        let mut arr = arr.borrow_mut();
                        if arr.frozen {
                            return Err(CokacError::new(
                                "불변 배열에 값을 설정할 수 없습니다.".to_string(),
                                stmt.line,
                            ));
                        }
                        let idx = value_to_index(&idx_val, arr.items.len(), false, stmt.line)?;
                        arr.items[idx] = val;
                    }
                    Value::Object(obj) => {
                        let mut obj = obj.borrow_mut();
                        if obj.frozen {
                            return Err(CokacError::new(
                                "불변 객체에 값을 설정할 수 없습니다.".to_string(),
                                stmt.line,
                            ));
                        }
                        let key = match idx_val {
                            Value::String(s) => s,
                            _ => idx_val.to_display_string(),
                        };
                        obj.set(key, val);
                    }
                    _ => {
                        return Err(CokacError::new(
                            "인덱스 대입은 배열 또는 객체에만 가능합니다.".to_string(),
                            stmt.line,
                        ));
                    }
                }
                Ok(ExecSignal::Normal)
            }
            StmtKind::PropertyAssign { target, ref name, value } => {
                let val = self.eval_expr(value, arena, env)?;
                let target_val = self.eval_expr(target, arena, env)?;
                match target_val {
                    Value::Object(obj) => {
                        let mut obj = obj.borrow_mut();
                        if obj.frozen {
                            return Err(CokacError::new(
                                "불변 객체에 값을 설정할 수 없습니다.".to_string(),
                                stmt.line,
                            ));
                        }
                        obj.set(name.clone(), val);
                    }
                    _ => {
                        return Err(CokacError::new(
                            format!("'{}' 속성을 설정할 수 없습니다. 객체가 아닙니다.", name),
                            stmt.line,
                        ));
                    }
                }
                Ok(ExecSignal::Normal)
            }
            StmtKind::Print(expr) => {
                let val = self.eval_expr(expr, arena, env)?;
                println!("{}", val.to_display_string());
                Ok(ExecSignal::Normal)
            }
            StmtKind::If { condition, then_branch, else_branch } => {
                let cond = self.eval_expr(condition, arena, env)?;
                if cond.is_truthy() {
                    self.exec_stmt(then_branch, arena, env)
                } else if let Some(else_b) = else_branch {
                    self.exec_stmt(else_b, arena, env)
                } else {
                    Ok(ExecSignal::Normal)
                }
            }
            StmtKind::While { condition, body } => {
                env.loop_depth += 1;
                loop {
                    let cond = self.eval_expr(condition, arena, env)?;
                    if !cond.is_truthy() {
                        break;
                    }
                    let signal = self.exec_stmt(body, arena, env)?;
                    match signal {
                        ExecSignal::Break => break,
                        ExecSignal::Continue => continue,
                        ExecSignal::Return(_) => {
                            env.loop_depth -= 1;
                            return Ok(signal);
                        }
                        ExecSignal::Normal => {}
                    }
                }
                env.loop_depth -= 1;
                Ok(ExecSignal::Normal)
            }
            StmtKind::For { initializer, condition, increment, body } => {
                if let Some(init) = initializer {
                    self.exec_stmt(init, arena, env)?;
                }
                env.loop_depth += 1;
                loop {
                    if let Some(cond) = condition {
                        let c = self.eval_expr(cond, arena, env)?;
                        if !c.is_truthy() {
                            break;
                        }
                    }
                    let signal = self.exec_stmt(body, arena, env)?;
                    match signal {
                        ExecSignal::Break => break,
                        ExecSignal::Return(_) => {
                            env.loop_depth -= 1;
                            return Ok(signal);
                        }
                        ExecSignal::Continue | ExecSignal::Normal => {}
                    }
                    if let Some(inc) = increment {
                        self.exec_stmt(inc, arena, env)?;
                    }
                }
                env.loop_depth -= 1;
                Ok(ExecSignal::Normal)
            }
            StmtKind::Function { ref name, ref params, body, is_async } => {
                let arena_index = self.resolve_arena_index_for(arena);
                self.runtime.register_function(
                    name.clone(),
                    params.clone(),
                    body,
                    is_async,
                    arena_index,
                );
                // Capture lexical environment for nested functions, and for module top-level
                // declarations while a module is being executed.
                let closure_env = if env.parent.is_some()
                    || self.runtime.current_exports.is_some()
                    || self.runtime.current_arena_index == 1
                {
                    Some(Rc::new(RefCell::new(env.clone())))
                } else {
                    None
                };
                let fn_val = Value::Function(FunctionValue {
                    name: name.clone(),
                    params: params.clone(),
                    body,
                    is_builtin: false,
                    is_async,
                    arena_index,
                    closure_env,
                });
                match env.assign(name, fn_val.clone()) {
                    Ok(true) => {}
                    Ok(false) => {
                        env.define(name.clone(), fn_val, false).map_err(|msg| {
                            CokacError::new(msg, stmt.line)
                        })?;
                    }
                    Err(msg) => return Err(CokacError::new(msg, stmt.line)),
                }
                Ok(ExecSignal::Normal)
            }
            StmtKind::Return { value } => {
                let val = if let Some(expr) = value {
                    self.eval_expr(expr, arena, env)?
                } else {
                    Value::Nil
                };
                Ok(ExecSignal::Return(val))
            }
            StmtKind::Import { ref path, ref alias } => {
                self.exec_import(path, alias.as_deref(), arena, env, stmt.line)
            }
            StmtKind::Try { try_block, catch_block, ref error_name, ref error_info_name, finally_block } => {
                let try_result = self.exec_stmt(try_block, arena, env);
                let (signal, caught_error) = match try_result {
                    Ok(signal) => (signal, None),
                    Err(err) => {
                        if let Some(catch_b) = catch_block {
                            let error_info = self.runtime.make_error_info(&err);
                            let mut catch_env = Environment::with_parent(std::mem::replace(env, Environment::new()));
                            if let Some(ref ename) = error_name {
                                let _ = catch_env.define(
                                    ename.clone(),
                                    Value::String(err.message.clone()),
                                    false,
                                );
                            }
                            if let Some(ref einame) = error_info_name {
                                let _ = catch_env.define(
                                    einame.clone(),
                                    error_info.clone(),
                                    false,
                                );
                            }
                            // Always define implicit 오류정보
                            let _ = catch_env.define(
                                "오류정보".to_string(),
                                error_info,
                                false,
                            );
                            let catch_result = self.exec_stmt(catch_b, arena, &mut catch_env);
                            // Restore parent env
                            *env = catch_env.take_parent().unwrap_or_else(Environment::new);
                            match catch_result {
                                Ok(sig) => (sig, Some(err)),
                                Err(e) => return Err(e),
                            }
                        } else {
                            (ExecSignal::Normal, Some(err))
                        }
                    }
                };

                // Execute finally
                if let Some(fin_b) = finally_block {
                    let fin_result = self.exec_stmt(fin_b, arena, env)?;
                    match fin_result {
                        ExecSignal::Normal => {}
                        other => return Ok(other), // finally's signal overrides
                    }
                }

                // If we had an error and no catch block, re-throw
                if catch_block.is_none() {
                    if let Some(err) = caught_error {
                        return Err(err);
                    }
                }

                Ok(signal)
            }
            StmtKind::Throw(expr) => {
                let val = self.eval_expr(expr, arena, env)?;
                let msg = val.to_display_string();
                Err(CokacError::new(msg, stmt.line).with_stack(self.runtime.build_stack_trace()))
            }
            StmtKind::Block(ref stmts_list) => {
                let stmts_clone = stmts_list.clone();
                self.exec_stmts(&stmts_clone, arena, env)
            }
            StmtKind::Break => {
                if env.loop_depth <= 0 {
                    return Err(CokacError::new(
                        "'중단'은 반복문 안에서만 사용할 수 있습니다.".to_string(),
                        stmt.line,
                    ));
                }
                Ok(ExecSignal::Break)
            }
            StmtKind::Continue => {
                if env.loop_depth <= 0 {
                    return Err(CokacError::new(
                        "'계속'은 반복문 안에서만 사용할 수 있습니다.".to_string(),
                        stmt.line,
                    ));
                }
                Ok(ExecSignal::Continue)
            }
            StmtKind::Expr(expr) => {
                self.eval_expr(expr, arena, env)?;
                Ok(ExecSignal::Normal)
            }
        }
    }

    pub fn eval_expr(
        &mut self,
        expr_id: ExprId,
        arena: &AstArena,
        env: &mut Environment,
    ) -> Result<Value, CokacError> {
        if expr_id >= arena.exprs.len() {
            return Err(CokacError::new(
                format!(
                    "내부 오류: 유효하지 않은 표현식 인덱스 {} (표현식 수: {})",
                    expr_id,
                    arena.exprs.len()
                ),
                0,
            ));
        }
        let expr = arena.get_expr(expr_id).clone();
        self.expr_depth += 1;
        if self.expr_depth > self.max_expr_depth {
            self.expr_depth -= 1;
            return Err(CokacError::new(
                format!(
                    "표현식 깊이 제한({})을 초과했습니다. 괄호/중첩 표현식을 줄이거나 COKAC_MAX_EVAL_EXPR_DEPTH 값을 조정하세요.",
                    self.max_expr_depth
                ),
                expr.line,
            ));
        }
        let _expr_depth_guard = DepthGuard::new(&mut self.expr_depth);

        match expr.kind {
            ExprKind::Literal(ref val) => {
                match val {
                    // Anonymous function expressions are parsed as literal function values.
                    // They must be bound to the current arena and capture lexical scope,
                    // otherwise module-scoped methods can execute against the wrong arena.
                    Value::Function(f) if !f.is_builtin => {
                        let mut bound = f.clone();
                        if bound.arena_index == 0 {
                            bound.arena_index = self.resolve_arena_index_for(arena);
                        }
                        if bound.closure_env.is_none()
                            && (env.parent.is_some()
                                || self.runtime.current_exports.is_some()
                                || self.runtime.current_arena_index == 1)
                        {
                            bound.closure_env = Some(Rc::new(RefCell::new(env.clone())));
                        }
                        Ok(Value::Function(bound))
                    }
                    _ => Ok(val.clone()),
                }
            },
            ExprKind::Variable(ref name) => {
                if let Some(val) = env.get(name) {
                    return Ok(val);
                }
                if let Some(func) = self.runtime.find_function(name) {
                    return Ok(Value::make_function_with_arena(
                        func.name.clone(),
                        func.params.clone(),
                        func.body,
                        func.is_async,
                        func.arena_index,
                    ));
                }
                if builtins::is_builtin(name) {
                    return Ok(Value::make_builtin(name.to_string()));
                }
                Err(CokacError::new(
                    format!("정의되지 않은 변수: '{}'", name),
                    expr.line,
                ))
            }
            ExprKind::Grouping(inner) => self.eval_expr(inner, arena, env),
            ExprKind::Unary { op, right } => {
                let val = self.eval_expr(right, arena, env)?;
                match op {
                    TokenType::Minus => {
                        let n = value_to_number(&val, expr.line)?;
                        Ok(Value::Number(-n))
                    }
                    TokenType::Bang => Ok(Value::Bool(!val.is_truthy())),
                    _ => unreachable!(),
                }
            }
            ExprKind::Binary { op, left, right } => {
                self.eval_binary(op, left, right, arena, env, expr.line)
            }
            ExprKind::Call { callee, ref args } => {
                let callee_val = self.eval_expr(callee, arena, env)?;
                let args_clone = args.clone();
                let mut arg_vals = Vec::with_capacity(args_clone.len());
                for &arg in &args_clone {
                    arg_vals.push(self.eval_expr(arg, arena, env)?);
                }
                self.invoke_callable(callee_val, arg_vals, arena, env, expr.line)
            }
            ExprKind::Await(inner) => {
                let val = self.eval_expr(inner, arena, env)?;
                match val {
                    Value::Task(task) => {
                        self.await_task(task, arena, env, expr.line)
                    }
                    other => Ok(other),
                }
            }
            ExprKind::Array(ref items) => {
                let items_clone = items.clone();
                let mut vals = Vec::with_capacity(items_clone.len());
                for &item in &items_clone {
                    vals.push(self.eval_expr(item, arena, env)?);
                }
                Ok(Value::new_array(vals))
            }
            ExprKind::Object { ref keys, ref values } => {
                let keys_clone = keys.clone();
                let values_clone = values.clone();
                let obj = crate::value::ObjectValue::new();
                let obj = Rc::new(RefCell::new(obj));
                for (key, &val_id) in keys_clone.iter().zip(values_clone.iter()) {
                    let val = self.eval_expr(val_id, arena, env)?;
                    obj.borrow_mut().set(key.clone(), val);
                }
                Ok(Value::Object(obj))
            }
            ExprKind::Index { target, index } => {
                let target_val = self.eval_expr(target, arena, env)?;
                let idx_val = self.eval_expr(index, arena, env)?;
                match target_val {
                    Value::Array(arr) => {
                        let arr = arr.borrow();
                        let idx = value_to_index(&idx_val, arr.items.len(), false, expr.line)?;
                        Ok(arr.items[idx].clone())
                    }
                    Value::Object(obj) => {
                        let obj = obj.borrow();
                        let key = match idx_val {
                            Value::String(s) => s,
                            _ => idx_val.to_display_string(),
                        };
                        match obj.get(&key) {
                            Some(v) => Ok(v.clone()),
                            None => Ok(Value::Nil),
                        }
                    }
                    Value::String(s) => {
                        let idx = value_to_index(&idx_val, s.len(), false, expr.line)?;
                        let ch = s.as_bytes().get(idx).map(|&b| (b as char).to_string()).unwrap_or_default();
                        Ok(Value::String(ch))
                    }
                    _ => Err(CokacError::new(
                        "인덱스 접근은 배열, 객체 또는 문자열에만 가능합니다.".to_string(),
                        expr.line,
                    )),
                }
            }
            ExprKind::Property { target, ref name } => {
                let target_val = self.eval_expr(target, arena, env)?;
                match target_val {
                    Value::Object(obj) => {
                        let obj = obj.borrow();
                        match obj.get(name) {
                            Some(v) => Ok(v.clone()),
                            None => Ok(Value::Nil),
                        }
                    }
                    _ => Err(CokacError::new(
                        format!("'{}' 속성에 접근할 수 없습니다. 객체가 아닙니다.", name),
                        expr.line,
                    )),
                }
            }
        }
    }

    fn eval_binary(
        &mut self,
        op: TokenType,
        left_id: ExprId,
        right_id: ExprId,
        arena: &AstArena,
        env: &mut Environment,
        line: i32,
    ) -> Result<Value, CokacError> {
        if op == TokenType::AndAnd {
            let left = self.eval_expr(left_id, arena, env)?;
            if !left.is_truthy() {
                return Ok(left);
            }
            return self.eval_expr(right_id, arena, env);
        }
        if op == TokenType::OrOr {
            let left = self.eval_expr(left_id, arena, env)?;
            if left.is_truthy() {
                return Ok(left);
            }
            return self.eval_expr(right_id, arena, env);
        }

        let left = self.eval_expr(left_id, arena, env)?;
        let right = self.eval_expr(right_id, arena, env)?;

        match op {
            TokenType::Plus => {
                match (&left, &right) {
                    (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                    _ => {
                        let ls = left.to_display_string();
                        let rs = right.to_display_string();
                        Ok(Value::String(format!("{}{}", ls, rs)))
                    }
                }
            }
            TokenType::Minus => {
                let a = value_to_number(&left, line)?;
                let b = value_to_number(&right, line)?;
                Ok(Value::Number(a - b))
            }
            TokenType::Star => {
                let a = value_to_number(&left, line)?;
                let b = value_to_number(&right, line)?;
                Ok(Value::Number(a * b))
            }
            TokenType::Slash => {
                let a = value_to_number(&left, line)?;
                let b = value_to_number(&right, line)?;
                if b.abs() < 1e-12 {
                    return Err(CokacError::new("0으로 나눌 수 없습니다.".to_string(), line));
                }
                Ok(Value::Number(a / b))
            }
            TokenType::Percent => {
                let a = value_to_number(&left, line)?;
                let b = value_to_number(&right, line)?;
                if b.abs() < 1e-12 {
                    return Err(CokacError::new("0으로 나눌 수 없습니다.".to_string(), line));
                }
                Ok(Value::Number(a % b))
            }
            TokenType::Greater => {
                let a = value_to_number(&left, line)?;
                let b = value_to_number(&right, line)?;
                Ok(Value::Bool(a > b))
            }
            TokenType::GreaterEqual => {
                let a = value_to_number(&left, line)?;
                let b = value_to_number(&right, line)?;
                Ok(Value::Bool(a >= b))
            }
            TokenType::Less => {
                let a = value_to_number(&left, line)?;
                let b = value_to_number(&right, line)?;
                Ok(Value::Bool(a < b))
            }
            TokenType::LessEqual => {
                let a = value_to_number(&left, line)?;
                let b = value_to_number(&right, line)?;
                Ok(Value::Bool(a <= b))
            }
            TokenType::EqualEqual => Ok(Value::Bool(left.equals(&right))),
            TokenType::BangEqual => Ok(Value::Bool(!left.equals(&right))),
            _ => unreachable!(),
        }
    }

    pub fn invoke_callable(
        &mut self,
        callee: Value,
        args: Vec<Value>,
        arena: &AstArena,
        env: &mut Environment,
        line: i32,
    ) -> Result<Value, CokacError> {
        match callee {
            Value::Function(ref func) => {
                if func.is_builtin {
                    return builtins::call_builtin(
                        &func.name,
                        args,
                        self,
                        arena,
                        env,
                        line,
                    );
                }

                // Extract values before borrowing issues
                let func_body = func.body;
                let func_arena_index = func.arena_index;
                let func_params = func.params.clone();
                let func_name = if func.name.is_empty() { "익명".to_string() } else { func.name.clone() };
                let func_is_async = func.is_async;
                let captured_env_ref = func.closure_env.clone();

                if func_is_async {
                    let task = Value::new_task();
                    if !self.runtime.try_enqueue_task(&task, line) {
                        return Ok(Value::Task(task));
                    }
                    let mut func_env = if let Some(captured) = captured_env_ref.clone() {
                        Environment::with_parent(captured.borrow().clone())
                    } else {
                        Environment::with_parent(env.clone())
                    };
                    func_env.function_depth = 1;
                    for (i, param) in func_params.iter().enumerate() {
                        let val = args.get(i).cloned().unwrap_or(Value::Nil);
                        let _ = func_env.define(param.clone(), val, false);
                    }
                    self.runtime.async_jobs.push(crate::runtime::AsyncJob {
                        task: task.clone(),
                        kind: crate::runtime::AsyncJobKind::Function {
                            env: func_env,
                            body: func_body,
                            arena_index: func_arena_index,
                        },
                    });
                    self.runtime.mark_async_enqueued();
                    return Ok(Value::Task(task));
                }

                // Captured closures run with lexical parent; normal functions keep existing propagation behavior.
                let mut func_env = if let Some(captured) = captured_env_ref.clone() {
                    Environment::with_parent(captured.borrow().clone())
                } else {
                    Environment::with_parent(std::mem::replace(env, Environment::new()))
                };
                func_env.function_depth = 1;
                for (i, param) in func_params.iter().enumerate() {
                    let val = args.get(i).cloned().unwrap_or(Value::Nil);
                    let _ = func_env.define(param.clone(), val, false);
                }

                self.runtime.call_push(&func_name, line);

                let result = self.exec_function_body_with_arena(
                    func_body,
                    func_arena_index,
                    arena,
                    &mut func_env,
                    line,
                );

                if let Some(captured) = captured_env_ref {
                    // Persist captured lexical scope mutations across closure calls.
                    let parent_after_call = func_env.take_parent().unwrap_or_else(Environment::new);
                    *captured.borrow_mut() = parent_after_call;
                } else {
                    // Restore caller environment for normal function calls.
                    *env = func_env.take_parent().unwrap_or_else(Environment::new);
                }

                match result {
                    Ok(ExecSignal::Return(val)) => {
                        self.runtime.call_pop();
                        Ok(val)
                    }
                    Ok(_) => {
                        self.runtime.call_pop();
                        Ok(Value::Nil)
                    }
                    Err(mut e) => {
                        if e.stack.is_empty() {
                            e.stack = self.runtime.build_stack_trace();
                        }
                        self.runtime.call_pop();
                        Err(e)
                    }
                }
            }
            _ => Err(CokacError::new(
                format!("'{}' 타입은 호출할 수 없습니다.", callee.type_name()),
                line,
            )),
        }
    }

    fn await_task(
        &mut self,
        task: Rc<RefCell<TaskValue>>,
        arena: &AstArena,
        env: &mut Environment,
        line: i32,
    ) -> Result<Value, CokacError> {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(300);
        loop {
            {
                let t = task.borrow();
                if t.completed {
                    if t.failed {
                        let original = t.error_message.clone().unwrap_or_else(|| "알 수 없는 오류".to_string());
                        let msg = format!("비동기 작업 실패: {}", original);
                        return Err(CokacError::new(msg, line));
                    }
                    return Ok(t.result.clone().unwrap_or(Value::Nil));
                }
            }
            self.runtime.async_wait_calls += 1;
            if std::time::Instant::now() >= deadline {
                self.runtime.async_wait_timeouts += 1;
                return Err(CokacError::new(
                    "비동기 작업 대기 시간 초과".to_string(),
                    line,
                ));
            }
            let progressed = self.drive_async(arena, env)?;
            if !progressed {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
    }

    pub fn drive_async(
        &mut self,
        arena: &AstArena,
        _env: &mut Environment,
    ) -> Result<bool, CokacError> {
        self.runtime.async_loop_ticks += 1;
        if self.runtime.async_jobs.is_empty() {
            return Ok(false);
        }

        let job = self.runtime.async_jobs.remove(0);
        match job.kind {
            crate::runtime::AsyncJobKind::Function { mut env, body, arena_index } => {
                let result = self.exec_function_body_with_arena(
                    body,
                    arena_index,
                    arena,
                    &mut env,
                    0,
                );
                match result {
                    Ok(ExecSignal::Return(val)) => {
                        job.task.borrow_mut().complete_success(val);
                        self.runtime.async_completed += 1;
                    }
                    Ok(_) => {
                        job.task.borrow_mut().complete_success(Value::Nil);
                        self.runtime.async_completed += 1;
                    }
                    Err(e) => {
                        job.task.borrow_mut().complete_error(
                            e.message.clone(),
                            e.code.as_str().to_string(),
                            e.line,
                            e.stack.clone(),
                        );
                        self.runtime.async_failed += 1;
                    }
                }
            }
            crate::runtime::AsyncJobKind::DeferredOk(value) => {
                job.task.borrow_mut().complete_success(value);
                self.runtime.async_completed += 1;
            }
            crate::runtime::AsyncJobKind::DeferredErr { message, code, line, stack } => {
                job.task.borrow_mut().complete_error(message, code, line, stack);
                self.runtime.async_failed += 1;
            }
            crate::runtime::AsyncJobKind::ThreadWorker { receiver } => {
                match receiver.try_recv() {
                    Ok(result) => {
                        match result {
                            crate::runtime::ThreadResult::HttpResponse { status, body, headers, headers_raw, method, url } => {
                                let obj = make_http_response_value(status, body, headers, headers_raw, method, url);
                                job.task.borrow_mut().complete_success(obj);
                                self.runtime.async_completed += 1;
                            }
                            crate::runtime::ThreadResult::ServerAccept { client_fd, method, path, version, headers, body, remote_addr } => {
                                let obj = make_accept_value(client_fd, method, path, version, headers, body, remote_addr);
                                job.task.borrow_mut().complete_success(obj);
                                self.runtime.async_completed += 1;
                            }
                            crate::runtime::ThreadResult::SimpleValue(text) => {
                                job.task.borrow_mut().complete_success(Value::String(text));
                                self.runtime.async_completed += 1;
                            }
                            crate::runtime::ThreadResult::Error(msg) => {
                                job.task.borrow_mut().complete_error(
                                    msg, "E_RUNTIME".to_string(), 0, Vec::new(),
                                );
                                self.runtime.async_failed += 1;
                            }
                        }
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        self.runtime.async_requeued += 1;
                        self.runtime.async_jobs.push(crate::runtime::AsyncJob {
                            task: job.task,
                            kind: crate::runtime::AsyncJobKind::ThreadWorker { receiver },
                        });
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        job.task.borrow_mut().complete_error(
                            "작업자 스레드 연결 끊김".to_string(),
                            "E_RUNTIME".to_string(),
                            0,
                            Vec::new(),
                        );
                        self.runtime.async_failed += 1;
                    }
                }
            }
            crate::runtime::AsyncJobKind::All { deps } => {
                let mut all_done = true;
                let mut results = Vec::with_capacity(deps.len());
                for dep in &deps {
                    let d = dep.borrow();
                    if !d.completed {
                        all_done = false;
                        break;
                    }
                    if d.failed {
                        let msg = d.error_message.clone().unwrap_or_default();
                        let code = d.error_code.clone().unwrap_or_default();
                        job.task.borrow_mut().complete_error(msg, code, d.error_line, d.error_stack.clone());
                        self.runtime.async_completed += 1;
                        return Ok(true);
                    }
                    results.push(d.result.clone().unwrap_or(Value::Nil));
                }
                if all_done {
                    job.task.borrow_mut().complete_success(Value::new_array(results));
                    self.runtime.async_completed += 1;
                } else {
                    self.runtime.async_requeued += 1;
                    self.runtime.async_jobs.push(crate::runtime::AsyncJob {
                        task: job.task,
                        kind: crate::runtime::AsyncJobKind::All { deps },
                    });
                }
            }
            crate::runtime::AsyncJobKind::Race { deps } => {
                let mut found = false;
                for dep in &deps {
                    let d = dep.borrow();
                    if d.completed {
                        if d.failed {
                            let msg = d.error_message.clone().unwrap_or_default();
                            let code = d.error_code.clone().unwrap_or_default();
                            job.task.borrow_mut().complete_error(msg, code, d.error_line, d.error_stack.clone());
                        } else {
                            job.task.borrow_mut().complete_success(d.result.clone().unwrap_or(Value::Nil));
                        }
                        self.runtime.async_completed += 1;
                        found = true;
                        break;
                    }
                }
                if !found {
                    self.runtime.async_requeued += 1;
                    self.runtime.async_jobs.push(crate::runtime::AsyncJob {
                        task: job.task,
                        kind: crate::runtime::AsyncJobKind::Race { deps },
                    });
                }
            }
        }
        Ok(true)
    }

    pub fn drain_async(&mut self, arena: &AstArena, env: &mut Environment) -> Result<(), CokacError> {
        while !self.runtime.async_jobs.is_empty() {
            self.drive_async(arena, env)?;
        }
        Ok(())
    }

    fn exec_import(
        &mut self,
        path: &str,
        alias: Option<&str>,
        _arena: &AstArena,
        env: &mut Environment,
        line: i32,
    ) -> Result<ExecSignal, CokacError> {
        let resolved = self.runtime.resolve_import_path(path);

        if let Some(alias_name) = alias {
            // Check cache
            if let Some(exports) = self.runtime.find_module(&resolved) {
                env.define(alias_name.to_string(), exports, true)
                    .map_err(|msg| CokacError::new(msg, line))?;
                return Ok(ExecSignal::Normal);
            }

            let source = std::fs::read_to_string(&resolved).map_err(|e| {
                CokacError::new(
                    format!("파일을 읽을 수 없습니다: '{}': {}", resolved, e),
                    line,
                )
            })?;

            let tokens = lexer::lex_source(&source).map_err(|msg| CokacError::new(msg, line))?;
            let parser = Parser::new(tokens);
            let (new_arena, stmts) = parser.parse().map_err(|msg| CokacError::new(msg, line))?;

            // Store module arena BEFORE executing
            self.runtime.loaded_arenas.push(Rc::new(new_arena));
            let arena_idx = self.runtime.loaded_arenas.len(); // 1-indexed
            let prev_arena_idx = self.runtime.current_arena_index;
            self.runtime.current_arena_index = arena_idx;

            // Get reference to stored arena
            let module_arena = Rc::clone(&self.runtime.loaded_arenas[arena_idx - 1]);

            let exports_obj = crate::value::ObjectValue::new();
            let exports = Rc::new(RefCell::new(exports_obj));
            let prev_exports = self.runtime.current_exports.take();
            let prev_file = self.runtime.current_file.take();
            self.runtime.current_exports = Some(exports.clone());
            self.runtime.current_file = Some(resolved.clone());

            let mut module_env = Environment::new();
            {
                let mut eval = Evaluator::new(self.runtime);
                let signal = eval.exec_stmts(&stmts, &module_arena, &mut module_env)?;
                match signal {
                    ExecSignal::Normal => {}
                    ExecSignal::Return(_) => {
                        return Err(CokacError::new(
                            "모듈 최상위에서는 '반환'을 사용할 수 없습니다.".to_string(),
                            line,
                        ));
                    }
                    ExecSignal::Break => {
                        return Err(CokacError::new(
                            "모듈 최상위에서는 '중단'을 사용할 수 없습니다.".to_string(),
                            line,
                        ));
                    }
                    ExecSignal::Continue => {
                        return Err(CokacError::new(
                            "모듈 최상위에서는 '계속'을 사용할 수 없습니다.".to_string(),
                            line,
                        ));
                    }
                }
            }

            self.runtime.current_exports = prev_exports;
            self.runtime.current_file = prev_file;
            self.runtime.current_arena_index = prev_arena_idx;

            let exports_val = Value::Object(exports);
            self.runtime.add_module(resolved, exports_val.clone());
            env.define(alias_name.to_string(), exports_val, true)
                .map_err(|msg| CokacError::new(msg, line))?;
        } else {
            // Direct import (no alias)
            if self.runtime.is_imported(&resolved) {
                return Ok(ExecSignal::Normal);
            }
            self.runtime.add_imported(resolved.clone());

            let source = std::fs::read_to_string(&resolved).map_err(|e| {
                CokacError::new(
                    format!("파일을 읽을 수 없습니다: '{}': {}", resolved, e),
                    line,
                )
            })?;

            let tokens = lexer::lex_source(&source).map_err(|msg| CokacError::new(msg, line))?;
            let parser = Parser::new(tokens);
            let (new_arena, stmts) = parser.parse().map_err(|msg| CokacError::new(msg, line))?;

            // Store module arena
            self.runtime.loaded_arenas.push(Rc::new(new_arena));
            let arena_idx = self.runtime.loaded_arenas.len();
            let prev_arena_idx = self.runtime.current_arena_index;
            self.runtime.current_arena_index = arena_idx;

            let module_arena = Rc::clone(&self.runtime.loaded_arenas[arena_idx - 1]);

            let prev_file = self.runtime.current_file.take();
            self.runtime.current_file = Some(resolved);

            {
                let mut eval = Evaluator::new(self.runtime);
                let signal = eval.exec_stmts(&stmts, &module_arena, env)?;
                match signal {
                    ExecSignal::Normal => {}
                    ExecSignal::Return(_) => {
                        return Err(CokacError::new(
                            "모듈 최상위에서는 '반환'을 사용할 수 없습니다.".to_string(),
                            line,
                        ));
                    }
                    ExecSignal::Break => {
                        return Err(CokacError::new(
                            "모듈 최상위에서는 '중단'을 사용할 수 없습니다.".to_string(),
                            line,
                        ));
                    }
                    ExecSignal::Continue => {
                        return Err(CokacError::new(
                            "모듈 최상위에서는 '계속'을 사용할 수 없습니다.".to_string(),
                            line,
                        ));
                    }
                }
            }

            self.runtime.current_file = prev_file;
            self.runtime.current_arena_index = prev_arena_idx;
        }

        Ok(ExecSignal::Normal)
    }
}

fn make_http_response_value(
    status: u16,
    body: String,
    headers: Vec<(String, String)>,
    headers_raw: String,
    method: String,
    url: String,
) -> Value {
    let obj = ObjectValue::new();
    let obj = Rc::new(RefCell::new(obj));
    let headers_obj = ObjectValue::new();
    let headers_rc = Rc::new(RefCell::new(headers_obj));
    for (k, v) in headers {
        headers_rc.borrow_mut().set(k, Value::String(v));
    }
    {
        let mut o = obj.borrow_mut();
        o.set("상태".to_string(), Value::Number(status as f64));
        o.set("본문".to_string(), Value::String(body));
        o.set("성공".to_string(), Value::Bool(status >= 200 && status < 300));
        o.set("헤더".to_string(), Value::String(headers_raw));
        o.set("헤더들".to_string(), Value::Object(headers_rc));
        o.set("메서드".to_string(), Value::String(method));
        o.set("주소".to_string(), Value::String(url));
    }
    Value::Object(obj)
}

fn make_accept_value(
    client_fd: i32, method: String, path: String, version: String,
    headers: Vec<(String, String)>, body: String, remote_addr: String,
) -> Value {
    let obj = ObjectValue::new();
    let obj = Rc::new(RefCell::new(obj));
    let headers_obj = ObjectValue::new();
    let headers_rc = Rc::new(RefCell::new(headers_obj));
    for (k, v) in headers {
        headers_rc.borrow_mut().set(k, Value::String(v));
    }
    {
        let mut o = obj.borrow_mut();
        o.set("연결".to_string(), Value::Number(client_fd as f64));
        o.set("메서드".to_string(), Value::String(method));
        o.set("경로".to_string(), Value::String(path));
        o.set("버전".to_string(), Value::String(version));
        o.set("헤더".to_string(), Value::Object(headers_rc));
        o.set("본문".to_string(), Value::String(body));
        o.set("원격주소".to_string(), Value::String(remote_addr));
    }
    Value::Object(obj)
}

fn read_depth_limit(key: &str, default_value: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|v| *v >= 64)
        .unwrap_or(default_value)
}

struct DepthGuard {
    depth: *mut usize,
}

impl DepthGuard {
    fn new(depth: &mut usize) -> Self {
        Self { depth: depth as *mut usize }
    }
}

impl Drop for DepthGuard {
    fn drop(&mut self) {
        // SAFETY: depth points to a live field on Evaluator for the guard lifetime.
        unsafe {
            *self.depth -= 1;
        }
    }
}

pub fn value_to_index(val: &Value, len: usize, allow_end: bool, line: i32) -> Result<usize, CokacError> {
    let n = match val {
        Value::Number(n) => *n,
        _ => return Err(CokacError::new(
            "인덱스는 숫자여야 합니다.".to_string(),
            line,
        )),
    };
    if n != n.trunc() || n < 0.0 {
        return Err(CokacError::new(
            format!("인덱스 {}이(가) 범위를 벗어났습니다.", n),
            line,
        ));
    }
    let idx = n as usize;
    let max = if allow_end { len } else { len.saturating_sub(1) };
    if len == 0 || idx > max {
        return Err(CokacError::new(
            format!("인덱스 {}이(가) 범위를 벗어났습니다 (길이: {}).", idx, len),
            line,
        ));
    }
    Ok(idx)
}
