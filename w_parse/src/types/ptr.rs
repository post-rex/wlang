use crate::{parse_type, tag, ItemTy, ParResult, TokenSpan};
use w_tokenize::Span;

#[derive(Debug, Clone)]
pub struct TyPtr {
    pub span_ptr: Span,
    pub ty: Box<ItemTy>,
}

pub fn parse_ty_ptr(i: TokenSpan) -> ParResult<TyPtr> {
    let (i, span_ptr) = tag!(Kind::Mul)(i)?;
    let (i, ty) = parse_type(i)?;
    Ok((
        i,
        TyPtr {
            span_ptr,
            ty: Box::new(ty),
        },
    ))
}
