use crate::expr::many::{parse_object, ExprObject};
use crate::expr::path::{parse_path, Path};
use crate::{ParResult, TokenSpan};

#[derive(Debug, Clone)]
pub struct ExprCtor<'a> {
    pub ty_path: Path<'a>,
    pub vals: ExprObject<'a>,
}

pub fn parse_ctor(i: TokenSpan) -> ParResult<ExprCtor> {
    let (i, ty_path) = parse_path(i)?;
    let (i, vals) = parse_object(i)?;

    Ok((i, ExprCtor { ty_path, vals }))
}