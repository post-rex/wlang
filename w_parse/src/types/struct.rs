use crate::expr::parse_many0;
use crate::util::{parse_name_ty_pair, NameTyPair};
use crate::{parse_keyword, tag, ParResult, TokenSpan};
use nom::combinator::all_consuming;
use w_tokenize::Span;

#[derive(Debug, Clone)]
pub struct TyStruct {
    pub span_struct: Span,
    pub fields: Vec<NameTyPair>,
}

pub fn parse_ty_struct(i: TokenSpan) -> ParResult<TyStruct> {
    let (i, span_struct) = parse_keyword("struct")(i)?;

    let (i, block) = tag!(Kind::Block(_), Token { kind: Kind::Block(vals), .. } => vals)(i)?;
    let block = TokenSpan::new(i.file.clone(), block);

    let (_, fields) = all_consuming(parse_many0(parse_name_ty_pair))(block)?;

    Ok((
        i,
        TyStruct {
            span_struct,
            fields,
        },
    ))
}
