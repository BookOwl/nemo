#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Binary(Box<Expr>, Op, Box<Expr>),
    Number(f64),
    Name(String),
    Call(Box<Expr>, Vec<Box<Expr>>),
    Lambda(Vec<String>, Box<Expr>),
    Pull,
    FinishedPipe,
    Block(Vec<Box<Expr>>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    While(Box<Expr>, Box<Expr>),
    Assignment(String, Box<Expr>),
    Push(Box<Expr>),
    Bool(bool),
    Return(Box<Expr>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Op {
    Plus,
    Minus,
    Times,
    Slash,
    Pipe,
    Percent,
    Greater,
    Lesser,
    Equals,
    And,
    Or,
}


#[derive(Debug, PartialEq, Clone)]
pub struct Prototype {
    pub name: String,
    pub args: Vec<String>,
}
impl Prototype {
    pub fn new(name: String, args: Vec<String>) -> Prototype {
        Prototype {
            name: name,
            args: args,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Definition {
    pub prototype: Prototype,
    pub body: Box<Expr>,
}
impl Definition {
    pub fn new(prototype: Prototype, body: Box<Expr>) -> Definition {
        Definition {
            prototype: prototype,
            body: body,
        }
    }
}
