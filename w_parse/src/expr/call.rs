use crate::expr::many::{parse_tuple, ExprTuple};
use crate::expr::Expr;
use crate::{ParResult, TokenSpan};

#[derive(Debug, Clone)]
pub struct ExprCall {
    pub base: Box<Expr>,
    pub args: ExprTuple,
}

pub fn parse_call_wrapper(i: TokenSpan) -> ParResult<Box<dyn FnOnce(Expr) -> Expr>> {
    let (i, args) = parse_tuple(i)?;
    Ok((
        i,
        Box::new(move |expr| {
            Expr::Call(ExprCall {
                base: Box::new(expr),
                args,
            })
        }),
    ))
}
