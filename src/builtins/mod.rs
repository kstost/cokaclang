pub mod core;
pub mod string_ops;
pub mod array_ops;
pub mod object_ops;
pub mod file_ops;
pub mod dir_path_ops;
pub mod json_ops;
pub mod hash_encoding;
pub mod time_random;
pub mod stdio_io;
pub mod process_ops;
pub mod http_client;
pub mod http_server;
pub mod task_ops;
pub mod oop;
pub mod module_ops;

use crate::ast::AstArena;
use crate::environment::Environment;
use crate::error::CokacError;
use crate::evaluator::Evaluator;
use crate::value::Value;

static BUILTIN_NAMES: &[&str] = &[
    "가져오기요청",
    "객체가짐",
    "객체값들",
    "객체복사",
    "객체삭제",
    "객체설정",
    "객체키들",
    "객체합치기",
    "경로이름",
    "경로정규화",
    "경로존재",
    "경로합치기",
    "길이",
    "난수",
    "난수정수",
    "내보내기",
    "대기밀리초",
    "대기최대",
    "단언",
    "디렉토리목록",
    "디렉토리복사",
    "디렉토리삭제",
    "디렉토리삭제재귀",
    "디렉토리생성",
    "디렉토리존재",
    "메서드호출",
    "명령실행",
    "명령실행결과",
    "모듈가져오기",
    "문자끝",
    "문자끝제거",
    "문자다듬기",
    "문자대문자",
    "문자반복",
    "문자분할",
    "문자소문자",
    "문자시작",
    "문자시작제거",
    "문자열",
    "문자치환",
    "문자포함",
    "배열꺼내기",
    "배열리듀스",
    "배열맵",
    "배열문자열합치기",
    "배열삭제",
    "배열삽입",
    "배열슬라이스",
    "배열정렬",
    "배열추가",
    "배열필터",
    "배열합치기",
    "베이스육십사디코드",
    "베이스육십사인코드",
    "불린",
    "불변인가",
    "비동기명령실행",
    "비동기명령실행결과",
    "비동기웹요청",
    "비동기가져오기요청",
    "비동기요청받기",
    "비동기통계",
    "비동기큐길이",
    "비동기파일읽기",
    "비동기파일쓰기",
    "비동기파일추가",
    "사용자입력",
    "상대경로",
    "상속확인",
    "상위경로",
    "서버열기",
    "숫자",
    "시간문자열",
    "연결닫기",
    "요청받기",
    "응답보내기",
    "응답본문",
    "응답자료",
    "응답헤더값",
    "웹가져오기",
    "웹요청",
    "인스턴스생성",
    "인수값",
    "인수개수",
    "인수목록",
    "입력",
    "자료문자열화",
    "자료쓰기",
    "자료예쁘게문자열화",
    "자료예쁘게쓰기",
    "자료읽기",
    "자료파싱",
    "자료파일읽기",
    "자료파일쓰기",
    "자료파일예쁘게쓰기",
    "작업경주",
    "작업결과",
    "작업모두",
    "작업상태",
    "작업실패",
    "작업오류",
    "작업오류코드",
    "작업완료",
    "작업취소",
    "절댓값",
    "절대경로",
    "정수",
    "종료",
    "최대",
    "최소",
    "클래스생성",
    "클래스확인",
    "타입",
    "파일복사",
    "파일삭제",
    "파일수정시각",
    "파일쓰기",
    "파일쓰기줄들",
    "파일이동",
    "파일읽기",
    "파일읽기줄들",
    "파일정보",
    "파일존재",
    "파일추가",
    "파일크기",
    "표준에러쓰기",
    "표준에러줄",
    "표준입력읽기",
    "표준출력쓰기",
    "표준출력줄",
    "해시문자열",
    "해시파일",
    "현재디렉토리",
    "현재시간",
    "환경",
    "환경목록",
    "확장자",
    // Async fs/path variants
    "비동기경로이름",
    "비동기경로정규화",
    "비동기경로존재",
    "비동기경로합치기",
    "비동기디렉토리목록",
    "비동기디렉토리복사",
    "비동기디렉토리삭제",
    "비동기디렉토리삭제재귀",
    "비동기디렉토리생성",
    "비동기디렉토리존재",
    "비동기상대경로",
    "비동기상위경로",
    "비동기절대경로",
    "비동기파일복사",
    "비동기파일삭제",
    "비동기파일수정시각",
    "비동기파일쓰기줄들",
    "비동기파일이동",
    "비동기파일읽기줄들",
    "비동기파일정보",
    "비동기파일존재",
    "비동기파일크기",
    "비동기현재디렉토리",
    "비동기확장자",
];

pub fn is_builtin(name: &str) -> bool {
    // Use linear search since Korean UTF-8 strings don't sort the same way as strcmp
    BUILTIN_NAMES.contains(&name)
}

pub fn call_builtin(
    name: &str,
    args: Vec<Value>,
    eval: &mut Evaluator,
    arena: &AstArena,
    env: &mut Environment,
    line: i32,
) -> Result<Value, CokacError> {
    // Fast-path core builtins
    match name {
        "길이" => return core::builtin_length(args, line),
        "단언" => return core::builtin_assert(args, line),
        "문자열" => return core::builtin_to_string(args, line),
        "불린" => return core::builtin_to_bool(args, line),
        "숫자" => return core::builtin_to_number(args, line),
        "절댓값" => return core::builtin_abs(args, line),
        "정수" => return core::builtin_integer(args, line),
        "최대" => return core::builtin_max(args, line),
        "최소" => return core::builtin_min(args, line),
        "타입" => return core::builtin_type(args, line),
        _ => {}
    }

    // String ops
    match name {
        "문자포함" => return string_ops::builtin_contains(args, line),
        "문자치환" => return string_ops::builtin_replace(args, line),
        "문자분할" => return string_ops::builtin_split(args, line),
        "문자시작" => return string_ops::builtin_starts_with(args, line),
        "문자끝" => return string_ops::builtin_ends_with(args, line),
        "문자다듬기" => return string_ops::builtin_trim(args, line),
        "문자대문자" => return string_ops::builtin_to_upper(args, line),
        "문자소문자" => return string_ops::builtin_to_lower(args, line),
        "문자시작제거" => return string_ops::builtin_remove_prefix(args, line),
        "문자끝제거" => return string_ops::builtin_remove_suffix(args, line),
        "문자반복" => return string_ops::builtin_repeat(args, line),
        _ => {}
    }

    // Array ops
    match name {
        "배열추가" => return array_ops::builtin_push(args, line),
        "배열삽입" => return array_ops::builtin_insert(args, line),
        "배열삭제" => return array_ops::builtin_remove(args, line),
        "배열꺼내기" => return array_ops::builtin_pop(args, line),
        "배열슬라이스" => return array_ops::builtin_slice(args, line),
        "배열합치기" => return array_ops::builtin_concat(args, line),
        "배열정렬" => return array_ops::builtin_sort(args, line),
        "배열문자열합치기" => return array_ops::builtin_join(args, line),
        "배열맵" => return array_ops::builtin_map(args, eval, arena, env, line),
        "배열필터" => return array_ops::builtin_filter(args, eval, arena, env, line),
        "배열리듀스" => return array_ops::builtin_reduce(args, eval, arena, env, line),
        _ => {}
    }

    // Object ops
    match name {
        "객체가짐" => return object_ops::builtin_has(args, line),
        "객체설정" => return object_ops::builtin_set(args, line),
        "객체삭제" => return object_ops::builtin_delete(args, line),
        "객체키들" => return object_ops::builtin_keys(args, line),
        "객체값들" => return object_ops::builtin_values(args, line),
        "객체복사" => return object_ops::builtin_clone(args, line),
        "객체합치기" => return object_ops::builtin_merge(args, line),
        _ => {}
    }

    // File ops
    match name {
        "파일읽기" => return file_ops::builtin_file_read(args, eval.runtime, line),
        "파일읽기줄들" => return file_ops::builtin_file_read_lines(args, eval.runtime, line),
        "파일쓰기" => return file_ops::builtin_file_write(args, eval.runtime, line),
        "파일쓰기줄들" => return file_ops::builtin_file_write_lines(args, eval.runtime, line),
        "파일추가" => return file_ops::builtin_file_append(args, eval.runtime, line),
        "파일복사" => return file_ops::builtin_file_copy(args, eval.runtime, line),
        "파일이동" => return file_ops::builtin_file_move(args, eval.runtime, line),
        "파일존재" => return file_ops::builtin_file_exists(args, eval.runtime, line),
        "파일삭제" => return file_ops::builtin_file_delete(args, eval.runtime, line),
        "파일정보" => return file_ops::builtin_file_info(args, eval.runtime, line),
        "파일크기" => return file_ops::builtin_file_size(args, eval.runtime, line),
        "파일수정시각" => return file_ops::builtin_file_mtime(args, eval.runtime, line),
        _ => {}
    }

    // Dir/Path ops
    match name {
        "디렉토리목록" => return dir_path_ops::builtin_dir_list(args, eval.runtime, line),
        "디렉토리생성" => return dir_path_ops::builtin_dir_create(args, eval.runtime, line),
        "디렉토리삭제" => return dir_path_ops::builtin_dir_remove(args, eval.runtime, line),
        "디렉토리삭제재귀" => return dir_path_ops::builtin_dir_remove_recursive(args, eval.runtime, line),
        "디렉토리복사" => return dir_path_ops::builtin_dir_copy(args, eval.runtime, line),
        "디렉토리존재" => return dir_path_ops::builtin_dir_exists(args, eval.runtime, line),
        "현재디렉토리" => return dir_path_ops::builtin_cwd(args, line),
        "경로합치기" => return dir_path_ops::builtin_path_join(args, line),
        "절대경로" => return dir_path_ops::builtin_abs_path(args, eval.runtime, line),
        "경로이름" => return dir_path_ops::builtin_basename(args, line),
        "상위경로" => return dir_path_ops::builtin_dirname(args, line),
        "확장자" => return dir_path_ops::builtin_extension(args, line),
        "경로정규화" => return dir_path_ops::builtin_normalize(args, line),
        "상대경로" => return dir_path_ops::builtin_relative(args, line),
        "경로존재" => return dir_path_ops::builtin_path_exists(args, eval.runtime, line),
        _ => {}
    }

    // JSON ops
    match name {
        "자료파싱" => return json_ops::builtin_json_parse(args, line),
        "자료문자열화" => return json_ops::builtin_json_stringify(args, line),
        "자료예쁘게문자열화" => return json_ops::builtin_json_stringify_pretty(args, line),
        "자료읽기" | "자료파일읽기" => return json_ops::builtin_json_read_file(args, eval.runtime, line),
        "자료쓰기" | "자료파일쓰기" => return json_ops::builtin_json_write_file(args, eval.runtime, line),
        "자료예쁘게쓰기" | "자료파일예쁘게쓰기" => return json_ops::builtin_json_write_file_pretty(args, eval.runtime, line),
        _ => {}
    }

    // Hash/Encoding
    match name {
        "해시문자열" => return hash_encoding::builtin_hash_string(args, line),
        "해시파일" => return hash_encoding::builtin_hash_file(args, eval.runtime, line),
        "베이스육십사인코드" => return hash_encoding::builtin_base64_encode(args, line),
        "베이스육십사디코드" => return hash_encoding::builtin_base64_decode(args, line),
        _ => {}
    }

    // Time/Random
    match name {
        "현재시간" => return time_random::builtin_current_time(args, line),
        "시간문자열" => return time_random::builtin_time_string(args, line),
        "대기밀리초" => return time_random::builtin_sleep(args, line),
        "난수" => return time_random::builtin_random(args, line),
        "난수정수" => return time_random::builtin_random_int(args, line),
        _ => {}
    }

    // Stdio/IO
    match name {
        "입력" => return stdio_io::builtin_input(args, line),
        "사용자입력" => return stdio_io::builtin_input(args, line),
        "표준입력읽기" => return stdio_io::builtin_stdin_read(args, line),
        "표준출력쓰기" => return stdio_io::builtin_stdout_write(args, line),
        "표준출력줄" => return stdio_io::builtin_stdout_writeln(args, line),
        "표준에러쓰기" => return stdio_io::builtin_stderr_write(args, line),
        "표준에러줄" => return stdio_io::builtin_stderr_writeln(args, line),
        _ => {}
    }

    // Process ops
    match name {
        "명령실행" => return process_ops::builtin_shell_exec(args, line),
        "명령실행결과" => return process_ops::builtin_shell_exec_full(args, line),
        "인수값" => return process_ops::builtin_arg_value(args, eval.runtime, line),
        "인수개수" => return process_ops::builtin_arg_count(args, eval.runtime, line),
        "인수목록" => return process_ops::builtin_arg_list(args, eval.runtime, line),
        "종료" => return process_ops::builtin_exit(args, line),
        "환경" => return process_ops::builtin_getenv(args, line),
        "환경목록" => return process_ops::builtin_env_list(args, line),
        _ => {}
    }

    // HTTP client
    match name {
        "웹가져오기" => return http_client::builtin_web_get(args, line),
        "웹요청" => return http_client::builtin_web_request(args, line),
        "가져오기요청" => return http_client::builtin_fetch(args, line),
        "응답본문" => return http_client::builtin_response_body(args, line),
        "응답자료" => return http_client::builtin_response_json(args, line),
        "응답헤더값" => return http_client::builtin_response_header(args, line),
        _ => {}
    }

    // HTTP server
    match name {
        "서버열기" => return http_server::builtin_server_listen(args, line),
        "요청받기" => return http_server::builtin_accept_request(args, line),
        "응답보내기" => return http_server::builtin_send_response(args, line),
        "연결닫기" => return http_server::builtin_close_connection(args, line),
        _ => {}
    }

    // Task ops
    match name {
        "작업완료" => return task_ops::builtin_task_done(args, line),
        "작업실패" => return task_ops::builtin_task_failed(args, line),
        "작업오류" => return task_ops::builtin_task_error(args, line),
        "작업오류코드" => return task_ops::builtin_task_error_code(args, line),
        "작업취소" => return task_ops::builtin_task_cancel(args, line),
        "작업상태" => return task_ops::builtin_task_state(args, line),
        "작업결과" => return task_ops::builtin_task_result(args, line),
        "작업모두" => return task_ops::builtin_task_all(args, eval, line),
        "작업경주" => return task_ops::builtin_task_race(args, eval, line),
        "대기최대" => return task_ops::builtin_await_timeout(args, eval, arena, env, line),
        "비동기큐길이" => return task_ops::builtin_async_queue_length(args, eval.runtime, line),
        "비동기통계" => return task_ops::builtin_async_stats(args, eval.runtime, line),
        _ => {}
    }

    // OOP
    match name {
        "클래스생성" => return oop::builtin_create_class(args, line),
        "인스턴스생성" => return oop::builtin_create_instance(args, eval, arena, env, line),
        "메서드호출" => return oop::builtin_method_call(args, eval, arena, env, line),
        "클래스확인" => return oop::builtin_is_class(args, line),
        "상속확인" => return oop::builtin_inherits(args, line),
        _ => {}
    }

    // Module ops
    match name {
        "내보내기" => return module_ops::builtin_export(args, eval.runtime, line),
        "모듈가져오기" => return module_ops::builtin_module_import(args, eval, arena, env, line),
        _ => {}
    }

    // Misc
    match name {
        "불변인가" => {
            if args.len() != 1 {
                return Err(CokacError::new("'불변인가'는 1개의 인수가 필요합니다.".to_string(), line));
            }
            return Ok(Value::Bool(args[0].is_frozen()));
        }
        _ => {}
    }

    // Async variants — wrap sync version in a task
    if name.starts_with("비동기") {
        return handle_async_builtin(name, args, eval, arena, env, line);
    }

    Err(CokacError::new(
        format!("알 수 없는 내장 함수: '{}'", name),
        line,
    ))
}

fn handle_async_builtin(
    name: &str,
    args: Vec<Value>,
    eval: &mut Evaluator,
    arena: &AstArena,
    env: &mut Environment,
    line: i32,
) -> Result<Value, CokacError> {
    // HTTP ops use thread workers for true concurrency
    if name == "비동기가져오기요청" || name == "비동기웹요청" {
        return handle_async_http(name, args, eval, line);
    }
    if name == "비동기요청받기" {
        return handle_async_accept(args, eval, line);
    }

    // Map async name to sync name
    let sync_name = match name {
        "비동기파일읽기" => "파일읽기",
        "비동기파일쓰기" => "파일쓰기",
        "비동기파일추가" => "파일추가",
        "비동기명령실행" => "명령실행",
        "비동기명령실행결과" => "명령실행결과",
        "비동기경로이름" => "경로이름",
        "비동기경로정규화" => "경로정규화",
        "비동기경로존재" => "경로존재",
        "비동기경로합치기" => "경로합치기",
        "비동기디렉토리목록" => "디렉토리목록",
        "비동기디렉토리복사" => "디렉토리복사",
        "비동기디렉토리삭제" => "디렉토리삭제",
        "비동기디렉토리삭제재귀" => "디렉토리삭제재귀",
        "비동기디렉토리생성" => "디렉토리생성",
        "비동기디렉토리존재" => "디렉토리존재",
        "비동기상대경로" => "상대경로",
        "비동기상위경로" => "상위경로",
        "비동기절대경로" => "절대경로",
        "비동기파일복사" => "파일복사",
        "비동기파일삭제" => "파일삭제",
        "비동기파일수정시각" => "파일수정시각",
        "비동기파일쓰기줄들" => "파일쓰기줄들",
        "비동기파일이동" => "파일이동",
        "비동기파일읽기줄들" => "파일읽기줄들",
        "비동기파일정보" => "파일정보",
        "비동기파일존재" => "파일존재",
        "비동기파일크기" => "파일크기",
        "비동기현재디렉토리" => "현재디렉토리",
        "비동기확장자" => "확장자",
        _ => {
            return Err(CokacError::new(
                format!("알 수 없는 비동기 함수: '{}'", name),
                line,
            ));
        }
    };

    // Execute sync version, but defer the result (task stays pending until drive_async)
    let task = Value::new_task();
    if !eval.runtime.try_enqueue_task(&task, line) {
        return Ok(Value::Task(task));
    }
    match call_builtin(sync_name, args, eval, arena, env, line) {
        Ok(result) => {
            eval.runtime.async_jobs.push(crate::runtime::AsyncJob {
                task: task.clone(),
                kind: crate::runtime::AsyncJobKind::DeferredOk(result),
            });
        }
        Err(err) => {
            eval.runtime.async_jobs.push(crate::runtime::AsyncJob {
                task: task.clone(),
                kind: crate::runtime::AsyncJobKind::DeferredErr {
                    message: err.message,
                    code: err.code.as_str().to_string(),
                    line: err.line,
                    stack: err.stack,
                },
            });
        }
    }
    eval.runtime.mark_async_enqueued();
    Ok(Value::Task(task))
}

fn handle_async_http(
    name: &str,
    args: Vec<Value>,
    eval: &mut Evaluator,
    line: i32,
) -> Result<Value, CokacError> {
    fn env_bool_or_default(key: &str, default: bool) -> bool {
        match std::env::var(key) {
            Ok(v) => {
                let s = v.trim().to_ascii_lowercase();
                matches!(s.as_str(), "1" | "true" | "yes" | "on")
            }
            Err(_) => default,
        }
    }
    fn https_verify_default() -> bool {
        env_bool_or_default("COKAC_HTTPS_VERIFY", true)
    }
    fn security_allow_insecure_https() -> bool {
        env_bool_or_default("COKAC_SECURITY_ALLOW_INSECURE_HTTPS", false)
    }
    fn enforce_https_security_policy(url: &str, https_verify: bool, line: i32) -> Result<(), CokacError> {
        if url.starts_with("https://") && !https_verify && !security_allow_insecure_https() {
            return Err(CokacError::new(
                "보안 정책 위반: HTTPS 요청에서 검증 비활성화는 허용되지 않습니다. (COKAC_SECURITY_ALLOW_INSECURE_HTTPS=1로만 예외 허용)".to_string(),
                line,
            ));
        }
        Ok(())
    }
    fn env_u64_or_default(key: &str, default: u64) -> u64 {
        match std::env::var(key) {
            Ok(v) => v.parse::<u64>().unwrap_or(default),
            Err(_) => default,
        }
    }
    fn url_allowed(url: &str, block_localhost: bool) -> bool {
        if !block_localhost {
            return true;
        }
        let parsed = match reqwest::Url::parse(url) {
            Ok(u) => u,
            Err(_) => return true,
        };
        let host = match parsed.host_str() {
            Some(h) => h.to_ascii_lowercase(),
            None => return true,
        };
        if host == "localhost" || host == "::1" {
            return false;
        }
        if let Ok(ip) = host.parse::<std::net::IpAddr>() {
            return !ip.is_loopback();
        }
        true
    }

    // Extract request parameters as plain strings for thread
    let url = if args.is_empty() {
        return Err(CokacError::new(format!("'{}'에 URL이 필요합니다.", name), line));
    } else {
        args[0].to_display_string()
    };

    let mut method = "GET".to_string();
    let mut req_headers: Vec<(String, String)> = Vec::new();
    let mut body_text: Option<String> = None;
    let mut retry: u64 = env_u64_or_default("COKAC_HTTP_RETRY", 2);
    let mut retry_delay_sec: u64 = env_u64_or_default("COKAC_HTTP_RETRY_DELAY_SEC", 1);
    let mut connect_timeout_secs: u64 = env_u64_or_default("COKAC_HTTP_CONNECT_TIMEOUT_SEC", 10);
    let mut timeout_secs: u64 = env_u64_or_default("COKAC_HTTP_MAX_TIME_SEC", 30);
    let mut max_redirects: u64 = env_u64_or_default("COKAC_HTTP_MAX_REDIRECTS", 5);
    let mut https_verify = https_verify_default();
    let mut block_localhost = env_bool_or_default("COKAC_HTTP_BLOCK_LOCALHOST", true);
    let ca_bundle = std::env::var("COKAC_CA_BUNDLE").ok();

    if name == "비동기웹요청" {
        // 비동기웹요청(메서드, URL, [헤더], [본문])
        if args.len() < 2 {
            return Err(CokacError::new("'비동기웹요청'은 2~4개의 인수가 필요합니다.".to_string(), line));
        }
        method = args[0].to_display_string().to_uppercase();
        let url = args[1].to_display_string();
        if let Some(Value::Object(h)) = args.get(2) {
            let h = h.borrow();
            for (k, v) in h.keys.iter().zip(h.values.iter()) {
                req_headers.push((k.clone(), v.to_display_string()));
            }
        }
        if let Some(v) = args.get(3) {
            body_text = Some(v.to_display_string());
        }
        // Use a separate variable since we shadowed url
        enforce_https_security_policy(&url, https_verify, line)?;
        if !url_allowed(&url, block_localhost) {
            return Err(CokacError::new("URL 검증 실패: 로컬호스트 접근이 차단되었습니다.".to_string(), line));
        }
        return spawn_http_thread(
            url,
            method,
            req_headers,
            body_text,
            retry,
            retry_delay_sec,
            connect_timeout_secs.max(1),
            timeout_secs.max(1),
            max_redirects,
            https_verify,
            ca_bundle,
            line,
            eval,
        );
    }

    // 비동기가져오기요청(URL, [옵션])
    if let Some(Value::Object(opts)) = args.get(1) {
        let opts = opts.borrow();
        if let Some(Value::String(m)) = opts.get("메서드") {
            method = m.to_uppercase();
        }
        if let Some(Value::Object(h)) = opts.get("헤더") {
            let h = h.borrow();
            for (k, v) in h.keys.iter().zip(h.values.iter()) {
                req_headers.push((k.clone(), v.to_display_string()));
            }
        }
        if let Some(Value::String(b)) = opts.get("본문") {
            body_text = Some(b.clone());
        }
        if let Some(Value::Bool(v)) = opts.get("HTTPS검증") {
            https_verify = *v;
        }
        if let Some(val) = opts.get("최대시간초") {
            if let Ok(n) = crate::value::value_to_number(val, line) {
                timeout_secs = n as u64;
            }
        }
        if let Some(val) = opts.get("재시도") {
            if let Ok(n) = crate::value::value_to_number(val, line) {
                retry = n.max(0.0) as u64;
            }
        }
        if let Some(val) = opts.get("재시도지연초") {
            if let Ok(n) = crate::value::value_to_number(val, line) {
                retry_delay_sec = n.max(0.0) as u64;
            }
        }
        if let Some(val) = opts.get("연결시간초") {
            // Use connection timeout as overall timeout if specified
            if let Ok(n) = crate::value::value_to_number(val, line) {
                if n > 0.0 {
                    connect_timeout_secs = n as u64;
                }
            }
        }
        if let Some(val) = opts.get("리다이렉트최대") {
            if let Ok(n) = crate::value::value_to_number(val, line) {
                max_redirects = n.max(0.0) as u64;
            }
        }
        if let Some(val) = opts.get("로컬차단") {
            block_localhost = val.is_truthy();
        }
    }

    enforce_https_security_policy(&url, https_verify, line)?;
    if !url_allowed(&url, block_localhost) {
        return Err(CokacError::new("URL 검증 실패: 로컬호스트 접근이 차단되었습니다.".to_string(), line));
    }
    spawn_http_thread(
        url,
        method,
        req_headers,
        body_text,
        retry,
        retry_delay_sec,
        connect_timeout_secs.max(1),
        timeout_secs.max(1),
        max_redirects,
        https_verify,
        ca_bundle,
        line,
        eval,
    )
}

fn spawn_http_thread(
    url: String,
    method: String,
    req_headers: Vec<(String, String)>,
    body_text: Option<String>,
    retry: u64,
    retry_delay_sec: u64,
    connect_timeout_secs: u64,
    timeout_secs: u64,
    max_redirects: u64,
    https_verify: bool,
    ca_bundle: Option<String>,
    line: i32,
    eval: &mut Evaluator,
) -> Result<Value, CokacError> {
    let task = Value::new_task();
    if !eval.runtime.try_enqueue_task(&task, line) {
        return Ok(Value::Task(task));
    }
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => { let _ = tx.send(crate::runtime::ThreadResult::Error(e.to_string())); return; }
        };
        let result = rt.block_on(async {
            let mut last_err = String::new();
            for attempt in 0..=retry {
                let mut builder = reqwest::Client::builder()
                    .connect_timeout(std::time::Duration::from_secs(connect_timeout_secs))
                    .timeout(std::time::Duration::from_secs(timeout_secs))
                    .redirect(reqwest::redirect::Policy::limited(max_redirects as usize))
                    .danger_accept_invalid_certs(!https_verify);
                if let Some(path) = &ca_bundle {
                    if let Ok(pem) = std::fs::read(path) {
                        if let Ok(cert) = reqwest::Certificate::from_pem(&pem) {
                            builder = builder.add_root_certificate(cert);
                        }
                    }
                }
                let client = builder.build().map_err(|e| e.to_string())?;
                let mut req = match method.as_str() {
                    "GET" => client.get(&url),
                    "POST" => client.post(&url),
                    "PUT" => client.put(&url),
                    "DELETE" => client.delete(&url),
                    "PATCH" => client.patch(&url),
                    "HEAD" => client.head(&url),
                    _ => return Err(format!("지원되지 않는 HTTP 메서드: {}", method)),
                };
                for (k, v) in &req_headers {
                    req = req.header(k.as_str(), v.as_str());
                }
                if let Some(ref body) = body_text {
                    req = req.body(body.clone());
                }
                match req.send().await {
                    Ok(resp) => {
                        let status = resp.status().as_u16();
                        let mut headers = Vec::new();
                        let mut headers_raw = String::new();
                        for (name, value) in resp.headers() {
                            if let Ok(v) = value.to_str() {
                                headers.push((name.as_str().to_string(), v.to_string()));
                                headers_raw.push_str(name.as_str());
                                headers_raw.push_str(": ");
                                headers_raw.push_str(v);
                                headers_raw.push_str("\r\n");
                            }
                        }
                        let body = resp.text().await.map_err(|e| e.to_string())?;
                        return Ok((status, body, headers, headers_raw));
                    }
                    Err(e) => {
                        last_err = e.to_string();
                        if attempt < retry {
                            tokio::time::sleep(std::time::Duration::from_secs(retry_delay_sec)).await;
                        }
                    }
                }
            }
            Err(last_err)
        });
        match result {
            Ok((status, body, headers, headers_raw)) => {
                let _ = tx.send(crate::runtime::ThreadResult::HttpResponse {
                    status,
                    body,
                    headers,
                    headers_raw,
                    method: method.clone(),
                    url: url.clone(),
                });
            }
            Err(msg) => {
                let _ = tx.send(crate::runtime::ThreadResult::Error(msg));
            }
        }
    });

    eval.runtime.async_jobs.push(crate::runtime::AsyncJob {
        task: task.clone(),
        kind: crate::runtime::AsyncJobKind::ThreadWorker { receiver: rx },
    });
    eval.runtime.mark_async_enqueued();
    Ok(Value::Task(task))
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(10 + (b - b'a')),
        b'A'..=b'F' => Some(10 + (b - b'A')),
        _ => None,
    }
}

fn percent_decode_lossy(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                out.push((h << 4) | l);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

fn handle_async_accept(
    args: Vec<Value>,
    eval: &mut Evaluator,
    line: i32,
) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'비동기요청받기'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let fd = crate::value::value_to_number(&args[0], line)? as i32;
    let task = Value::new_task();
    if !eval.runtime.try_enqueue_task(&task, line) {
        return Ok(Value::Task(task));
    }
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        use std::io::{BufRead, BufReader, Read};
        use std::net::TcpListener;
        use std::os::unix::io::FromRawFd;

        let listener = unsafe { TcpListener::from_raw_fd(fd) };
        let accept_result = listener.accept();
        // Don't drop listener (would close fd)
        std::mem::forget(listener);

        match accept_result {
            Ok((stream, addr)) => {
                stream.set_read_timeout(Some(std::time::Duration::from_secs(30))).ok();
                let mut reader = BufReader::new(&stream);
                let mut request_text = String::new();
                loop {
                    let mut line_buf = String::new();
                    match reader.read_line(&mut line_buf) {
                        Ok(0) => break,
                        Ok(_) => {
                            request_text.push_str(&line_buf);
                            if line_buf == "\r\n" || line_buf == "\n" { break; }
                        }
                        Err(_) => break,
                    }
                }
                let first_line = request_text.lines().next().unwrap_or("");
                let parts: Vec<&str> = first_line.split_whitespace().collect();
                let method = parts.get(0).unwrap_or(&"GET").to_string();
                let path_raw = parts.get(1).unwrap_or(&"/").to_string();
                let path = percent_decode_lossy(&path_raw);
                let version = parts.get(2).unwrap_or(&"HTTP/1.1").to_string();
                let mut headers = Vec::new();
                let mut content_length: usize = 0;
                for line_str in request_text.lines().skip(1) {
                    if line_str.is_empty() || line_str == "\r" { break; }
                    if let Some(colon_pos) = line_str.find(':') {
                        let key = line_str[..colon_pos].trim().to_string();
                        let val = line_str[colon_pos + 1..].trim().to_string();
                        if key.to_lowercase() == "content-length" {
                            content_length = val.parse().unwrap_or(0);
                        }
                        headers.push((key, val));
                    }
                }
                let body = if content_length > 0 {
                    let mut body_buf = vec![0u8; content_length];
                    reader.read_exact(&mut body_buf).ok();
                    String::from_utf8_lossy(&body_buf).to_string()
                } else {
                    String::new()
                };
                use std::os::unix::io::IntoRawFd;
                let client_fd = stream.into_raw_fd();
                let _ = tx.send(crate::runtime::ThreadResult::ServerAccept {
                    client_fd, method, path, version, headers, body,
                    remote_addr: addr.ip().to_string(),
                });
            }
            Err(e) => {
                let _ = tx.send(crate::runtime::ThreadResult::Error(format!("accept 실패: {}", e)));
            }
        }
    });

    eval.runtime.async_jobs.push(crate::runtime::AsyncJob {
        task: task.clone(),
        kind: crate::runtime::AsyncJobKind::ThreadWorker { receiver: rx },
    });
    eval.runtime.mark_async_enqueued();
    Ok(Value::Task(task))
}
