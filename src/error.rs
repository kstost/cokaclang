use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCode {
    Runtime,
    SecurityPolicy,
    HttpStatus,
    HttpRedirect,
    HttpTls,
    Timeout,
    HttpNetwork,
    TaskCancelled,
    UndefinedVariable,
    Argument,
    OutOfRange,
    NotAllowed,
    Json,
    FileNotFound,
    FilePermission,
    Http,
    Filesystem,
    Task,
    Memory,
    Backpressure,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::Runtime => "E_RUNTIME",
            ErrorCode::SecurityPolicy => "E_SECURITY_POLICY",
            ErrorCode::HttpStatus => "E_HTTP_STATUS",
            ErrorCode::HttpRedirect => "E_HTTP_REDIRECT",
            ErrorCode::HttpTls => "E_HTTP_TLS",
            ErrorCode::Timeout => "E_TIMEOUT",
            ErrorCode::HttpNetwork => "E_HTTP_NETWORK",
            ErrorCode::TaskCancelled => "E_TASK_CANCELLED",
            ErrorCode::UndefinedVariable => "E_UNDEFINED_VARIABLE",
            ErrorCode::Argument => "E_ARGUMENT",
            ErrorCode::OutOfRange => "E_OUT_OF_RANGE",
            ErrorCode::NotAllowed => "E_NOT_ALLOWED",
            ErrorCode::Json => "E_JSON",
            ErrorCode::FileNotFound => "E_FILE_NOT_FOUND",
            ErrorCode::FilePermission => "E_FILE_PERMISSION",
            ErrorCode::Http => "E_HTTP",
            ErrorCode::Filesystem => "E_FILESYSTEM",
            ErrorCode::Task => "E_TASK",
            ErrorCode::Memory => "E_MEMORY",
            ErrorCode::Backpressure => "E_BACKPRESSURE",
        }
    }

    pub fn from_message(msg: &str) -> ErrorCode {
        if msg.contains("보안 정책") { return ErrorCode::SecurityPolicy; }
        if msg.contains("HTTP 상태") { return ErrorCode::HttpStatus; }
        if msg.contains("리다이렉트") || msg.to_ascii_lowercase().contains("redirect") {
            return ErrorCode::HttpRedirect;
        }
        if msg.contains("TLS") || msg.contains("인증서") { return ErrorCode::HttpTls; }
        if msg.contains("시간 초과") || msg.contains("타임아웃") { return ErrorCode::Timeout; }
        if msg.contains("네트워크") || msg.contains("연결") { return ErrorCode::HttpNetwork; }
        if msg.contains("취소") { return ErrorCode::TaskCancelled; }
        if msg.contains("정의되지 않은") { return ErrorCode::UndefinedVariable; }
        if msg.contains("인수") || msg.contains("인자") { return ErrorCode::Argument; }
        if msg.contains("범위") { return ErrorCode::OutOfRange; }
        if msg.contains("허용") { return ErrorCode::NotAllowed; }
        if msg.contains("JSON") { return ErrorCode::Json; }
        if msg.contains("파일을 찾을 수 없") || msg.contains("존재하지 않") { return ErrorCode::FileNotFound; }
        if msg.contains("권한") { return ErrorCode::FilePermission; }
        if msg.contains("HTTP") || msg.contains("웹") { return ErrorCode::Http; }
        if msg.contains("파일") || msg.contains("디렉토리") { return ErrorCode::Filesystem; }
        if msg.contains("작업") { return ErrorCode::Task; }
        if msg.contains("메모리") { return ErrorCode::Memory; }
        if msg.contains("백프레셔") { return ErrorCode::Backpressure; }
        ErrorCode::Runtime
    }
}

#[derive(Debug, Clone)]
pub struct CokacError {
    pub message: String,
    pub code: ErrorCode,
    pub line: i32,
    pub stack: Vec<String>,
}

impl CokacError {
    pub fn new(message: String, line: i32) -> Self {
        let code = ErrorCode::from_message(&message);
        CokacError {
            message,
            code,
            line,
            stack: Vec::new(),
        }
    }

    pub fn with_stack(mut self, stack: Vec<String>) -> Self {
        self.stack = stack;
        self
    }
}

impl fmt::Display for CokacError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[줄 {}] 실행 오류: {}", self.line, self.message)?;
        if !self.stack.is_empty() {
            write!(f, "\n호출 스택:")?;
            for entry in self.stack.iter().rev() {
                write!(f, "\n  {}", entry)?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for CokacError {}

pub type CokacResult<T> = Result<T, CokacError>;
