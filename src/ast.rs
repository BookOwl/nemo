#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Binary(Box<Expr>, Op, Box<Expr>),
    Number(i32),
    Name(String),
    Call(Box<Expr>, Vec<Box<Expr>>),
    Lambda(Vec<String>, Box<Expr>),
    Pull,
    Block(Vec<Box<Statement>>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Op {
    Plus,
    Minus,
    Times,
    Slash,
    Pipe,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Assignment(String, Box<Expr>),
    Push(Box<Expr>)
}
