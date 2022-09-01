use crate::data::err::{DefinitionKind, MultipleDefinitionsError, UnresolvedTypeError};
use crate::data::types::{
    TypeArray, TypeEnum, TypeFunc, TypeInfo, TypeKind, TypeNever, TypePtr, TypeRef, TypeStruct,
    TypeTuple,
};
use crate::data::Origin;
use crate::{ErrorCollector, Module, PathBuf};
use std::cell::RefCell;
use w_parse::expr::path::{parse_path, ExprPath};
use w_parse::item::import::{Imports, ItemImports};
use w_parse::item::named::NamedKind;
use w_parse::item::Item;
use w_parse::types::array::TyArray;
use w_parse::types::func::TyFunc;
use w_parse::types::never::TyNever;
use w_parse::types::ptr::TyPtr;
use w_parse::types::r#enum::TyEnum;
use w_parse::types::r#struct::TyStruct;
use w_parse::types::tuple::TyTuple;
use w_parse::types::ItemTy;
use w_parse::util::NameTyPair;
use w_parse::ParsedModule;

pub fn run_pass1<'a, 'gc>(
    module: &ParsedModule<'a>,
    tsys: &'gc Module<'a, 'gc>,
    errs: &ErrorCollector<'a>,
) {
    // Imports
    for item in module.items.iter() {
        let def = match item {
            Item::Import(def) => def,
            Item::Definer(_) => continue,
        };

        let (root, base) = conv_path(tsys, &def.from);
        let root = root.access_or_create_module(&base);

        resolve_imports(&def.imports, root, tsys, errs);
    }

    // Type definitions
    for item in module.items.iter() {
        let def = match item {
            Item::Definer(def) => def,
            Item::Import(_) => continue,
        };

        let ty = match &def.kind {
            NamedKind::Type(ty) => ty,
            NamedKind::Func(_) => continue,
        };

        let tref = match tsys.access_or_create_type(&PathBuf::from([def.name.clone()])) {
            Origin::Local(tref) => tref,
            Origin::Import(imp) => {
                errs.add_error(MultipleDefinitionsError {
                    loc: def.name.clone(),
                    first: imp.loc.as_ref().unwrap().name.clone(),
                    kind: DefinitionKind::Import,
                });
                continue;
            }
        };

        if tref.definition.borrow().is_some() {
            errs.add_error(MultipleDefinitionsError {
                loc: def.name.clone(),
                first: tref
                    .loc
                    .clone()
                    .expect("Type ref should have location if present in type system")
                    .name
                    .clone(),
                kind: DefinitionKind::Type,
            });
            continue;
        }

        let kind = resolve_type(&ty.ty, tsys, errs);

        *tref.definition.borrow_mut() = Some(TypeInfo {
            kind: TypeKind::Named(kind),
        });
    }

    tsys.types
        .borrow()
        .iter()
        .filter(|(_, v)| v.definition.borrow().is_none())
        .for_each(|(_, v)| {
            errs.add_error(UnresolvedTypeError(v.loc.as_ref().unwrap().name.clone()))
        })
}

fn resolve_imports<'a, 'gc>(
    imps: &[Imports<'a>],
    root: &'gc Module<'a, 'gc>,
    tsys: &'gc Module<'a, 'gc>,
    errs: &ErrorCollector<'a>,
) {
    for imp in imps {
        match imp {
            Imports::Single(single) => {
                // import paths can not be absolute
                let (_, path) = conv_path(tsys, single);

                let name = path.last().expect("imported paths may not be empty");
                let path = path.slice(0..path.len() - 1);

                let md = root.access_or_create_module(path);
                tsys.imports.borrow_mut().insert(name.clone(), md);
            }
            Imports::Multiple(offset, imps) => {
                let (_, path) = conv_path(tsys, offset);
                let rel_root = root.access_or_create_module(&path);
                resolve_imports(imps, rel_root, tsys, errs);
            }
        }
    }
}

fn resolve_type<'a, 'gc>(
    ty: &ItemTy<'a>,
    tsys: &'gc Module<'a, 'gc>,
    errs: &ErrorCollector<'a>,
) -> &'gc TypeRef<'a, 'gc> {
    match ty {
        ItemTy::Referred(name) => {
            let (md, path) = conv_path(tsys, name);
            md.access_or_create_type(&path).unwrap()
        }
        // FIXME: check for known anonymous types, idiot :)
        _ => {
            let ty = build_type(ty, tsys, errs);
            let tref = &*tsys.types_arena.alloc(TypeRef {
                loc: None,
                definition: RefCell::new(Some(TypeInfo { kind: ty })),
            });
            tref
        }
    }
}

fn build_type<'a, 'gc>(
    ty: &ItemTy<'a>,
    tsys: &'gc Module<'a, 'gc>,
    errs: &ErrorCollector<'a>,
) -> TypeKind<'a, 'gc> {
    match ty {
        ItemTy::Referred(reference) => {
            let (root, path) = conv_path(tsys, reference);
            TypeKind::Referred(root.access_or_create_type(&path).unwrap())
        }
        ItemTy::Struct(TyStruct {
            span_struct,
            fields,
        }) => TypeKind::Struct(TypeStruct {
            def: *span_struct,
            fields: fields
                .iter()
                .map(|NameTyPair { name, ty }| (name.clone(), resolve_type(ty, tsys, errs)))
                .collect(),
        }),
        ItemTy::Enum(TyEnum {
            span_enum,
            variants,
        }) => TypeKind::Enum(TypeEnum {
            def: *span_enum,
            variants: variants
                .iter()
                .map(|(name, ty)| {
                    (
                        name.clone(),
                        ty.as_ref().map(|tp| conv_tuple(tp, tsys, errs)),
                    )
                })
                .collect(),
        }),
        ItemTy::Tuple(tp) => TypeKind::Tuple(conv_tuple(tp, tsys, errs)),
        ItemTy::Func(TyFunc {
            span_func,
            args,
            ret_ty,
        }) => TypeKind::Func(TypeFunc {
            def: *span_func,
            args: args.iter().map(|ty| resolve_type(ty, tsys, errs)).collect(),
            ret: resolve_type(ret_ty, tsys, errs),
        }),
        ItemTy::Array(TyArray { span, ty, size }) => TypeKind::Array(TypeArray {
            def: *span,
            ty: resolve_type(ty, tsys, errs),
            len: size.clone(),
        }),
        ItemTy::Pointer(TyPtr { span_ptr, ty }) => TypeKind::Ptr(TypePtr {
            def: *span_ptr,
            ty: resolve_type(ty, tsys, errs),
        }),
        ItemTy::Never(TyNever(span)) => TypeKind::Never(TypeNever(*span)),
    }
}

fn conv_tuple<'a, 'gc>(
    TyTuple { span, types }: &TyTuple<'a>,
    tsys: &'gc Module<'a, 'gc>,
    errs: &ErrorCollector<'a>,
) -> TypeTuple<'a, 'gc> {
    TypeTuple {
        def: *span,
        fields: types
            .iter()
            .map(|ty| resolve_type(ty, tsys, errs))
            .collect(),
    }
}

fn conv_path<'a, 'gc>(
    tsys: &'gc Module<'a, 'gc>,
    path: &ExprPath<'a>,
) -> (&'gc Module<'a, 'gc>, PathBuf<'a>) {
    let md = if path.root.is_some() {
        tsys.root()
    } else {
        tsys
    };
    (md, PathBuf::from(path.path.as_slice()))
}
