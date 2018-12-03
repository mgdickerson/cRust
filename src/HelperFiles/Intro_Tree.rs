/// Still unsure if this gets used. We shall see.

pub enum StatementKind {
    ACompoundStm,
    AAssignStm,
    APrintStm,
}

pub union StatementType {
    comp: Compound,
    assign: Assignment,
    prnt: Print,
}

pub struct AStm {
    kind: StatementKind,
    stmType: StatementType,
}

pub struct Compound {
    stm1: Box<AStm>,
    stm2: Box<AStm>,
}

pub struct Assignment {
    ID: String,
    exp: Box<AExp>,
}

pub struct Print {
    exps: AExpList,
}

//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~//

pub enum ExpressionKind {
    AIdExpr,
    ANumExpr,
    AOpExpr,
    AEseqExpr,
}

pub union ExpressionType {
    ID: String,
    num: i64,
    oper: Operation,
    eseq: Eseq,
}

pub struct Operation {
    left: Box<AExp>,
    oper: Box<ABinOp>,
    right: Box<AExp>,
}

pub struct Eseq {
    stm: Box<AStm>,
    exp: Box<AExp>,
}

pub struct AExp {
    kind: ExpressionKind,
    exprType: ExpressionType,
}