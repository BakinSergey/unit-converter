// Abstract Syntax Tree

#[derive(Debug)]
pub enum Stmt {
    Conversation(Expr),  // applicable for Convert expr
    Decomposition(Expr), // applicable for Fraction and Unit expr
}

#[derive(Debug)]
pub enum Expr {
    Convert(f64, Box<Expr>, Box<Expr>),
    Fraction {
        up: Vec<Expr>,
        down: Vec<Expr>,
    },
    Unit {
        pfx: Option<String>,
        tag: String,
        pow: i8,
        den: bool,
    },
}
