use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use crate::error::CokacError;
use crate::json;
use crate::value::*;

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

fn security_audit_log(level: &str, event: &str, detail: &str) {
    let to_stderr = env_bool_or_default("COKAC_SECURITY_AUDIT_STDERR", false);
    let path = std::env::var("COKAC_SECURITY_AUDIT_LOG").ok();
    if !to_stderr && path.is_none() {
        return;
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let line = format!(
        "{} level={} event={} detail={}\n",
        now, level, event, detail
    );
    if to_stderr {
        eprint!("{}", line);
    }
    if let Some(p) = path {
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(p)
            .and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes()));
    }
}

fn enforce_https_security_policy(url: &str, https_verify: bool, line: i32) -> Result<(), CokacError> {
    if url.starts_with("https://") && !https_verify && !security_allow_insecure_https() {
        security_audit_log(
            "ERROR",
            "https_verify_override_blocked",
            &format!("url={} verify=false allow_insecure=false", url),
        );
        return Err(CokacError::new(
            "보안 정책 위반: HTTPS 요청에서 검증 비활성화는 허용되지 않습니다. (COKAC_SECURITY_ALLOW_INSECURE_HTTPS=1로만 예외 허용)".to_string(),
            line,
        ));
    }
    if url.starts_with("https://") && !https_verify && security_allow_insecure_https() {
        security_audit_log(
            "WARN",
            "https_verify_override_allowed",
            &format!("url={} verify=false allow_insecure=true", url),
        );
    }
    Ok(())
}

fn env_u64_or_default(key: &str, default: u64) -> u64 {
    match std::env::var(key) {
        Ok(v) => v.parse::<u64>().unwrap_or(default),
        Err(_) => default,
    }
}

#[derive(Clone)]
struct HttpConfig {
    retry: u64,
    retry_delay_sec: u64,
    connect_timeout_sec: u64,
    max_time_sec: u64,
    max_redirects: u64,
    https_verify: bool,
    block_localhost: bool,
    ca_bundle: Option<String>,
}

impl HttpConfig {
    fn from_env() -> Self {
        HttpConfig {
            retry: env_u64_or_default("COKAC_HTTP_RETRY", 2),
            retry_delay_sec: env_u64_or_default("COKAC_HTTP_RETRY_DELAY_SEC", 1),
            connect_timeout_sec: env_u64_or_default("COKAC_HTTP_CONNECT_TIMEOUT_SEC", 10),
            max_time_sec: env_u64_or_default("COKAC_HTTP_MAX_TIME_SEC", 30),
            max_redirects: env_u64_or_default("COKAC_HTTP_MAX_REDIRECTS", 5),
            https_verify: https_verify_default(),
            block_localhost: env_bool_or_default("COKAC_HTTP_BLOCK_LOCALHOST", true),
            ca_bundle: std::env::var("COKAC_CA_BUNDLE").ok(),
        }
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

fn parse_int_opt(v: &Value, line: i32) -> Result<u64, CokacError> {
    let n = crate::value::value_to_number(v, line)?;
    if n < 0.0 {
        return Err(CokacError::new("옵션 값은 0 이상이어야 합니다.".to_string(), line));
    }
    Ok(n as u64)
}

pub fn builtin_web_get(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'웹가져오기'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let url = args[0].to_display_string();
    let cfg = HttpConfig::from_env();
    enforce_https_security_policy(&url, cfg.https_verify, line)?;
    if !url_allowed(&url, cfg.block_localhost) {
        return Err(CokacError::new("URL 검증 실패: 로컬호스트 접근이 차단되었습니다.".to_string(), line));
    }

    let rt = tokio::runtime::Runtime::new().map_err(|e| {
        CokacError::new(format!("HTTP 런타임 생성 실패: {}", e), line)
    })?;

    let result = rt.block_on(async {
        let mut last_err = String::new();
        for attempt in 0..=cfg.retry {
            let mut builder = reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(cfg.connect_timeout_sec.max(1)))
                .timeout(Duration::from_secs(cfg.max_time_sec.max(1)))
                .redirect(reqwest::redirect::Policy::limited(cfg.max_redirects as usize))
                .danger_accept_invalid_certs(!cfg.https_verify);
            if let Some(path) = &cfg.ca_bundle {
                if let Ok(pem) = std::fs::read(path) {
                    if let Ok(cert) = reqwest::Certificate::from_pem(&pem) {
                        builder = builder.add_root_certificate(cert);
                    }
                }
            }
            let client = builder.build().map_err(|e| format!("HTTP 클라이언트 생성 실패: {}", e))?;
            match client.get(&url).send().await {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    if !(200..300).contains(&status) {
                        last_err = format!("HTTP 상태 코드: {}", status);
                    } else {
                        let body = resp.text().await.map_err(|e| format!("응답 읽기 실패: {}", e))?;
                        return Ok(body);
                    }
                }
                Err(e) => last_err = format!("HTTP 요청 실패: {}", e),
            }
            if attempt < cfg.retry {
                tokio::time::sleep(Duration::from_secs(cfg.retry_delay_sec)).await;
            }
        }
        Err(last_err)
    });

    match result {
        Ok(body) => Ok(Value::String(body)),
        Err(msg) => Err(CokacError::new(msg, line)),
    }
}

pub fn builtin_web_request(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() < 2 || args.len() > 4 {
        return Err(CokacError::new("'웹요청'은 2~4개의 인수가 필요합니다.".to_string(), line));
    }
    let method = args[0].to_display_string().to_uppercase();
    let url = args[1].to_display_string();
    let headers_val = args.get(2);
    let body_text = args.get(3).map(|v| v.to_display_string());
    let mut headers_obj: Option<ObjectValue> = None;
    if let Some(Value::Object(obj)) = headers_val {
        let obj = obj.borrow();
        let mut copy = ObjectValue::new();
        for (k, v) in obj.keys.iter().zip(obj.values.iter()) {
            copy.set(k.clone(), v.clone());
        }
        headers_obj = Some(copy);
    }
    let cfg = HttpConfig::from_env();
    execute_web_request(method, url, headers_obj, body_text, cfg, line)
}

fn execute_web_request(
    method: String,
    url: String,
    headers_obj: Option<ObjectValue>,
    body_text: Option<String>,
    cfg: HttpConfig,
    line: i32,
) -> Result<Value, CokacError> {
    enforce_https_security_policy(&url, cfg.https_verify, line)?;
    if !url_allowed(&url, cfg.block_localhost) {
        return Err(CokacError::new("URL 검증 실패: 로컬호스트 접근이 차단되었습니다.".to_string(), line));
    }

    let rt = tokio::runtime::Runtime::new().map_err(|e| {
        CokacError::new(format!("HTTP 런타임 생성 실패: {}", e), line)
    })?;

    let result = rt.block_on(async {
        let mut last_err = String::new();
        for attempt in 0..=cfg.retry {
            let mut builder = reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(cfg.connect_timeout_sec.max(1)))
                .timeout(Duration::from_secs(cfg.max_time_sec.max(1)))
                .redirect(reqwest::redirect::Policy::limited(cfg.max_redirects as usize))
                .danger_accept_invalid_certs(!cfg.https_verify);
            if let Some(path) = &cfg.ca_bundle {
                if let Ok(pem) = std::fs::read(path) {
                    if let Ok(cert) = reqwest::Certificate::from_pem(&pem) {
                        builder = builder.add_root_certificate(cert);
                    }
                }
            }
            let client = builder
                .build()
                .map_err(|e| format!("HTTP 클라이언트 생성 실패: {}", e))?;

            let mut req = match method.as_str() {
                "GET" => client.get(&url),
                "POST" => client.post(&url),
                "PUT" => client.put(&url),
                "DELETE" => client.delete(&url),
                "PATCH" => client.patch(&url),
                "HEAD" => client.head(&url),
                _ => return Err(format!("지원되지 않는 HTTP 메서드: {}", method)),
            };

            if let Some(obj) = &headers_obj {
                for (key, val) in obj.keys.iter().zip(obj.values.iter()) {
                    req = req.header(key.as_str(), val.to_display_string());
                }
            }

            if let Some(ref body) = body_text {
                req = req.body(body.clone());
            }

            match req.send().await {
                Ok(resp) => {
                    let status = resp.status().as_u16();
                    let mut resp_headers = ObjectValue::new();
                    let mut resp_headers_raw = String::new();
                    for (name, value) in resp.headers() {
                        if let Ok(v) = value.to_str() {
                            resp_headers.set(name.as_str().to_string(), Value::String(v.to_string()));
                            resp_headers_raw.push_str(name.as_str());
                            resp_headers_raw.push_str(": ");
                            resp_headers_raw.push_str(v);
                            resp_headers_raw.push_str("\r\n");
                        }
                    }
                    let body = resp.text().await.map_err(|e| format!("응답 읽기 실패: {}", e))?;
                    return Ok((status, body, resp_headers, resp_headers_raw));
                }
                Err(e) => {
                    last_err = format!("HTTP 요청 실패: {}", e);
                    if attempt < cfg.retry {
                        tokio::time::sleep(Duration::from_secs(cfg.retry_delay_sec)).await;
                    }
                }
            }
        }
        Err(last_err)
    });

    match result {
        Ok((status, body, headers, headers_raw)) => {
            Ok(make_response_object(status, body, headers, headers_raw, method, url))
        }
        Err(msg) => Err(CokacError::new(msg, line)),
    }
}

pub fn builtin_fetch(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.is_empty() || args.len() > 2 {
        return Err(CokacError::new("'가져오기요청'은 1~2개의 인수가 필요합니다.".to_string(), line));
    }
    let url = args[0].to_display_string();

    // Parse options
    let mut method = "GET".to_string();
    let mut headers_obj: Option<ObjectValue> = None;
    let mut body_text: Option<String> = None;
    let mut cfg = HttpConfig::from_env();

    if let Some(Value::Object(opts)) = args.get(1) {
        let opts = opts.borrow();
        if let Some(Value::String(m)) = opts.get("메서드") {
            method = m.to_uppercase();
        }
        if let Some(Value::Object(h)) = opts.get("헤더") {
            let h = h.borrow();
            let mut hobj = ObjectValue::new();
            for (k, v) in h.keys.iter().zip(h.values.iter()) {
                hobj.set(k.clone(), v.clone());
            }
            headers_obj = Some(hobj);
        }
        if let Some(Value::String(b)) = opts.get("본문") {
            body_text = Some(b.clone());
        }
        if let Some(Value::Bool(v)) = opts.get("HTTPS검증") {
            cfg.https_verify = *v;
        }
        if let Some(v) = opts.get("재시도") {
            cfg.retry = parse_int_opt(v, line)?;
        }
        if let Some(v) = opts.get("재시도지연초") {
            cfg.retry_delay_sec = parse_int_opt(v, line)?;
        }
        if let Some(v) = opts.get("연결시간초") {
            cfg.connect_timeout_sec = parse_int_opt(v, line)?.max(1);
        }
        if let Some(v) = opts.get("최대시간초") {
            cfg.max_time_sec = parse_int_opt(v, line)?.max(1);
        }
        if let Some(v) = opts.get("리다이렉트최대") {
            cfg.max_redirects = parse_int_opt(v, line)?;
        }
        if let Some(v) = opts.get("로컬차단") {
            cfg.block_localhost = v.is_truthy();
        }
    }
    enforce_https_security_policy(&url, cfg.https_verify, line)?;
    if !url_allowed(&url, cfg.block_localhost) {
        return Err(CokacError::new("URL 검증 실패: 로컬호스트 접근이 차단되었습니다.".to_string(), line));
    }

    execute_web_request(method, url, headers_obj, body_text, cfg, line)
}

fn make_response_object(
    status: u16,
    body: String,
    headers: ObjectValue,
    headers_raw: String,
    method: String,
    url: String,
) -> Value {
    let obj = ObjectValue::new();
    let obj = Rc::new(RefCell::new(obj));
    {
        let mut o = obj.borrow_mut();
        o.set("상태".to_string(), Value::Number(status as f64));
        o.set("본문".to_string(), Value::String(body));
        o.set("성공".to_string(), Value::Bool(status >= 200 && status < 300));
        o.set("헤더".to_string(), Value::String(headers_raw));
        o.set("헤더들".to_string(), Value::Object(Rc::new(RefCell::new(headers))));
        o.set("메서드".to_string(), Value::String(method));
        o.set("주소".to_string(), Value::String(url));
    }
    Value::Object(obj)
}

pub fn builtin_response_body(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'응답본문'은 1개의 인수가 필요합니다.".to_string(), line));
    }
    match &args[0] {
        Value::Object(obj) => {
            let obj = obj.borrow();
            match obj.get("본문") {
                Some(v) => Ok(Value::String(v.to_display_string())),
                None => Ok(Value::String(String::new())),
            }
        }
        _ => Ok(Value::String(String::new())),
    }
}

pub fn builtin_response_json(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'응답자료'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    match &args[0] {
        Value::Object(obj) => {
            let obj = obj.borrow();
            match obj.get("본문") {
                Some(v) => {
                    let text = v.to_display_string();
                    json::json_parse(&text, line)
                }
                None => json::json_parse("", line),
            }
        }
        _ => json::json_parse("", line),
    }
}

pub fn builtin_response_header(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 2 {
        return Err(CokacError::new("'응답헤더값'은 2개의 인수가 필요합니다.".to_string(), line));
    }
    let header_name = args[1].to_display_string();
    match &args[0] {
        Value::Object(obj) => {
            let obj = obj.borrow();
            if let Some(Value::Object(headers)) = obj.get("헤더들") {
                let headers = headers.borrow();
                // Try exact match
                if let Some(v) = headers.get(&header_name) {
                    return Ok(v.clone());
                }
                // Try lowercase
                let lower = header_name.to_lowercase();
                if let Some(v) = headers.get(&lower) {
                    return Ok(v.clone());
                }
                // Try case-insensitive search
                for (k, v) in headers.keys.iter().zip(headers.values.iter()) {
                    if k.to_lowercase() == lower {
                        return Ok(v.clone());
                    }
                }
            }
            Ok(Value::Nil)
        }
        _ => Ok(Value::Nil),
    }
}
