#![allow(dead_code)]

use rustdoc_types::{
    Constant, DynTrait, Enum, FnDecl, Function, FunctionPointer, GenericArg, GenericArgs,
    GenericBound, GenericParamDef, GenericParamDefKind, Generics, Impl, Import, Item, ItemEnum,
    OpaqueTy, Path, PolyTrait, Static, Struct, StructKind, Term, Trait, TraitAlias, Type,
    TypeBinding, TypeBindingKind, Typedef, Union, WherePredicate,
};

#[allow(unused_variables)]
pub trait Visitor {
    #[inline]
    fn visit_path(&mut self, path: &Path) {}

    #[inline]
    fn visit_import(&mut self, import: &Import) {}
}

pub fn visit_item(item: &Item, v: &mut impl Visitor) {
    match &item.inner {
        ItemEnum::Function(fun) => visit_function(fun, v),
        ItemEnum::Struct(struct_) => visit_struct(struct_, v),
        ItemEnum::StructField(field_type) => visit_type(field_type, v),
        ItemEnum::AssocType {
            generics,
            bounds,
            default,
        } => {
            visit_generics(generics, v);
            for bound in bounds {
                visit_generic_bound(bound, v);
            }
            if let Some(default) = default {
                visit_type(default, v);
            }
        }
        ItemEnum::AssocConst { type_, default: _ } => {
            visit_type(type_, v);
        }
        ItemEnum::Impl(impl_) => visit_impl(impl_, v),
        ItemEnum::Typedef(type_def) => visit_type_def(type_def, v),
        ItemEnum::Union(union_) => visit_union(union_, v),
        ItemEnum::Enum(enum_) => visit_enum(enum_, v),

        ItemEnum::Trait(trait_) => visit_trait(trait_, v),
        ItemEnum::TraitAlias(trait_alias) => visit_trait_alias(trait_alias, v),
        ItemEnum::OpaqueTy(opaque_type) => visit_opaque_type(opaque_type, v),
        ItemEnum::Constant(constant) => visit_constant(constant, v),
        ItemEnum::Static(static_) => visit_static(static_, v),
        ItemEnum::Import(import) => {
            v.visit_import(import);
        }

        // ignore these because they don't contain anything of interest
        ItemEnum::Module(_) => {}
        ItemEnum::Variant(_) => {}
        ItemEnum::ExternCrate { .. } => {}
        ItemEnum::ForeignType => {}
        ItemEnum::Primitive(_) => {}
        ItemEnum::ProcMacro(_) => {}
        ItemEnum::Macro(_) => {}
    }
}

fn visit_static(static_: &Static, v: &mut impl Visitor) {
    let Static {
        type_,
        mutable: _,
        expr: _,
    } = static_;
    visit_type(type_, v);
}

fn visit_opaque_type(opaque_type: &OpaqueTy, v: &mut impl Visitor) {
    let OpaqueTy { bounds, generics } = opaque_type;
    for bound in bounds {
        visit_generic_bound(bound, v);
    }
    visit_generics(generics, v);
}

fn visit_trait_alias(trait_alias: &TraitAlias, v: &mut impl Visitor) {
    let TraitAlias { generics, params } = trait_alias;
    visit_generics(generics, v);
    for param in params {
        visit_generic_bound(param, v);
    }
}

fn visit_trait(trait_: &Trait, v: &mut impl Visitor) {
    let Trait {
        is_auto: _,
        is_unsafe: _,
        items: _,
        generics,
        bounds,
        implementations: _,
    } = trait_;
    visit_generics(generics, v);
    for bound in bounds {
        visit_generic_bound(bound, v);
    }
}

fn visit_enum(enum_: &Enum, v: &mut impl Visitor) {
    let Enum {
        generics,
        variants_stripped: _,
        variants: _,
        impls: _,
    } = enum_;
    visit_generics(generics, v);
}

fn visit_union(union: &Union, v: &mut impl Visitor) {
    let Union {
        generics,
        fields_stripped: _,
        fields: _,
        impls: _,
    } = union;
    visit_generics(generics, v);
}

fn visit_impl(impl_: &Impl, v: &mut impl Visitor) {
    let Impl {
        is_unsafe: _,
        generics,
        provided_trait_methods: _,
        trait_,
        for_,
        items: _,
        negative: _,
        synthetic: _,
        blanket_impl,
    } = impl_;
    // blanket impls in other crates that happen to match one of our types shouldn't count
    if blanket_impl.is_some() {
        return;
    }
    visit_generics(generics, v);
    if let Some(trait_) = trait_ {
        visit_path(trait_, v);
    }
    visit_type(for_, v);
}

fn visit_type_def(type_def: &Typedef, v: &mut impl Visitor) {
    let Typedef { type_, generics } = type_def;
    visit_type(type_, v);
    visit_generics(generics, v);
}

fn visit_struct(struct_: &Struct, v: &mut impl Visitor) {
    let Struct {
        kind,
        generics,
        impls: _,
    } = struct_;
    visit_struct_kind(kind, v);
    visit_generics(generics, v);
}

fn visit_struct_kind(kind: &StructKind, _v: &mut impl Visitor) {
    match kind {
        StructKind::Unit => {}
        StructKind::Tuple(_) => {}
        StructKind::Plain {
            fields: _,
            fields_stripped: _,
        } => {}
    }
}

fn visit_function(fun: &Function, v: &mut impl Visitor) {
    let Function {
        decl,
        generics,
        header: _,
        has_body: _,
    } = fun;
    visit_fn_decl(decl, v);
    visit_generics(generics, v);
}

fn visit_fn_decl(decl: &FnDecl, v: &mut impl Visitor) {
    let FnDecl {
        inputs,
        output,
        c_variadic: _,
    } = decl;
    for (_, ty) in inputs {
        visit_type(ty, v);
    }
    if let Some(output) = output {
        visit_type(output, v);
    }
}

fn visit_generics(generics: &Generics, v: &mut impl Visitor) {
    let Generics {
        params,
        where_predicates,
    } = generics;
    for param in params {
        visit_generic_param_def(param, v);
    }
    for where_predicate in where_predicates {
        visit_where_predicate(where_predicate, v);
    }
}

fn visit_generic_param_def(param: &GenericParamDef, v: &mut impl Visitor) {
    let GenericParamDef { name: _, kind } = param;
    visit_generic_param_def_kind(kind, v);
}

fn visit_where_predicate(where_predicate: &WherePredicate, v: &mut impl Visitor) {
    match where_predicate {
        WherePredicate::BoundPredicate {
            type_,
            bounds,
            generic_params,
        } => {
            visit_type(type_, v);
            for bound in bounds {
                visit_generic_bound(bound, v);
            }
            for generic_param in generic_params {
                visit_generic_param_def(generic_param, v);
            }
        }
        WherePredicate::RegionPredicate {
            lifetime: _,
            bounds,
        } => {
            for bound in bounds {
                visit_generic_bound(bound, v);
            }
        }
        WherePredicate::EqPredicate { lhs, rhs } => {
            visit_type(lhs, v);
            visit_term(rhs, v);
        }
    }
}

fn visit_generic_param_def_kind(kind: &GenericParamDefKind, v: &mut impl Visitor) {
    match kind {
        GenericParamDefKind::Lifetime { outlives: _ } => {}
        GenericParamDefKind::Type {
            bounds,
            default,
            synthetic: _,
        } => {
            for bound in bounds {
                visit_generic_bound(bound, v);
            }
            if let Some(default) = default {
                visit_type(default, v);
            }
        }
        GenericParamDefKind::Const { type_, default: _ } => {
            visit_type(type_, v);
        }
    }
}

fn visit_generic_bound(bound: &GenericBound, v: &mut impl Visitor) {
    match bound {
        GenericBound::TraitBound {
            trait_,
            generic_params,
            modifier: _,
        } => {
            visit_path(trait_, v);
            for param in generic_params {
                visit_generic_param_def(param, v);
            }
        }
        GenericBound::Outlives(_) => {}
    }
}

fn visit_term(term: &Term, v: &mut impl Visitor) {
    match term {
        Term::Type(type_) => visit_type(type_, v),
        Term::Constant(constant) => visit_constant(constant, v),
    }
}

fn visit_path(path: &Path, v: &mut impl Visitor) {
    v.visit_path(path);
    let Path {
        name: _,
        id: _,
        args,
    } = path;
    if let Some(args) = args {
        visit_generic_args(args, v);
    }
}

fn visit_generic_args(args: &GenericArgs, v: &mut impl Visitor) {
    match args {
        GenericArgs::AngleBracketed { args, bindings } => {
            for arg in args {
                visit_generic_arg(arg, v);
            }
            for binding in bindings {
                visit_type_binding(binding, v);
            }
        }
        GenericArgs::Parenthesized { inputs, output } => {
            for type_ in inputs {
                visit_type(type_, v);
            }
            if let Some(type_) = output {
                visit_type(type_, v);
            }
        }
    }
}

fn visit_type_binding(binding: &TypeBinding, v: &mut impl Visitor) {
    let TypeBinding {
        name: _,
        args,
        binding,
    } = binding;
    visit_generic_args(args, v);
    visit_type_binding_kind(binding, v);
}

fn visit_type_binding_kind(binding: &TypeBindingKind, v: &mut impl Visitor) {
    match binding {
        TypeBindingKind::Equality(term) => visit_term(term, v),
        TypeBindingKind::Constraint(bounds) => {
            for bound in bounds {
                visit_generic_bound(bound, v)
            }
        }
    }
}

fn visit_generic_arg(arg: &GenericArg, v: &mut impl Visitor) {
    match arg {
        GenericArg::Lifetime(_) => {}
        GenericArg::Type(type_) => visit_type(type_, v),
        GenericArg::Const(constant) => visit_constant(constant, v),
        GenericArg::Infer => {}
    }
}

fn visit_constant(constant: &Constant, v: &mut impl Visitor) {
    let Constant {
        type_,
        expr: _,
        value: _,
        is_literal: _,
    } = constant;
    visit_type(type_, v);
}

fn visit_type(type_: &Type, v: &mut impl Visitor) {
    match type_ {
        Type::ResolvedPath(path) => visit_path(path, v),
        Type::DynTrait(dyn_trait) => visit_dyn_trait(dyn_trait, v),
        Type::Generic(_) => {}
        Type::Primitive(_) => {}
        Type::FunctionPointer(fn_pointer) => visit_function_pointer(fn_pointer, v),
        Type::Tuple(types) => {
            for type_ in types {
                visit_type(type_, v);
            }
        }
        Type::Slice(type_) => visit_type(type_, v),
        Type::Array { type_, len: _ } => visit_type(type_, v),
        Type::ImplTrait(bounds) => {
            for bound in bounds {
                visit_generic_bound(bound, v);
            }
        }
        Type::Infer => {}
        Type::RawPointer { mutable: _, type_ } => visit_type(type_, v),
        Type::BorrowedRef {
            lifetime: _,
            mutable: _,
            type_,
        } => visit_type(type_, v),
        Type::QualifiedPath {
            name: _,
            args,
            self_type,
            trait_,
        } => {
            visit_generic_args(args, v);
            visit_type(self_type, v);
            visit_path(trait_, v);
        }
    }
}

fn visit_function_pointer(fn_pointer: &FunctionPointer, v: &mut impl Visitor) {
    let FunctionPointer {
        decl,
        generic_params,
        header: _,
    } = fn_pointer;
    visit_fn_decl(decl, v);
    for generic_param in generic_params {
        visit_generic_param_def(generic_param, v);
    }
}

fn visit_dyn_trait(dyn_trait: &DynTrait, v: &mut impl Visitor) {
    let DynTrait {
        traits,
        lifetime: _,
    } = dyn_trait;
    for trait_ in traits {
        visit_poly_trait(trait_, v);
    }
}

fn visit_poly_trait(trait_: &PolyTrait, v: &mut impl Visitor) {
    let PolyTrait {
        trait_,
        generic_params,
    } = trait_;
    visit_path(trait_, v);
    for generic_param in generic_params {
        visit_generic_param_def(generic_param, v);
    }
}
