use std::path::Path;
use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("사용법: {} <파일.cokac> [인수...]", args[0]);
        eprintln!("        {} --test <파일_또는_디렉토리> [...]", args[0]);
        eprintln!("        {} --test-json <파일_또는_디렉토리> [...]", args[0]);
        std::process::exit(1);
    }

    match args[1].as_str() {
        "--test" => {
            if args.len() < 3 {
                eprintln!("사용법: {} --test <파일_또는_디렉토리> [...]", args[0]);
                std::process::exit(1);
            }
            let code = run_test_mode(&args[0], &args[2..], false);
            std::process::exit(code);
        }
        "--test-json" => {
            if args.len() < 3 {
                println!("{{\"status\":\"error\",\"message\":\"사용법: {} --test-json <파일_또는_디렉토리>\"}}",
                    args[0]);
                std::process::exit(1);
            }
            let code = run_test_mode(&args[0], &args[2..], true);
            std::process::exit(code);
        }
        _ => {
            let script_path = &args[1];
            if !script_path.ends_with(".cokac") {
                eprintln!("오류: 파일 확장자가 .cokac이어야 합니다: {}", script_path);
                std::process::exit(1);
            }
            let script_args = args[2..].to_vec();
            match cokaclang::run_script(script_path, script_args) {
                Ok(()) => {}
                Err(msg) => {
                    eprintln!("{}", msg);
                    std::process::exit(1);
                }
            }
        }
    }
}

fn collect_test_files(path: &str) -> Vec<String> {
    let p = Path::new(path);
    if p.is_file() {
        if path.ends_with(".cokac") {
            return vec![path.to_string()];
        }
        return Vec::new();
    }
    if p.is_dir() {
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(p) {
            for entry in entries.flatten() {
                let child = entry.path().to_string_lossy().to_string();
                files.extend(collect_test_files(&child));
            }
        }
        return files;
    }
    Vec::new()
}

fn run_test_mode(exe: &str, targets: &[String], json_mode: bool) -> i32 {
    let mut test_files = Vec::new();
    for target in targets {
        test_files.extend(collect_test_files(target));
    }

    if test_files.is_empty() {
        if json_mode {
            println!("{{\"status\":\"error\",\"message\":\"테스트 파일을 찾을 수 없습니다.\"}}");
        } else {
            eprintln!("오류: 테스트 파일을 찾을 수 없습니다.");
        }
        return 1;
    }

    test_files.sort();

    let total = test_files.len();
    let mut pass_count = 0;
    let mut fail_count = 0;

    if !json_mode {
        println!("총 {}개 테스트 실행", total);
    }

    let mut results: Vec<(String, bool)> = Vec::new();

    for file in &test_files {
        let status = Command::new(exe)
            .arg(file)
            .stdout(if json_mode { std::process::Stdio::null() } else { std::process::Stdio::inherit() })
            .stderr(if json_mode { std::process::Stdio::null() } else { std::process::Stdio::inherit() })
            .status();

        let passed = match status {
            Ok(s) => s.success(),
            Err(_) => false,
        };

        if passed {
            pass_count += 1;
        } else {
            fail_count += 1;
        }

        if !json_mode {
            if passed {
                println!("[PASS] {}", file);
            } else {
                println!("[FAIL] {}", file);
            }
        }

        results.push((file.clone(), passed));
    }

    if json_mode {
        print!("{{\"status\":\"ok\",\"results\":[");
        for (i, (file, passed)) in results.iter().enumerate() {
            if i > 0 { print!(","); }
            print!("{{\"file\":\"{}\",\"status\":\"{}\"}}", json_escape(file), if *passed { "pass" } else { "fail" });
        }
        println!("],\"total\":{},\"pass\":{},\"fail\":{}}}", total, pass_count, fail_count);
    } else {
        println!("\n결과: {} 통과, {} 실패 / 총 {}", pass_count, fail_count, total);
    }

    if fail_count > 0 { 1 } else { 0 }
}

fn json_escape(s: &str) -> String {
    let mut result = String::new();
    for ch in s.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result
}
