use crate::data::err::fmt::ErrorFormatter;
use crate::data::err::{AnalyzerError, ErrKind};

use w_parse::Ident;

pub struct UnresolvedTypeError(pub Ident);

impl AnalyzerError for UnresolvedTypeError {
    fn kind(&self) -> ErrKind {
        ErrKind::Error
    }

    fn fmt(&self, f: &mut ErrorFormatter) {
        f.err()
            .description("Unable to resolve type")
            .location(self.0 .0.clone())
            .add_note("Try defining the type")
            .add_note("Try importing the type")
            .submit();
    }
}
