// Use *codemirror (bundle mode) to avoid duplicate @codemirror/state instances
import { basicSetup, EditorView, keymap } from "https://esm.sh/*codemirror@6.0.1";

// --- Custom dark theme (avoids importing @codemirror/theme-one-dark which causes conflicts) ---
const darkTheme = EditorView.theme({
    "&": {
        backgroundColor: "#1f2335",
        color: "#c0caf5",
        height: "100%",
    },
    ".cm-content": {
        caretColor: "#7aa2f7",
        fontFamily: '"JetBrains Mono", "Fira Code", "Cascadia Code", monospace',
    },
    ".cm-cursor, .cm-dropCursor": {
        borderLeftColor: "#7aa2f7",
    },
    "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection": {
        backgroundColor: "rgba(122, 162, 247, 0.2)",
    },
    ".cm-panels": {
        backgroundColor: "#1a1b26",
        color: "#c0caf5",
    },
    ".cm-panels.cm-panels-top": {
        borderBottom: "1px solid #3b4261",
    },
    ".cm-panels.cm-panels-bottom": {
        borderTop: "1px solid #3b4261",
    },
    ".cm-searchMatch": {
        backgroundColor: "rgba(122, 162, 247, 0.3)",
    },
    ".cm-searchMatch.cm-searchMatch-selected": {
        backgroundColor: "rgba(122, 162, 247, 0.5)",
    },
    ".cm-activeLine": {
        backgroundColor: "rgba(122, 162, 247, 0.06)",
    },
    ".cm-selectionMatch": {
        backgroundColor: "rgba(122, 162, 247, 0.15)",
    },
    ".cm-matchingBracket, .cm-nonmatchingBracket": {
        backgroundColor: "rgba(122, 162, 247, 0.25)",
        outline: "1px solid rgba(122, 162, 247, 0.5)",
    },
    ".cm-gutters": {
        backgroundColor: "#24283b",
        color: "#565f89",
        borderRight: "1px solid #3b4261",
    },
    ".cm-activeLineGutter": {
        backgroundColor: "#1f2335",
        color: "#c0caf5",
    },
    ".cm-foldPlaceholder": {
        backgroundColor: "transparent",
        border: "none",
        color: "#565f89",
    },
    ".cm-tooltip": {
        border: "1px solid #3b4261",
        backgroundColor: "#24283b",
        color: "#c0caf5",
    },
    ".cm-tooltip .cm-tooltip-arrow:before": {
        borderTopColor: "transparent",
        borderBottomColor: "transparent",
    },
    ".cm-tooltip .cm-tooltip-arrow:after": {
        borderTopColor: "#24283b",
        borderBottomColor: "#24283b",
    },
    ".cm-tooltip-autocomplete": {
        "& > ul > li[aria-selected]": {
            backgroundColor: "rgba(122, 162, 247, 0.2)",
        },
    },
    ".cm-scroller": {
        overflow: "auto",
    },
}, { dark: true });

// --- WASM loading ---
let wasmModule = null;

async function loadWasm() {
    const output = document.getElementById("output");
    output.textContent = "WASM 모듈 로딩 중...";
    output.className = "loading";
    try {
        const mod = await import("./pkg/cokaclang.js");
        await mod.default();
        wasmModule = mod;
        output.textContent = "준비 완료! 코드를 작성하고 실행 버튼을 누르세요.";
        output.className = "";
    } catch (e) {
        output.textContent = `WASM 로딩 실패: ${e.message}\n\npkg/ 디렉토리에 WASM 빌드가 있는지 확인하세요.\nbuildweb.sh 를 실행하여 빌드할 수 있습니다.`;
        output.className = "has-error";
    }
}

// --- Examples ---
const EXAMPLES = {
    hello: `출력("안녕하세요, 코카클랭!")
출력("Hello, Cokaclang!")`,

    variables: `변수 이름 = "세계"
변수 나이 = 2024
변수 실수 = 3.14

출력("안녕, " + 이름 + "!")
출력("원주율: " + 문자열(실수))
출력("합계: " + 문자열(나이 + 1))`,

    loop: `// 반복문
변수 합 = 0
반복 (변수 i = 1; i <= 10; i = i + 1) {
    합 = 합 + i
}
출력("1부터 10까지의 합: " + 문자열(합))

// 조건문
만약 (합 > 50) {
    출력("합이 50보다 큽니다")
} 아니면 {
    출력("합이 50 이하입니다")
}`,

    function: `함수 인사(이름) {
    반환 "안녕하세요, " + 이름 + "님!"
}

함수 팩토리얼(n) {
    만약 (n <= 1) {
        반환 1
    }
    반환 n * 팩토리얼(n - 1)
}

출력(인사("코카클랭"))
출력("5! = " + 문자열(팩토리얼(5)))
출력("10! = " + 문자열(팩토리얼(10)))`,

    array: `변수 과일 = ["사과", "바나나", "딸기", "포도"]

출력("과일 목록:")
반복 (변수 i = 0; i < 길이(과일); i = i + 1) {
    출력("  " + 문자열(i + 1) + ". " + 과일[i])
}

배열추가(과일, "수박")
출력("\\n추가 후: " + 문자열(길이(과일)) + "개")

변수 결과 = 배열맵(과일, 함수(x) { 반환 x + "!" })
출력("맵 결과: " + 문자열(결과))`,

    object: `변수 사람 = {
    "이름": "홍길동",
    "나이": 30,
    "취미": ["독서", "코딩"]
}

출력("이름: " + 사람.이름)
출력("나이: " + 문자열(사람.나이))
출력("취미: " + 문자열(사람.취미))

객체설정(사람, "직업", "개발자")
출력("직업: " + 사람.직업)
출력("키 목록: " + 문자열(객체키들(사람)))`,

    fibonacci: `함수 피보나치(n) {
    만약 (n <= 0) { 반환 0 }
    만약 (n == 1) { 반환 1 }

    변수 a = 0
    변수 b = 1
    반복 (변수 i = 2; i <= n; i = i + 1) {
        변수 temp = b
        b = a + b
        a = temp
    }
    반환 b
}

출력("피보나치 수열 (처음 15개):")
변수 결과 = ""
반복 (변수 i = 0; i < 15; i = i + 1) {
    만약 (i > 0) { 결과 = 결과 + ", " }
    결과 = 결과 + 문자열(피보나치(i))
}
출력(결과)`,

    class: `변수 동물 = 클래스생성("동물", 거짓, {
    "초기화": 함수(자신, 이름, 소리) {
        자신.이름 = 이름
        자신.소리 = 소리
    },
    "울기": 함수(자신) {
        출력(자신.이름 + ": " + 자신.소리 + "!")
    },
    "소개": 함수(자신) {
        출력("저는 " + 자신.이름 + "입니다.")
    }
})

변수 고양이 = 인스턴스생성(동물, "고양이", "야옹")
변수 강아지 = 인스턴스생성(동물, "강아지", "멍멍")

메서드호출(고양이, "소개")
메서드호출(고양이, "울기")
메서드호출(강아지, "소개")
메서드호출(강아지, "울기")`,
};

// --- Editor setup ---
const defaultCode = `// 코카클랭 플레이그라운드에 오신 것을 환영합니다!
// 위의 "예제 선택"에서 예제를 골라보거나, 직접 코드를 작성해보세요.

출력("안녕하세요, 코카클랭!")

변수 x = 42
출력("x = " + 문자열(x))
`;

const editorParent = document.getElementById("editor");

const editor = new EditorView({
    doc: defaultCode,
    extensions: [
        basicSetup,
        darkTheme,
        keymap.of([
            {
                key: "Ctrl-Enter",
                run: () => { runCode(); return true; },
            },
            {
                key: "Cmd-Enter",
                run: () => { runCode(); return true; },
            },
        ]),
    ],
    parent: editorParent,
});

// --- Run code ---
function runCode() {
    const output = document.getElementById("output");
    const execTime = document.getElementById("exec-time");

    if (!wasmModule) {
        output.textContent = "WASM 모듈이 로딩되지 않았습니다.";
        output.className = "has-error";
        return;
    }

    const source = editor.state.doc.toString();
    if (!source.trim()) {
        output.textContent = "";
        output.className = "";
        execTime.textContent = "";
        return;
    }

    const start = performance.now();
    let result;
    try {
        result = wasmModule.run_code(source);
    } catch (e) {
        result = `[시스템 오류] ${e.message}`;
    }
    const elapsed = performance.now() - start;

    output.textContent = result || "(출력 없음)";
    output.className = result.includes("[오류]") ? "has-error" : "";
    execTime.textContent = `${elapsed.toFixed(1)}ms`;
}

// --- Event listeners ---
document.getElementById("run-btn").addEventListener("click", runCode);

document.getElementById("clear-btn").addEventListener("click", () => {
    document.getElementById("output").textContent = "";
    document.getElementById("output").className = "";
    document.getElementById("exec-time").textContent = "";
});

document.getElementById("examples").addEventListener("change", (e) => {
    const key = e.target.value;
    if (key && EXAMPLES[key]) {
        editor.dispatch({
            changes: {
                from: 0,
                to: editor.state.doc.length,
                insert: EXAMPLES[key],
            },
        });
        e.target.value = "";
    }
});

// --- Load WASM ---
loadWasm();
