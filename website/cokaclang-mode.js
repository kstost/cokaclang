import { StreamLanguage } from "@codemirror/language";
import { tags } from "@lezer/highlight";

const keywords = new Set([
    "변수", "상수", "만약", "아니면", "반복", "동안",
    "중단", "계속", "반환", "시도", "잡기", "마침",
    "던지기", "가져오기", "별칭", "비동기", "대기",
    "함수", "형식", "물려받기", "속성", "만들기", "행동",
]);

const constants = new Set(["참", "거짓", "없음"]);

const builtins = new Set([
    "출력", "단언", "타입", "길이", "문자열", "숫자", "정수", "불린",
    "자료파싱", "자료문자열화", "자료읽기", "자료쓰기",
    "파일읽기", "파일쓰기", "파일존재",
    "서버열기", "요청받기", "응답보내기", "연결닫기",
    "명령실행", "명령실행결과", "현재시간", "시간문자열",
    "내보내기", "모듈가져오기",
    "객체설정", "객체삭제", "객체키들",
    "배열추가", "배열삭제", "배열정렬", "배열맵",
    "문자분할", "문자포함", "문자다듬기",
    "클래스생성", "인스턴스생성", "메서드호출",
]);

const operatorKeywords = new Set(["그리고", "또는"]);

// Match a Unicode identifier: starts with letter or _, followed by letters/digits/_
const identRe = /[_\p{L}][\p{L}\p{N}_]*/u;

const cokaclangDef = {
    startState() {
        return { inBlockComment: false };
    },

    token(stream, state) {
        // Block comment continuation
        if (state.inBlockComment) {
            if (stream.match(/.*?\*\//)) {
                state.inBlockComment = false;
            } else {
                stream.skipToEnd();
            }
            return "blockComment";
        }

        // Skip whitespace
        if (stream.eatSpace()) return null;

        // Block comment start
        if (stream.match("/*")) {
            if (!stream.match(/.*?\*\//)) {
                state.inBlockComment = true;
                stream.skipToEnd();
            }
            return "blockComment";
        }

        // Line comments: // or #
        if (stream.match("//") || stream.match("#")) {
            stream.skipToEnd();
            return "lineComment";
        }

        // Strings
        if (stream.match('"')) {
            while (!stream.eol()) {
                const ch = stream.next();
                if (ch === "\\") {
                    stream.next(); // skip escaped char
                } else if (ch === '"') {
                    return "string";
                }
            }
            return "string"; // unterminated string
        }

        // Numbers
        if (stream.match(/^\d+(?:\.\d+)?/)) {
            return "number";
        }

        // Multi-char operators (must check before single-char)
        if (stream.match("==") || stream.match("!=") || stream.match("<=") || stream.match(">=")) {
            return "compareOperator";
        }

        // Single-char operators and punctuation
        const ch = stream.peek();
        if ("+-*/%".includes(ch)) {
            stream.next();
            return "arithmeticOperator";
        }
        if (ch === "!" || ch === "<" || ch === ">") {
            stream.next();
            return "compareOperator";
        }
        if (ch === "=") {
            stream.next();
            return "operator";
        }
        if ("(){}[];,.".includes(ch)) {
            stream.next();
            return "punctuation";
        }

        // Identifiers / keywords
        if (stream.match(identRe)) {
            const word = stream.current();
            if (keywords.has(word)) return "keyword";
            if (constants.has(word)) return "bool";
            if (builtins.has(word)) return "standard(variableName)";
            if (operatorKeywords.has(word)) return "operatorKeyword";
            return "variableName";
        }

        // Fallback: advance one character
        stream.next();
        return null;
    },

    languageData: {
        commentTokens: { line: "//", block: { open: "/*", close: "*/" } },
    },

    tokenTable: {
        keyword: tags.keyword,
        bool: tags.bool,
        number: tags.number,
        string: tags.string,
        lineComment: tags.lineComment,
        blockComment: tags.blockComment,
        variableName: tags.variableName,
        "standard(variableName)": tags.standard(tags.variableName),
        operatorKeyword: tags.operatorKeyword,
        compareOperator: tags.compareOperator,
        arithmeticOperator: tags.arithmeticOperator,
        operator: tags.operator,
        punctuation: tags.punctuation,
    },
};

export const cokaclang = StreamLanguage.define(cokaclangDef);
