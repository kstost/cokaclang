use crate::token::TokenType;
use crate::value::Value;

pub type ExprId = usize;
pub type StmtId = usize;

#[derive(Debug, Clone)]
pub enum ExprKind {
    Literal(Value),
    Variable(String),
    Binary {
        op: TokenType,
        left: ExprId,
        right: ExprId,
    },
    Unary {
        op: TokenType,
        right: ExprId,
    },
    Grouping(ExprId),
    Call {
        callee: ExprId,
        args: Vec<ExprId>,
    },
    Await(ExprId),
    Array(Vec<ExprId>),
    Object {
        keys: Vec<String>,
        values: Vec<ExprId>,
    },
    Index {
        target: ExprId,
        index: ExprId,
    },
    Property {
        target: ExprId,
        name: String,
    },
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub line: i32,
}

#[derive(Debug, Clone)]
pub enum StmtKind {
    Let {
        name: String,
        initializer: ExprId,
        is_const: bool,
    },
    Assign {
        name: String,
        value: ExprId,
    },
    IndexAssign {
        target: ExprId,
        index: ExprId,
        value: ExprId,
    },
    PropertyAssign {
        target: ExprId,
        name: String,
        value: ExprId,
    },
    Print(ExprId),
    If {
        condition: ExprId,
        then_branch: StmtId,
        else_branch: Option<StmtId>,
    },
    While {
        condition: ExprId,
        body: StmtId,
    },
    For {
        initializer: Option<StmtId>,
        condition: Option<ExprId>,
        increment: Option<StmtId>,
        body: StmtId,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: StmtId,
        is_async: bool,
    },
    Return {
        value: Option<ExprId>,
    },
    Import {
        path: String,
        alias: Option<String>,
    },
    Try {
        try_block: StmtId,
        catch_block: Option<StmtId>,
        error_name: Option<String>,
        error_info_name: Option<String>,
        finally_block: Option<StmtId>,
    },
    Throw(ExprId),
    Block(Vec<StmtId>),
    Break,
    Continue,
    Expr(ExprId),
}

#[derive(Debug, Clone)]
pub struct Stmt {
    pub kind: StmtKind,
    pub line: i32,
}

#[derive(Debug)]
pub struct AstArena {
    pub exprs: Vec<Expr>,
    pub stmts: Vec<Stmt>,
}

impl AstArena {
    pub fn new() -> Self {
        AstArena {
            exprs: Vec::new(),
            stmts: Vec::new(),
        }
    }

    pub fn add_expr(&mut self, kind: ExprKind, line: i32) -> ExprId {
        let id = self.exprs.len();
        self.exprs.push(Expr { kind, line });
        id
    }

    pub fn add_stmt(&mut self, kind: StmtKind, line: i32) -> StmtId {
        let id = self.stmts.len();
        self.stmts.push(Stmt { kind, line });
        id
    }

    pub fn get_expr(&self, id: ExprId) -> &Expr {
        &self.exprs[id]
    }

    pub fn get_stmt(&self, id: StmtId) -> &Stmt {
        &self.stmts[id]
    }
}
