use std::path::{Path, PathBuf};
use std::cell::RefCell;
use std::rc::Rc;

use crate::ast::{AstArena, StmtId};
use crate::environment::Environment;
use crate::error::CokacError;
use crate::value::{ObjectValue, TaskValue, Value};

pub struct FunctionEntry {
    pub name: String,
    pub params: Vec<String>,
    pub body: StmtId,
    pub is_async: bool,
    pub arena_index: usize,
}

pub struct ModuleEntry {
    pub path: String,
    pub exports: Value, // Object
}

pub struct Runtime {
    pub functions: Vec<FunctionEntry>,
    pub imported_paths: Vec<String>,
    pub loaded_arenas: Vec<Rc<AstArena>>,
    pub loaded_stmts: Vec<Vec<StmtId>>,
    pub current_arena_index: usize, // 0 = main, N = loaded_arenas[N-1]
    pub modules: Vec<ModuleEntry>,
    pub call_stack: Vec<(String, i32)>,
    pub current_exports: Option<Rc<RefCell<ObjectValue>>>,
    pub current_file: Option<String>,
    pub script_argc: usize,
    pub script_argv: Vec<String>,
    // Async job queue (simplified: we use tokio in Rust)
    pub async_runtime: Option<tokio::runtime::Runtime>,
    pub async_jobs: Vec<AsyncJob>,
    pub async_enqueued: u64,
    pub async_completed: u64,
    pub async_failed: u64,
    pub async_cancelled: u64,
    pub async_max_queue: usize,
    pub async_queue_limit: usize,
    pub async_requeued: u64,
    pub async_backpressure: u64,
    pub async_loop_ticks: u64,
    pub async_wait_calls: u64,
    pub async_wait_timeouts: u64,
}

pub struct AsyncJob {
    pub task: Rc<RefCell<crate::value::TaskValue>>,
    pub kind: AsyncJobKind,
}

pub enum AsyncJobKind {
    Function {
        env: Environment,
        body: StmtId,
        arena_index: usize,
    },
    DeferredOk(Value),
    DeferredErr {
        message: String,
        code: String,
        line: i32,
        stack: Vec<String>,
    },
    ThreadWorker {
        receiver: std::sync::mpsc::Receiver<ThreadResult>,
    },
    All {
        deps: Vec<Rc<RefCell<crate::value::TaskValue>>>,
    },
    Race {
        deps: Vec<Rc<RefCell<crate::value::TaskValue>>>,
    },
}

    pub enum ThreadResult {
    HttpResponse {
        status: u16,
        body: String,
        headers: Vec<(String, String)>,
        headers_raw: String,
        method: String,
        url: String,
    },
    ServerAccept {
        client_fd: i32,
        method: String,
        path: String,
        version: String,
        headers: Vec<(String, String)>,
        body: String,
        remote_addr: String,
    },
    SimpleValue(String),
    Error(String),
}

impl Runtime {
    pub fn new() -> Self {
        let max_queue: usize = std::env::var("COKAC_ASYNC_MAX_QUEUE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(4096);

        Runtime {
            functions: Vec::new(),
            imported_paths: Vec::new(),
            loaded_arenas: Vec::new(),
            loaded_stmts: Vec::new(),
            current_arena_index: 0,
            modules: Vec::new(),
            call_stack: Vec::new(),
            current_exports: None,
            current_file: None,
            script_argc: 0,
            script_argv: Vec::new(),
            async_runtime: None,
            async_jobs: Vec::new(),
            async_enqueued: 0,
            async_completed: 0,
            async_failed: 0,
            async_cancelled: 0,
            async_max_queue: 0,
            async_queue_limit: max_queue,
            async_requeued: 0,
            async_backpressure: 0,
            async_loop_ticks: 0,
            async_wait_calls: 0,
            async_wait_timeouts: 0,
        }
    }

    pub fn try_enqueue_task(&mut self, task: &Rc<RefCell<TaskValue>>, line: i32) -> bool {
        if self.async_jobs.len() >= self.async_queue_limit {
            self.async_backpressure += 1;
            task.borrow_mut().complete_error(
                "비동기 큐가 가득 찼습니다.".to_string(),
                "E_BACKPRESSURE".to_string(),
                line,
                Vec::new(),
            );
            return false;
        }
        true
    }

    pub fn mark_async_enqueued(&mut self) {
        self.async_enqueued += 1;
        let qlen = self.async_jobs.len();
        if qlen > self.async_max_queue {
            self.async_max_queue = qlen;
        }
    }

    pub fn register_function(&mut self, name: String, params: Vec<String>, body: StmtId, is_async: bool, arena_index: usize) {
        // Replace existing if same name
        for f in &mut self.functions {
            if f.name == name {
                f.params = params;
                f.body = body;
                f.is_async = is_async;
                f.arena_index = arena_index;
                return;
            }
        }
        self.functions.push(FunctionEntry { name, params, body, is_async, arena_index });
    }

    pub fn find_function(&self, name: &str) -> Option<&FunctionEntry> {
        self.functions.iter().find(|f| f.name == name)
    }

    pub fn call_push(&mut self, name: &str, line: i32) {
        self.call_stack.push((name.to_string(), line));
    }

    pub fn call_pop(&mut self) {
        self.call_stack.pop();
    }

    pub fn build_stack_trace(&self) -> Vec<String> {
        self.call_stack.iter().map(|(name, line)| {
            format!("{} (줄 {})", name, line)
        }).collect()
    }

    pub fn make_error_info(&self, err: &CokacError) -> Value {
        let obj = ObjectValue::new();
        let obj = Rc::new(RefCell::new(obj));
        {
            let mut o = obj.borrow_mut();
            o.set("메시지".to_string(), Value::String(err.message.clone()));
            o.set("코드".to_string(), Value::String(err.code.as_str().to_string()));
            o.set("줄".to_string(), Value::Number(err.line as f64));
            let stack_arr: Vec<Value> = err.stack.iter().map(|s| Value::String(s.clone())).collect();
            o.set("스택".to_string(), Value::new_array(stack_arr));
        }
        Value::Object(obj)
    }

    pub fn is_imported(&self, path: &str) -> bool {
        self.imported_paths.iter().any(|p| p == path)
    }

    pub fn add_imported(&mut self, path: String) {
        self.imported_paths.push(path);
    }

    pub fn resolve_import_path(&self, import_path: &str) -> String {
        if Path::new(import_path).is_absolute() {
            return import_path.to_string();
        }
        let base_dir = if let Some(ref cur) = self.current_file {
            Path::new(cur).parent().unwrap_or(Path::new(".")).to_path_buf()
        } else {
            PathBuf::from(".")
        };
        let candidate = base_dir.join(import_path);
        // Try to canonicalize
        match candidate.canonicalize() {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => candidate.to_string_lossy().to_string(),
        }
    }

    pub fn resolve_path(&self, path: &str) -> String {
        if Path::new(path).is_absolute() {
            return path.to_string();
        }
        let base_dir = if let Some(ref cur) = self.current_file {
            Path::new(cur).parent().unwrap_or(Path::new(".")).to_path_buf()
        } else {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        };
        let candidate = base_dir.join(path);
        candidate.to_string_lossy().to_string()
    }

    pub fn find_module(&self, path: &str) -> Option<Value> {
        self.modules.iter().find(|m| m.path == path).map(|m| m.exports.clone())
    }

    pub fn add_module(&mut self, path: String, exports: Value) {
        self.modules.push(ModuleEntry { path, exports });
    }

    pub fn get_or_create_tokio_runtime(&mut self) -> &tokio::runtime::Runtime {
        if self.async_runtime.is_none() {
            self.async_runtime = Some(
                tokio::runtime::Runtime::new().expect("tokio 런타임 생성 실패")
            );
        }
        self.async_runtime.as_ref().unwrap()
    }
}
