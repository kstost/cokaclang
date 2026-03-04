import { basicSetup } from "codemirror";
import { EditorView, keymap } from "@codemirror/view";
import { oneDark } from "@codemirror/theme-one-dark";
import { cokaclang } from "./cokaclang-mode.js";

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
    hello: `출력("안녕하세요, cokaclang!");
출력("Hello, Cokaclang!");`,

    variables: `변수 이름 = "세계";
변수 나이 = 2024;
변수 실수 = 3.14;

출력("안녕, " + 이름 + "!");
출력("원주율: " + 문자열(실수));
출력("합계: " + 문자열(나이 + 1));`,

    loop: `// 반복문 (동안 = for 루프)
변수 합 = 0;
동안 (변수 i = 1; i <= 10; i = i + 1) {
    합 = 합 + i;
}
출력("1부터 10까지의 합: " + 문자열(합));

// 조건문
만약 (합 > 50) {
    출력("합이 50보다 큽니다");
} 아니면 {
    출력("합이 50 이하입니다");
}`,

    function: `함수 인사(이름) {
    반환 "안녕하세요, " + 이름 + "님!";
}

함수 팩토리얼(n) {
    만약 (n <= 1) {
        반환 1;
    }
    반환 n * 팩토리얼(n - 1);
}

출력(인사("cokaclang"));
출력("5! = " + 문자열(팩토리얼(5)));
출력("10! = " + 문자열(팩토리얼(10)));`,

    array: `변수 과일 = ["사과", "바나나", "딸기", "포도"];

출력("과일 목록:");
동안 (변수 i = 0; i < 길이(과일); i = i + 1) {
    출력("  " + 문자열(i + 1) + ". " + 과일[i]);
}

배열추가(과일, "수박");
출력("\\n추가 후: " + 문자열(길이(과일)) + "개");

변수 결과 = 배열맵(과일, 함수(x) { 반환 x + "!"; });
출력("맵 결과: " + 문자열(결과));`,

    object: `변수 사람 = {
    "이름": "홍길동",
    "나이": 30,
    "취미": ["독서", "코딩"]
};

출력("이름: " + 사람.이름);
출력("나이: " + 문자열(사람.나이));
출력("취미: " + 문자열(사람.취미));

객체설정(사람, "직업", "개발자");
출력("직업: " + 사람.직업);
출력("키 목록: " + 문자열(객체키들(사람)));`,

    fibonacci: `함수 피보나치(n) {
    만약 (n <= 0) { 반환 0; }
    만약 (n == 1) { 반환 1; }

    변수 a = 0;
    변수 b = 1;
    변수 temp = 0;
    동안 (변수 i = 2; i <= n; i = i + 1) {
        temp = b;
        b = a + b;
        a = temp;
    }
    반환 b;
}

출력("피보나치 수열 (처음 15개):");
변수 결과 = "";
동안 (변수 i = 0; i < 15; i = i + 1) {
    만약 (i > 0) { 결과 = 결과 + ", "; }
    결과 = 결과 + 문자열(피보나치(i));
}
출력(결과);`,

    class: `변수 동물 = 클래스생성("동물", {
    "초기화": 함수(자신, 이름, 소리) {
        자신.이름 = 이름;
        자신.소리 = 소리;
    },
    "울기": 함수(자신) {
        출력(자신.이름 + ": " + 자신.소리 + "!");
    },
    "소개": 함수(자신) {
        출력("저는 " + 자신.이름 + "입니다.");
    }
});

변수 고양이 = 인스턴스생성(동물, ["고양이", "야옹"]);
변수 강아지 = 인스턴스생성(동물, ["강아지", "멍멍"]);

메서드호출(고양이, "소개");
메서드호출(고양이, "울기");
메서드호출(강아지, "소개");
메서드호출(강아지, "울기");`,

    string: `// 문자열 처리
변수 텍스트 = "  Hello, cokaclang!  ";
출력("원본: '" + 텍스트 + "'");
출력("다듬기: '" + 문자다듬기(텍스트) + "'");
출력("대문자: " + 문자대문자("hello world"));
출력("소문자: " + 문자소문자("HELLO WORLD"));

변수 문장 = "사과,바나나,딸기,포도";
변수 과일들 = 문자분할(문장, ",");
출력("분할: " + 문자열(과일들));
출력("합치기: " + 배열문자열합치기(과일들, " | "));

출력("포함: " + 문자열(문자포함(문장, "바나나")));
출력("치환: " + 문자치환(문장, "딸기", "수박"));
출력("반복: " + 문자반복("코카 ", 3));`,

    higher: `// 고차 함수: 맵, 필터, 리듀스
변수 숫자들 = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

// 맵: 각 요소를 제곱
변수 제곱들 = 배열맵(숫자들, 함수(x) { 반환 x * x; });
출력("제곱: " + 문자열(제곱들));

// 필터: 짝수만
변수 짝수들 = 배열필터(숫자들, 함수(x) { 반환 x % 2 == 0; });
출력("짝수: " + 문자열(짝수들));

// 리듀스: 합계
변수 합계 = 배열리듀스(숫자들, 함수(누적, x) { 반환 누적 + x; }, 0);
출력("합계: " + 문자열(합계));

// 조합: 짝수의 제곱의 합
변수 결과 = 배열리듀스(
    배열맵(
        배열필터(숫자들, 함수(x) { 반환 x % 2 == 0; }),
        함수(x) { 반환 x * x; }
    ),
    함수(누적, x) { 반환 누적 + x; },
    0
);
출력("짝수 제곱의 합: " + 문자열(결과));`,

    error: `// 시도-잡기 구문
시도 {
    변수 결과 = 10 / 0;
    출력("10 / 0 = " + 문자열(결과));
} 잡기 (오류) {
    출력("에러 발생: " + 오류);
}

// 사용자 정의 에러
함수 나누기(a, b) {
    만약 (b == 0) {
        던지기 "0으로 나눌 수 없습니다!";
    }
    반환 a / b;
}

시도 {
    출력("10 / 2 = " + 문자열(나누기(10, 2)));
    출력("10 / 0 = " + 문자열(나누기(10, 0)));
} 잡기 (오류) {
    출력("잡힌 에러: " + 오류);
} 마침 {
    출력("마침 블록은 항상 실행됩니다.");
}`,

    closure: `// 클로저: 카운터 만들기
함수 카운터만들기(시작값) {
    변수 상태 = {"값": 시작값};
    반환 {
        "증가": 함수() { 상태.값 = 상태.값 + 1; 반환 상태.값; },
        "감소": 함수() { 상태.값 = 상태.값 - 1; 반환 상태.값; },
        "현재값": 함수() { 반환 상태.값; }
    };
}

변수 카운터 = 카운터만들기(0);
출력("현재: " + 문자열(카운터.현재값()));
출력("증가: " + 문자열(카운터.증가()));
출력("증가: " + 문자열(카운터.증가()));
출력("증가: " + 문자열(카운터.증가()));
출력("감소: " + 문자열(카운터.감소()));
출력("현재: " + 문자열(카운터.현재값()));`,

    sort: `// 버블 정렬 구현
함수 버블정렬(배열) {
    변수 n = 길이(배열);
    변수 i = 0;
    변수 j = 0;
    변수 임시 = 0;
    반복 (i < n - 1) {
        j = 0;
        반복 (j < n - i - 1) {
            만약 (배열[j] > 배열[j + 1]) {
                임시 = 배열[j];
                배열[j] = 배열[j + 1];
                배열[j + 1] = 임시;
            }
            j = j + 1;
        }
        i = i + 1;
    }
    반환 배열;
}

변수 데이터 = [64, 34, 25, 12, 22, 11, 90];
출력("정렬 전: " + 문자열(데이터));
버블정렬(데이터);
출력("정렬 후: " + 문자열(데이터));

// 내장 정렬
변수 이름들 = ["다", "가", "마", "나", "라"];
배열정렬(이름들);
출력("문자 정렬: " + 문자열(이름들));`,

    math: `// 수학 함수
출력("절댓값(-42) = " + 문자열(절댓값(-42)));
출력("최대(3, 7) = " + 문자열(최대(3, 7)));
출력("최소(3, 7) = " + 문자열(최소(3, 7)));
출력("정수(3.7) = " + 문자열(정수(3.7)));

// 타입 확인
출력("타입(42) = " + 타입(42));
변수 인사 = "안녕";
출력("타입(인사) = " + 타입(인사));
출력("타입(참) = " + 타입(참));
출력("타입([1,2]) = " + 타입([1, 2]));
변수 빈객체 = {};
출력("타입(객체) = " + 타입(빈객체));
출력("타입(없음) = " + 타입(없음));

// 타입 변환
출력("숫자(\\"123\\") = " + 문자열(숫자("123")));
출력("불린(0) = " + 문자열(불린(0)));
출력("불린(1) = " + 문자열(불린(1)));`,

    json: `// JSON 파싱과 문자열화
변수 제이슨 = "{\\"이름\\":\\"홍길동\\",\\"나이\\":25,\\"취미\\":[\\"코딩\\",\\"독서\\"]}";
변수 데이터 = 자료파싱(제이슨);
출력("이름: " + 데이터.이름);
출력("나이: " + 문자열(데이터.나이));
출력("취미: " + 문자열(데이터.취미));

// 객체를 JSON 문자열로
변수 설정 = {"언어": "cokaclang", "버전": 1, "재미있는가": 참};
출력("JSON: " + 자료문자열화(설정));
출력("예쁘게:\\n" + 자료예쁘게문자열화(설정));`,

    inherit: `// 상속: 도형 계층 구조
변수 도형 = 클래스생성("도형", {
    "초기화": 함수(자신, 이름) {
        자신.이름 = 이름;
    },
    "소개": 함수(자신) {
        반환 "도형: " + 자신.이름;
    }
});

변수 원 = 클래스생성("원", {
    "초기화": 함수(자신, 반지름) {
        자신.이름 = "원";
        자신.반지름 = 반지름;
    },
    "넓이": 함수(자신) {
        반환 3.14159 * 자신.반지름 * 자신.반지름;
    }
}, 도형);

변수 사각형 = 클래스생성("사각형", {
    "초기화": 함수(자신, 가로, 세로) {
        자신.이름 = "사각형";
        자신.가로 = 가로;
        자신.세로 = 세로;
    },
    "넓이": 함수(자신) {
        반환 자신.가로 * 자신.세로;
    }
}, 도형);

변수 원1 = 인스턴스생성(원, [5]);
변수 사각형1 = 인스턴스생성(사각형, [4, 6]);

출력(메서드호출(원1, "소개"));
출력("원 넓이: " + 문자열(메서드호출(원1, "넓이")));
출력(메서드호출(사각형1, "소개"));
출력("사각형 넓이: " + 문자열(메서드호출(사각형1, "넓이")));
출력("원은 도형? " + 문자열(상속확인(원1, 도형)));`,
};

// --- Editor setup ---
const defaultCode = `// cokaclang 플레이그라운드에 오신 것을 환영합니다!
// 위의 "예제 선택"에서 예제를 골라보거나, 직접 코드를 작성해보세요.

출력("안녕하세요, cokaclang!");

변수 x = 42;
출력("x = " + 문자열(x));
`;

const editorParent = document.getElementById("editor");

const editor = new EditorView({
    doc: defaultCode,
    extensions: [
        basicSetup,
        cokaclang,
        oneDark,
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
        EditorView.theme({
            "&": { height: "100%" },
            ".cm-scroller": { overflow: "auto" },
        }),
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
