use crate::error::CokacError;
use crate::runtime::Runtime;
use crate::value::Value;

const FNV1A_OFFSET: u64 = 0xcbf29ce484222325;
const FNV1A_PRIME: u64 = 0x100000001b3;

fn fnv1a_64(data: &[u8]) -> u64 {
    let mut hash = FNV1A_OFFSET;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV1A_PRIME);
    }
    hash
}

fn hash_to_hex(hash: u64) -> String {
    format!("{:016x}", hash)
}

pub fn builtin_hash_string(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'해시문자열'은 1개의 인수가 필요합니다.".to_string(), line));
    }
    let text = args[0].to_display_string();
    let hash = fnv1a_64(text.as_bytes());
    Ok(Value::String(hash_to_hex(hash)))
}

pub fn builtin_hash_file(args: Vec<Value>, runtime: &Runtime, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'해시파일'은 1개의 인수가 필요합니다.".to_string(), line));
    }
    let path = runtime.resolve_path(&args[0].to_display_string());
    let data = std::fs::read(&path).map_err(|e| {
        CokacError::new(format!("파일을 읽을 수 없습니다: '{}': {}", path, e), line)
    })?;
    let hash = fnv1a_64(&data);
    Ok(Value::String(hash_to_hex(hash)))
}

pub fn builtin_base64_encode(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'베이스육십사인코드'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let text = args[0].to_display_string();
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
    Ok(Value::String(encoded))
}

pub fn builtin_base64_decode(args: Vec<Value>, line: i32) -> Result<Value, CokacError> {
    if args.len() != 1 {
        return Err(CokacError::new("'베이스육십사디코드'는 1개의 인수가 필요합니다.".to_string(), line));
    }
    let encoded = args[0].to_display_string();
    use base64::Engine;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(encoded.as_bytes())
        .map_err(|e| CokacError::new(format!("베이스육십사디코드 실패: {}", e), line))?;
    let text = String::from_utf8(decoded)
        .map_err(|e| CokacError::new(format!("디코딩 결과가 유효한 UTF-8이 아닙니다: {}", e), line))?;
    Ok(Value::String(text))
}
