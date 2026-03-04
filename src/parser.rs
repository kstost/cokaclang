use crate::ast::*;
use crate::token::{Token, TokenType};
use crate::value::Value;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    pub arena: AstArena,
    expr_depth: usize,
    max_expr_depth: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
            arena: AstArena::new(),
            expr_depth: 0,
            max_expr_depth: read_depth_limit("COKAC_MAX_PARSE_EXPR_DEPTH", 4096),
        }
    }

    pub fn parse(mut self) -> Result<(AstArena, Vec<StmtId>), String> {
        let mut stmts = Vec::new();
        while !self.is_at_end() {
            let stmt = self.parse_declaration()?;
            stmts.push(stmt);
        }
        Ok((self.arena, stmts))
    }

    // ----- helpers -----

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn peek_type(&self) -> TokenType {
        self.tokens[self.current].token_type
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        self.peek_type() == TokenType::Eof
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }

    fn check(&self, tt: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek_type() == tt
        }
    }

    fn match_token(&mut self, tt: TokenType) -> bool {
        if self.check(tt) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, tt: TokenType, msg: &str) -> Result<Token, String> {
        if self.check(tt) {
            Ok(self.advance().clone())
        } else {
            Err(format!(
                "[줄 {}] 오류: {}",
                self.peek().line, msg
            ))
        }
    }

    fn line(&self) -> i32 {
        self.peek().line
    }

    // ----- parsing -----

    fn parse_declaration(&mut self) -> Result<StmtId, String> {
        let tt = self.peek_type();
        match tt {
            TokenType::Let | TokenType::Const => self.parse_var_declaration(),
            TokenType::Function => self.parse_function_declaration(false),
            TokenType::Async => {
                self.advance();
                self.consume(TokenType::Function, "'함수' 키워드가 필요합니다.")?;
                self.parse_function_declaration(true)
            }
            _ => self.parse_statement(),
        }
    }

    fn parse_var_declaration(&mut self) -> Result<StmtId, String> {
        let is_const = self.peek_type() == TokenType::Const;
        let line = self.line();
        self.advance(); // consume 변수/상수
        let name_tok = self.consume(TokenType::Identifier, "변수 이름이 필요합니다.")?;
        let name = name_tok.lexeme.clone();
        self.consume(TokenType::Equal, "'=' 기호가 필요합니다.")?;
        let init = self.parse_expression()?;
        self.consume(TokenType::Semicolon, "';' 기호가 필요합니다.")?;
        Ok(self.arena.add_stmt(StmtKind::Let { name, initializer: init, is_const }, line))
    }

    fn parse_function_declaration(&mut self, is_async: bool) -> Result<StmtId, String> {
        let line = self.line();
        // 'function' already consumed (or we consume it)
        if !is_async {
            self.advance(); // consume 함수
        }
        let name_tok = self.consume(TokenType::Identifier, "함수 이름이 필요합니다.")?;
        let name = name_tok.lexeme.clone();
        self.consume(TokenType::LParen, "'(' 기호가 필요합니다.")?;
        let mut params = Vec::new();
        if !self.check(TokenType::RParen) {
            loop {
                let param = self.consume(TokenType::Identifier, "매개변수 이름이 필요합니다.")?;
                params.push(param.lexeme.clone());
                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RParen, "')' 기호가 필요합니다.")?;
        let body = self.parse_block()?;
        Ok(self.arena.add_stmt(StmtKind::Function { name, params, body, is_async }, line))
    }

    fn parse_statement(&mut self) -> Result<StmtId, String> {
        let tt = self.peek_type();
        match tt {
            TokenType::Print => self.parse_print_statement(),
            TokenType::If => self.parse_if_statement(),
            TokenType::While => self.parse_while_statement(),
            TokenType::For => self.parse_for_statement(),
            TokenType::Return => self.parse_return_statement(),
            TokenType::Break => self.parse_break_statement(),
            TokenType::Continue => self.parse_continue_statement(),
            TokenType::LBrace => self.parse_block_statement(),
            TokenType::Import => self.parse_import_statement(),
            TokenType::Try => self.parse_try_statement(),
            TokenType::Throw => self.parse_throw_statement(),
            _ => self.parse_assignment_or_expr(),
        }
    }

    fn parse_print_statement(&mut self) -> Result<StmtId, String> {
        let line = self.line();
        self.advance(); // consume 출력
        let expr = self.parse_expression()?;
        self.consume(TokenType::Semicolon, "';' 기호가 필요합니다.")?;
        Ok(self.arena.add_stmt(StmtKind::Print(expr), line))
    }

    fn parse_if_statement(&mut self) -> Result<StmtId, String> {
        let line = self.line();
        self.advance(); // consume 만약
        self.consume(TokenType::LParen, "'(' 기호가 필요합니다.")?;
        let condition = self.parse_expression()?;
        self.consume(TokenType::RParen, "')' 기호가 필요합니다.")?;
        let then_branch = self.parse_block()?;
        let else_branch = if self.match_token(TokenType::Else) {
            if self.check(TokenType::If) {
                Some(self.parse_if_statement()?)
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };
        Ok(self.arena.add_stmt(StmtKind::If { condition, then_branch, else_branch }, line))
    }

    fn parse_while_statement(&mut self) -> Result<StmtId, String> {
        let line = self.line();
        self.advance(); // consume 반복
        self.consume(TokenType::LParen, "'(' 기호가 필요합니다.")?;
        let condition = self.parse_expression()?;
        self.consume(TokenType::RParen, "')' 기호가 필요합니다.")?;
        let body = self.parse_block()?;
        Ok(self.arena.add_stmt(StmtKind::While { condition, body }, line))
    }

    fn parse_for_statement(&mut self) -> Result<StmtId, String> {
        let line = self.line();
        self.advance(); // consume 동안
        self.consume(TokenType::LParen, "'(' 기호가 필요합니다.")?;

        // Initializer
        let initializer = if self.match_token(TokenType::Semicolon) {
            None
        } else if self.check(TokenType::Let) || self.check(TokenType::Const) {
            Some(self.parse_var_declaration()?)
        } else {
            let s = self.parse_assignment_or_expr()?;
            Some(s)
        };

        // Condition
        let condition = if self.check(TokenType::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.consume(TokenType::Semicolon, "';' 기호가 필요합니다.")?;

        // Increment (can be assignment like "i = i + 1" or expression)
        let increment = if self.check(TokenType::RParen) {
            None
        } else {
            Some(self.parse_for_increment()?)
        };
        self.consume(TokenType::RParen, "')' 기호가 필요합니다.")?;

        let body = self.parse_block()?;
        Ok(self.arena.add_stmt(StmtKind::For { initializer, condition, increment, body }, line))
    }

    /// Parse for-loop increment: `ident = expr` or just `expr` (no semicolon)
    fn parse_for_increment(&mut self) -> Result<StmtId, String> {
        let line = self.line();
        if self.peek_type() == TokenType::Identifier {
            let saved = self.current;
            let name = self.advance().lexeme.clone();
            if self.match_token(TokenType::Equal) {
                let value = self.parse_expression()?;
                return Ok(self.arena.add_stmt(StmtKind::Assign { name, value }, line));
            }
            self.current = saved;
        }
        let expr = self.parse_expression()?;
        Ok(self.arena.add_stmt(StmtKind::Expr(expr), line))
    }

    fn parse_return_statement(&mut self) -> Result<StmtId, String> {
        let line = self.line();
        self.advance(); // consume 반환
        let value = if self.check(TokenType::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.consume(TokenType::Semicolon, "';' 기호가 필요합니다.")?;
        Ok(self.arena.add_stmt(StmtKind::Return { value }, line))
    }

    fn parse_break_statement(&mut self) -> Result<StmtId, String> {
        let line = self.line();
        self.advance();
        self.consume(TokenType::Semicolon, "';' 기호가 필요합니다.")?;
        Ok(self.arena.add_stmt(StmtKind::Break, line))
    }

    fn parse_continue_statement(&mut self) -> Result<StmtId, String> {
        let line = self.line();
        self.advance();
        self.consume(TokenType::Semicolon, "';' 기호가 필요합니다.")?;
        Ok(self.arena.add_stmt(StmtKind::Continue, line))
    }

    fn parse_block_statement(&mut self) -> Result<StmtId, String> {
        self.parse_block()
    }

    fn parse_block(&mut self) -> Result<StmtId, String> {
        let line = self.line();
        self.consume(TokenType::LBrace, "'{' 기호가 필요합니다.")?;
        let mut stmts = Vec::new();
        while !self.check(TokenType::RBrace) && !self.is_at_end() {
            let s = self.parse_declaration()?;
            stmts.push(s);
        }
        self.consume(TokenType::RBrace, "'}' 기호가 필요합니다.")?;
        Ok(self.arena.add_stmt(StmtKind::Block(stmts), line))
    }

    fn parse_import_statement(&mut self) -> Result<StmtId, String> {
        let line = self.line();
        self.advance(); // consume 가져오기
        let path_tok = self.consume(TokenType::StringLit, "파일 경로가 필요합니다.")?;
        let path = path_tok.lexeme.clone();
        let alias = if self.match_token(TokenType::Alias) {
            let alias_tok = self.consume(TokenType::Identifier, "별칭 이름이 필요합니다.")?;
            Some(alias_tok.lexeme.clone())
        } else {
            None
        };
        self.consume(TokenType::Semicolon, "';' 기호가 필요합니다.")?;
        Ok(self.arena.add_stmt(StmtKind::Import { path, alias }, line))
    }

    fn parse_try_statement(&mut self) -> Result<StmtId, String> {
        let line = self.line();
        self.advance(); // consume 시도
        let try_block = self.parse_block()?;

        let mut catch_block = None;
        let mut error_name = None;
        let mut error_info_name = None;
        let mut finally_block = None;

        if self.match_token(TokenType::Catch) {
            self.consume(TokenType::LParen, "'(' 기호가 필요합니다.")?;
            let err_tok = self.consume(TokenType::Identifier, "오류 변수 이름이 필요합니다.")?;
            error_name = Some(err_tok.lexeme.clone());
            if self.match_token(TokenType::Comma) {
                let info_tok = self.consume(TokenType::Identifier, "오류 정보 변수 이름이 필요합니다.")?;
                error_info_name = Some(info_tok.lexeme.clone());
            }
            self.consume(TokenType::RParen, "')' 기호가 필요합니다.")?;
            catch_block = Some(self.parse_block()?);
        }

        if self.match_token(TokenType::Finally) {
            finally_block = Some(self.parse_block()?);
        }

        if catch_block.is_none() && finally_block.is_none() {
            return Err(format!("[줄 {}] 오류: '시도' 뒤에 '잡기' 또는 '마침' 블록이 필요합니다.", line));
        }

        Ok(self.arena.add_stmt(
            StmtKind::Try {
                try_block,
                catch_block,
                error_name,
                error_info_name,
                finally_block,
            },
            line,
        ))
    }

    fn parse_throw_statement(&mut self) -> Result<StmtId, String> {
        let line = self.line();
        self.advance(); // consume 던지기
        let expr = self.parse_expression()?;
        self.consume(TokenType::Semicolon, "';' 기호가 필요합니다.")?;
        Ok(self.arena.add_stmt(StmtKind::Throw(expr), line))
    }

    fn parse_assignment_or_expr(&mut self) -> Result<StmtId, String> {
        let line = self.line();

        // Check for simple variable assignment: identifier = expr ;
        if self.peek_type() == TokenType::Identifier {
            let saved = self.current;
            let name = self.advance().lexeme.clone();

            if self.match_token(TokenType::Equal) {
                let value = self.parse_expression()?;
                self.consume(TokenType::Semicolon, "';' 기호가 필요합니다.")?;
                return Ok(self.arena.add_stmt(StmtKind::Assign { name, value }, line));
            }

            // Not a simple assignment, backtrack
            self.current = saved;
        }

        // Parse as expression, then check for compound assignment targets
        let expr = self.parse_expression()?;

        // Check if this is property/index assignment
        if self.match_token(TokenType::Equal) {
            let value = self.parse_expression()?;
            self.consume(TokenType::Semicolon, "';' 기호가 필요합니다.")?;
            let expr_kind = &self.arena.get_expr(expr).kind;
            match expr_kind {
                ExprKind::Index { target, index } => {
                    let target = *target;
                    let index = *index;
                    return Ok(self.arena.add_stmt(StmtKind::IndexAssign { target, index, value }, line));
                }
                ExprKind::Property { target, name } => {
                    let target = *target;
                    let name = name.clone();
                    return Ok(self.arena.add_stmt(StmtKind::PropertyAssign { target, name, value }, line));
                }
                _ => {
                    return Err(format!("[줄 {}] 오류: 잘못된 대입 대상입니다.", line));
                }
            }
        }

        self.consume(TokenType::Semicolon, "';' 기호가 필요합니다.")?;
        Ok(self.arena.add_stmt(StmtKind::Expr(expr), line))
    }

    // ----- expression parsing (precedence climbing) -----

    fn parse_expression(&mut self) -> Result<ExprId, String> {
        self.expr_depth += 1;
        if self.expr_depth > self.max_expr_depth {
            let line = self.line();
            self.expr_depth -= 1;
            return Err(format!(
                "[줄 {}] 오류: 파싱 깊이 제한({})을 초과했습니다. 괄호/중첩 표현식을 줄이거나 COKAC_MAX_PARSE_EXPR_DEPTH 값을 조정하세요.",
                line,
                self.max_expr_depth
            ));
        }
        let result = self.parse_or();
        self.expr_depth -= 1;
        result
    }

    fn parse_or(&mut self) -> Result<ExprId, String> {
        let mut left = self.parse_and()?;
        while self.check(TokenType::OrOr) {
            let line = self.line();
            self.advance();
            let right = self.parse_and()?;
            left = self.arena.add_expr(
                ExprKind::Binary { op: TokenType::OrOr, left, right },
                line,
            );
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<ExprId, String> {
        let mut left = self.parse_equality()?;
        while self.check(TokenType::AndAnd) {
            let line = self.line();
            self.advance();
            let right = self.parse_equality()?;
            left = self.arena.add_expr(
                ExprKind::Binary { op: TokenType::AndAnd, left, right },
                line,
            );
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<ExprId, String> {
        let mut left = self.parse_comparison()?;
        while self.check(TokenType::EqualEqual) || self.check(TokenType::BangEqual) {
            let line = self.line();
            let op = self.advance().token_type;
            let right = self.parse_comparison()?;
            left = self.arena.add_expr(
                ExprKind::Binary { op, left, right },
                line,
            );
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<ExprId, String> {
        let mut left = self.parse_term()?;
        while self.check(TokenType::Greater)
            || self.check(TokenType::GreaterEqual)
            || self.check(TokenType::Less)
            || self.check(TokenType::LessEqual)
        {
            let line = self.line();
            let op = self.advance().token_type;
            let right = self.parse_term()?;
            left = self.arena.add_expr(
                ExprKind::Binary { op, left, right },
                line,
            );
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<ExprId, String> {
        let mut left = self.parse_factor()?;
        while self.check(TokenType::Plus) || self.check(TokenType::Minus) {
            let line = self.line();
            let op = self.advance().token_type;
            let right = self.parse_factor()?;
            left = self.arena.add_expr(
                ExprKind::Binary { op, left, right },
                line,
            );
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<ExprId, String> {
        let mut left = self.parse_unary()?;
        while self.check(TokenType::Star)
            || self.check(TokenType::Slash)
            || self.check(TokenType::Percent)
        {
            let line = self.line();
            let op = self.advance().token_type;
            let right = self.parse_unary()?;
            left = self.arena.add_expr(
                ExprKind::Binary { op, left, right },
                line,
            );
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<ExprId, String> {
        if self.check(TokenType::Bang) || self.check(TokenType::Minus) {
            let line = self.line();
            let op = self.advance().token_type;
            let right = self.parse_unary()?;
            return Ok(self.arena.add_expr(ExprKind::Unary { op, right }, line));
        }
        if self.check(TokenType::Await) {
            let line = self.line();
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(self.arena.add_expr(ExprKind::Await(expr), line));
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<ExprId, String> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.check(TokenType::LParen) {
                let line = self.line();
                self.advance();
                let mut args = Vec::new();
                if !self.check(TokenType::RParen) {
                    loop {
                        args.push(self.parse_expression()?);
                        if !self.match_token(TokenType::Comma) {
                            break;
                        }
                    }
                }
                self.consume(TokenType::RParen, "')' 기호가 필요합니다.")?;
                expr = self.arena.add_expr(ExprKind::Call { callee: expr, args }, line);
            } else if self.check(TokenType::LBracket) {
                let line = self.line();
                self.advance();
                let index = self.parse_expression()?;
                self.consume(TokenType::RBracket, "']' 기호가 필요합니다.")?;
                expr = self.arena.add_expr(ExprKind::Index { target: expr, index }, line);
            } else if self.check(TokenType::Dot) {
                let line = self.line();
                self.advance();
                let name_tok = self.consume(TokenType::Identifier, "속성 이름이 필요합니다.")?;
                expr = self.arena.add_expr(
                    ExprKind::Property { target: expr, name: name_tok.lexeme.clone() },
                    line,
                );
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<ExprId, String> {
        let line = self.line();
        let tt = self.peek_type();

        match tt {
            TokenType::Number => {
                let tok = self.advance().clone();
                let val = Value::Number(tok.number_value);
                Ok(self.arena.add_expr(ExprKind::Literal(val), line))
            }
            TokenType::StringLit => {
                let tok = self.advance().clone();
                let val = Value::String(tok.lexeme.clone());
                Ok(self.arena.add_expr(ExprKind::Literal(val), line))
            }
            TokenType::True => {
                self.advance();
                Ok(self.arena.add_expr(ExprKind::Literal(Value::Bool(true)), line))
            }
            TokenType::False => {
                self.advance();
                Ok(self.arena.add_expr(ExprKind::Literal(Value::Bool(false)), line))
            }
            TokenType::Nil => {
                self.advance();
                Ok(self.arena.add_expr(ExprKind::Literal(Value::Nil), line))
            }
            TokenType::Identifier => {
                let tok = self.advance().clone();
                Ok(self.arena.add_expr(ExprKind::Variable(tok.lexeme.clone()), line))
            }
            TokenType::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.consume(TokenType::RParen, "')' 기호가 필요합니다.")?;
                Ok(self.arena.add_expr(ExprKind::Grouping(expr), line))
            }
            TokenType::LBracket => {
                self.advance();
                let mut items = Vec::new();
                if !self.check(TokenType::RBracket) {
                    loop {
                        items.push(self.parse_expression()?);
                        if !self.match_token(TokenType::Comma) {
                            break;
                        }
                        // allow trailing comma
                        if self.check(TokenType::RBracket) {
                            break;
                        }
                    }
                }
                self.consume(TokenType::RBracket, "']' 기호가 필요합니다.")?;
                Ok(self.arena.add_expr(ExprKind::Array(items), line))
            }
            TokenType::LBrace => {
                // Object literal — but only if we're in expression context (not a block)
                self.advance();
                let mut keys = Vec::new();
                let mut values = Vec::new();
                if !self.check(TokenType::RBrace) {
                    loop {
                        // Key can be identifier or string
                        let key = if self.check(TokenType::Identifier) {
                            self.advance().lexeme.clone()
                        } else if self.check(TokenType::StringLit) {
                            self.advance().lexeme.clone()
                        } else {
                            return Err(format!(
                                "[줄 {}] 오류: 객체 키가 필요합니다.",
                                self.line()
                            ));
                        };
                        self.consume(TokenType::Colon, "':' 기호가 필요합니다.")?;
                        let val = self.parse_expression()?;
                        keys.push(key);
                        values.push(val);
                        if !self.match_token(TokenType::Comma) {
                            break;
                        }
                        if self.check(TokenType::RBrace) {
                            break;
                        }
                    }
                }
                self.consume(TokenType::RBrace, "'}' 기호가 필요합니다.")?;
                Ok(self.arena.add_expr(ExprKind::Object { keys, values }, line))
            }
            TokenType::Function => {
                // Anonymous function expression: 함수(params) { body }
                self.advance();
                self.consume(TokenType::LParen, "'(' 기호가 필요합니다.")?;
                let mut params = Vec::new();
                if !self.check(TokenType::RParen) {
                    loop {
                        let param = self.consume(TokenType::Identifier, "매개변수 이름이 필요합니다.")?;
                        params.push(param.lexeme.clone());
                        if !self.match_token(TokenType::Comma) {
                            break;
                        }
                    }
                }
                self.consume(TokenType::RParen, "')' 기호가 필요합니다.")?;
                let body = self.parse_block()?;
                // Create function as value literal
                let func_val = Value::make_function(String::new(), params, body, false);
                Ok(self.arena.add_expr(ExprKind::Literal(func_val), line))
            }
            _ => Err(format!(
                "[줄 {}] 오류: 예상치 못한 토큰: '{}'",
                self.line(),
                self.peek().lexeme
            )),
        }
    }
}

fn read_depth_limit(key: &str, default_value: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|v| *v >= 64)
        .unwrap_or(default_value)
}
