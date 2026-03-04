use wasm_bindgen::prelude::*;

use crate::environment::Environment;
use crate::evaluator::{Evaluator, ExecSignal};
use crate::output;
use crate::runtime::Runtime;
use std::rc::Rc;

#[wasm_bindgen]
pub fn run_code(source: &str) -> String {
    // Clear output buffer
    output::take_output();

    // Lex
    let tokens = match crate::lexer::lex_source(source) {
        Ok(t) => t,
        Err(e) => return format!("[오류] {}", e),
    };

    // Parse
    let parser = crate::parser::Parser::new(tokens);
    let (arena, stmts) = match parser.parse() {
        Ok(r) => r,
        Err(e) => return format!("[오류] {}", e),
    };

    // Evaluate
    let mut runtime = Runtime::new();
    runtime.loaded_arenas.push(Rc::new(arena));
    runtime.current_arena_index = 1;
    let main_arena = Rc::clone(&runtime.loaded_arenas[0]);

    let mut env = Environment::new();
    {
        let mut eval = Evaluator::new(&mut runtime);
        match eval.exec_stmts(&stmts, &main_arena, &mut env) {
            Ok(signal) => {
                match signal {
                    ExecSignal::Normal => {}
                    ExecSignal::Return(_) => {
                        let mut out = output::take_output();
                        out.push_str("[오류] 함수 바깥에서는 '반환'을 사용할 수 없습니다.\n");
                        return out;
                    }
                    ExecSignal::Break => {
                        let mut out = output::take_output();
                        out.push_str("[오류] 반복문 바깥에서는 '중단'을 사용할 수 없습니다.\n");
                        return out;
                    }
                    ExecSignal::Continue => {
                        let mut out = output::take_output();
                        out.push_str("[오류] 반복문 바깥에서는 '계속'을 사용할 수 없습니다.\n");
                        return out;
                    }
                }
                // Drain any remaining async jobs
                if let Err(e) = eval.drain_async(&main_arena, &mut env) {
                    let mut out = output::take_output();
                    out.push_str(&format!("[오류] {}\n", e));
                    return out;
                }
            }
            Err(e) => {
                let mut out = output::take_output();
                out.push_str(&format!("[오류] {}\n", e));
                return out;
            }
        }
    }

    output::take_output()
}
