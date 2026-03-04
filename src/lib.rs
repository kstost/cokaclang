pub mod error;
pub mod token;
pub mod lexer;
pub mod ast;
pub mod parser;
pub mod value;
pub mod environment;
pub mod evaluator;
pub mod runtime;
pub mod json;
pub mod builtins;

use environment::Environment;
use evaluator::{Evaluator, ExecSignal};
use runtime::Runtime;
use std::rc::Rc;

pub fn run_script(path: &str, args: Vec<String>) -> Result<(), String> {
    let source = std::fs::read_to_string(path).map_err(|e| {
        format!("파일을 읽을 수 없습니다: '{}': {}", path, e)
    })?;

    let tokens = lexer::lex_source(&source)?;
    let parser_instance = parser::Parser::new(tokens);
    let (arena, stmts) = parser_instance.parse()?;

    let mut runtime = Runtime::new();
    runtime.script_argc = args.len();
    runtime.script_argv = args;

    // Register main script arena so user functions keep a stable arena index
    // even when called from imported module arenas.
    runtime.loaded_arenas.push(Rc::new(arena));
    runtime.current_arena_index = 1;
    let main_arena = Rc::clone(&runtime.loaded_arenas[0]);

    let canonical = std::fs::canonicalize(path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string());
    runtime.current_file = Some(canonical);

    let mut env = Environment::new();
    {
        let mut eval = Evaluator::new(&mut runtime);
        let signal = eval.exec_stmts(&stmts, &main_arena, &mut env).map_err(|e| {
            format!("{}", e)
        })?;
        match signal {
            ExecSignal::Normal => {}
            ExecSignal::Return(_) => {
                return Err("함수 바깥에서는 '반환'을 사용할 수 없습니다.".to_string());
            }
            ExecSignal::Break => {
                return Err("반복문 바깥에서는 '중단'을 사용할 수 없습니다.".to_string());
            }
            ExecSignal::Continue => {
                return Err("반복문 바깥에서는 '계속'을 사용할 수 없습니다.".to_string());
            }
        }
        eval.drain_async(&main_arena, &mut env).map_err(|e| {
            format!("{}", e)
        })?;
    }

    Ok(())
}
