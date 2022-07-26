use crate::expr::{parse_expression, Expr};
use crate::{parse_name, ErrorChain, Ident, ParResult, TokenSpan, Weak};
use assert_matches::assert_matches;
use nom::combinator::{all_consuming, map, opt};
use nom::multi::separated_list0;
use nom::sequence::{terminated, tuple};
use nom::Parser;
use std::rc::Rc;
use w_tokenize::{Kind, Span};

#[derive(Debug, Clone)]
pub struct ExprTuple {
    pub span: Span,
    pub values: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprArray {
    pub span: Span,
    pub values: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct ExprObject {
    pub span: Span,
    pub values: Vec<(Ident, Expr)>,
}

pub fn parse_tuple(i: TokenSpan) -> ParResult<ExprTuple> {
    let (i, tuple) = Weak(Kind::Tuple(Rc::from([]))).parse(i)?;
    let span = tuple.span;
    let tuple =
        assert_matches!(tuple.kind, Kind::Tuple(vals) => TokenSpan::new(i.file.clone(), vals));
    let (_, vals) = all_consuming(parse_many0(parse_expression))(tuple)?;

    Ok((i, ExprTuple { span, values: vals }))
}

pub fn parse_array(i: TokenSpan) -> ParResult<ExprArray> {
    let (i, array) = Weak(Kind::Array(Rc::from([]))).parse(i)?;
    let span = array.span;
    let array =
        assert_matches!(array.kind, Kind::Array(vals) => TokenSpan::new(i.file.clone(), vals));
    let (_, vals) = all_consuming(parse_many0(parse_expression))(array)?;

    Ok((i, ExprArray { span, values: vals }))
}

pub fn parse_object(i: TokenSpan) -> ParResult<ExprObject> {
    let (i, block) = Weak(Kind::Block(Rc::from([]))).parse(i)?;
    let span = block.span;
    let block =
        assert_matches!(block.kind, Kind::Block(vals) => TokenSpan::new(i.file.clone(), vals));
    let (_, vals) = all_consuming(parse_many0(map(
        tuple((parse_name, Weak(Kind::Assign), parse_expression)),
        |(k, _, v)| (k, v),
    )))(block)?;

    Ok((i, ExprObject { span, values: vals }))
}

pub fn parse_many0<F, T>(parser: F) -> impl FnMut(TokenSpan) -> ParResult<Vec<T>>
where
    F: Parser<TokenSpan, T, ErrorChain>,
{
    terminated(
        separated_list0(Weak(Kind::Comma), parser),
        opt(Weak(Kind::Comma)),
    )
}
