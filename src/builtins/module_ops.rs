use crate::ast::AstArena;
use crate::environment::Environment;
use crate::error::CokacError;
use crate::evaluator::{Evaluator, ExecSignal};
use crate::runtime::Runtime;
use crate::value::Value;

pub fn builtin_export(args: Vec<Value>, runtime: &mut Runtime, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'내보내기'는 2개의 인수가 필요합니다.".to_string(), line));
    }
    let name = args[0].to_display_string();
    let value = args[1].clone();

    match &runtime.current_exports {
        Some(exports) => {
            exports.borrow_mut().set(name, value.clone());
            Ok(value)
        }
        None => Err(CokacError::new(
            "'내보내기'는 모듈 실행 중에만 사용할 수 있습니다.".to_string(),
            line,
        )),
    }
}

pub fn builtin_module_import(
    args: Vec<Value>,
    eval: &mut Evaluator,
    _arena: &AstArena,
    _env: &mut Environment,
    line: i32,
) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'모듈가져오기'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let path = args[0].to_display_string();
    let resolved = eval.runtime.resolve_import_path(&path);

    // Check cache
    if let Some(exports) = eval.runtime.find_module(&resolved) {
        return Ok(exports);
    }

    let source = std::fs::read_to_string(&resolved).map_err(|e| {
        CokacError::new(
            format!("모듈 파일을 읽을 수 없습니다: '{}': {}", resolved, e),
            line,
        )
    })?;

    let tokens = crate::lexer::lex_source(&source).map_err(|msg| CokacError::new(msg, line))?;
    let parser = crate::parser::Parser::new(tokens);
    let (new_arena, stmts) = parser.parse().map_err(|msg| CokacError::new(msg, line))?;

    // Store module arena
    eval.runtime.loaded_arenas.push(std::rc::Rc::new(new_arena));
    let arena_idx = eval.runtime.loaded_arenas.len();
    let prev_arena_idx = eval.runtime.current_arena_index;
    eval.runtime.current_arena_index = arena_idx;

    let module_arena = std::rc::Rc::clone(&eval.runtime.loaded_arenas[arena_idx - 1]);

    // Execute module in new env
    let exports_obj = crate::value::ObjectValue::new();
    let exports = std::rc::Rc::new(std::cell::RefCell::new(exports_obj));
    let prev_exports = eval.runtime.current_exports.take();
    let prev_file = eval.runtime.current_file.take();
    eval.runtime.current_exports = Some(exports.clone());
    eval.runtime.current_file = Some(resolved.clone());

    let mut module_env = Environment::new();
    {
        let mut module_eval = Evaluator::new(eval.runtime);
        let signal = module_eval.exec_stmts(&stmts, &module_arena, &mut module_env)?;
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

    eval.runtime.current_exports = prev_exports;
    eval.runtime.current_file = prev_file;
    eval.runtime.current_arena_index = prev_arena_idx;

    let exports_val = Value::Object(exports);
    eval.runtime.add_module(resolved, exports_val.clone());
    Ok(exports_val)
}
