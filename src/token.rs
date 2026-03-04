#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenType {
    // Literals
    Eof,
    Identifier,
    Number,
    StringLit,

    // Keywords
    Let,       // 변수
    Const,     // 상수
    Print,     // 출력
    If,        // 만약
    Else,      // 아니면
    While,     // 반복
    For,       // 동안
    Break,     // 중단
    Continue,  // 계속
    Function,  // 함수
    Return,    // 반환
    Async,     // 비동기
    Await,     // 대기
    Import,    // 가져오기
    Alias,     // 별칭
    Try,       // 시도
    Catch,     // 잡기
    Finally,   // 마침
    Throw,     // 던지기
    True,      // 참
    False,     // 거짓
    Nil,       // 없음

    // Operators
    Plus,         // +
    Minus,        // -
    Star,         // *
    Slash,        // /
    Percent,      // %
    Bang,         // !
    BangEqual,    // !=
    Equal,        // =
    EqualEqual,   // ==
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=
    AndAnd,       // && or 그리고
    OrOr,         // || or 또는

    // Delimiters
    LParen,    // (
    RParen,    // )
    LBrace,    // {
    RBrace,    // }
    LBracket,  // [
    RBracket,  // ]
    Comma,     // ,
    Dot,       // .
    Colon,     // :
    Semicolon, // ;

    Error,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: i32,
    pub number_value: f64,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: i32) -> Self {
        Token {
            token_type,
            lexeme,
            line,
            number_value: 0.0,
        }
    }

    pub fn number(lexeme: String, value: f64, line: i32) -> Self {
        Token {
            token_type: TokenType::Number,
            lexeme,
            line,
            number_value: value,
        }
    }

    pub fn eof(line: i32) -> Self {
        Token {
            token_type: TokenType::Eof,
            lexeme: String::new(),
            line,
            number_value: 0.0,
        }
    }
}

pub fn keyword_type(word: &str) -> Option<TokenType> {
    match word {
        "변수" => Some(TokenType::Let),
        "상수" => Some(TokenType::Const),
        "출력" => Some(TokenType::Print),
        "만약" => Some(TokenType::If),
        "아니면" => Some(TokenType::Else),
        "반복" => Some(TokenType::While),
        "동안" => Some(TokenType::For),
        "중단" => Some(TokenType::Break),
        "계속" => Some(TokenType::Continue),
        "함수" => Some(TokenType::Function),
        "반환" => Some(TokenType::Return),
        "비동기" => Some(TokenType::Async),
        "대기" => Some(TokenType::Await),
        "가져오기" => Some(TokenType::Import),
        "별칭" => Some(TokenType::Alias),
        "시도" => Some(TokenType::Try),
        "잡기" => Some(TokenType::Catch),
        "마침" => Some(TokenType::Finally),
        "던지기" => Some(TokenType::Throw),
        "참" => Some(TokenType::True),
        "거짓" => Some(TokenType::False),
        "없음" => Some(TokenType::Nil),
        "그리고" => Some(TokenType::AndAnd),
        "또는" => Some(TokenType::OrOr),
        _ => None,
    }
}
