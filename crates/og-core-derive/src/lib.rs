//! `#[derive(Reflect)]` for `og-core`.
//!
//! The derive turns a plain struct into a reflected layout: its schema (name,
//! version, docs, and per field the type, docs, label, unit, range, default and
//! `tunable` flag) and the field-by-field byte access the ECS uses for column
//! storage. The generated code names `::og_core`, so a crate using the derive
//! depends on `og-core` (which re-exports the derive as `og_core::Reflect`).
//!
//! ```ignore
//! /// Hit points of anything that can be destroyed.
//! #[derive(Clone, Copy, Debug, Reflect)]
//! #[reflect(name = "Health", version = 1)]
//! struct Health {
//!     /// Current hit points.
//!     #[reflect(min = 0.0, max = 1000.0, label = "Hit points", unit = "hp", tunable, default = 100.0)]
//!     hp: f32,
//! }
//! ```
//!
//! Supported shapes: structs with named fields, `struct Tag {}` and unit
//! structs (tags with no fields). Generic structs, tuple structs, enums and
//! unions are refused at compile time with a reason.
//!
//! Field types are the `og_core::reflect::ReflectField` set (`f32` ... `Mat4`,
//! `bool`, `Bool8`, `Str<N>`). Two field attributes shape the
//! schema beyond metadata: `name = "camelCase"` sets the schema field name
//! (the Rust ident otherwise), and `variants = ["a", "b"]` on a `u32` field
//! makes it an `enum` field whose value is the variant index.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Expr, ExprLit, Fields, Lit, LitStr, UnOp, parse_macro_input,
};

#[proc_macro_derive(Reflect, attributes(reflect))]
pub fn derive_reflect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand(&input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

struct FieldMeta {
    ident: syn::Ident,
    ty: syn::Type,
    doc: String,
    label: Option<String>,
    unit: Option<String>,
    min: Option<f64>,
    max: Option<f64>,
    tunable: bool,
    default: Option<Vec<f64>>,
    rename: Option<String>,
    variants: Option<Vec<String>>,
}

fn expand(input: &DeriveInput) -> syn::Result<TokenStream2> {
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &input.generics,
            "Reflect: generic types are not supported; a schema names one concrete layout",
        ));
    }
    let ident = &input.ident;
    let (mut name, mut version) = (ident.to_string(), 1u32);
    for attr in reflect_attrs(&input.attrs) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                name = meta.value()?.parse::<LitStr>()?.value();
                Ok(())
            } else if meta.path.is_ident("version") {
                version = meta.value()?.parse::<syn::LitInt>()?.base10_parse()?;
                Ok(())
            } else {
                Err(meta
                    .error("unknown reflect attribute on a struct; expected `name` or `version`"))
            }
        })?;
    }
    let doc = doc_of(&input.attrs);

    let data = match &input.data {
        Data::Struct(data) => data,
        _ => {
            return Err(syn::Error::new_spanned(
                ident,
                "Reflect: only structs are supported (enums and unions have no column layout)",
            ));
        }
    };
    let (fields, unit) = match &data.fields {
        Fields::Named(named) => (
            named
                .named
                .iter()
                .map(field_meta)
                .collect::<syn::Result<Vec<_>>>()?,
            false,
        ),
        Fields::Unit => (Vec::new(), true),
        Fields::Unnamed(_) => {
            return Err(syn::Error::new_spanned(
                ident,
                "Reflect: tuple structs are not supported; name the fields so the schema can",
            ));
        }
    };

    let schema_fields = fields.iter().map(|f| {
        let fname = f.rename.clone().unwrap_or_else(|| f.ident.to_string());
        let ty = &f.ty;
        let doc = &f.doc;
        let label = opt_string(&f.label);
        let unit = opt_string(&f.unit);
        let min = opt_f64(f.min);
        let max = opt_f64(f.max);
        let tunable = f.tunable;
        let default = match &f.default {
            Some(values) => {
                let values = values.iter().map(|v| f64_tokens(*v));
                quote! { ::std::option::Option::Some(::std::vec![#(#values),*]) }
            }
            None => quote! { ::std::option::Option::None },
        };
        let (field_type, variants) = match &f.variants {
            Some(names) => (
                quote! {{
                    // An enum field is stored as its variant index.
                    let _: ::core::marker::PhantomData<u32> = ::core::marker::PhantomData::<#ty>;
                    ::og_core::reflect::FieldType::Enum
                }},
                quote! { ::std::vec![#(::std::string::String::from(#names)),*] },
            ),
            None => (
                quote! { <#ty as ::og_core::reflect::ReflectField>::TYPE },
                quote! { ::std::vec::Vec::new() },
            ),
        };
        quote! {
            ::og_core::reflect::FieldSchema {
                name: ::std::string::String::from(#fname),
                ty: #field_type,
                doc: ::std::string::String::from(#doc),
                label: #label,
                unit: #unit,
                min: #min,
                max: #max,
                tunable: #tunable,
                default: #default,
                variants: #variants,
            }
        }
    });

    let writes = fields.iter().enumerate().map(|(i, f)| {
        let fident = &f.ident;
        quote! {
            ::og_core::reflect::ReflectField::write_packed(&self.#fident, &mut |bytes: &[u8]| out(#i, bytes));
        }
    });
    let temps: Vec<_> = (0..fields.len())
        .map(|i| format_ident!("__og_field_{}", i))
        .collect();
    let reads = fields.iter().enumerate().map(|(i, f)| {
        let ty = &f.ty;
        let tmp = &temps[i];
        quote! {
            let #tmp: #ty = <#ty as ::og_core::reflect::ReflectField>::read_packed(
                &mut |buf: &mut [u8]| input(#i, buf),
            );
        }
    });
    let idents = fields.iter().map(|f| &f.ident);
    let construct = if unit {
        quote! { Self }
    } else {
        quote! { Self { #(#idents: #temps),* } }
    };
    let input_name = if fields.is_empty() {
        quote! { _input }
    } else {
        quote! { input }
    };
    let out_name = if fields.is_empty() {
        quote! { _out }
    } else {
        quote! { out }
    };

    Ok(quote! {
        impl ::og_core::reflect::Reflect for #ident {
            const NAME: &'static str = #name;

            fn schema() -> ::og_core::reflect::Schema {
                ::og_core::reflect::Schema {
                    name: ::std::string::String::from(#name),
                    version: #version,
                    doc: ::std::string::String::from(#doc),
                    fields: ::std::vec![#(#schema_fields),*],
                }
            }

            fn write_fields(&self, #out_name: &mut dyn ::core::ops::FnMut(usize, &[u8])) {
                #(#writes)*
            }

            fn read_fields(#input_name: &mut dyn ::core::ops::FnMut(usize, &mut [u8])) -> Self {
                #(#reads)*
                #construct
            }
        }
    })
}

fn reflect_attrs(attrs: &[Attribute]) -> impl Iterator<Item = &Attribute> {
    attrs.iter().filter(|a| a.path().is_ident("reflect"))
}

fn doc_of(attrs: &[Attribute]) -> String {
    let mut lines = Vec::new();
    for attr in attrs.iter().filter(|a| a.path().is_ident("doc")) {
        if let syn::Meta::NameValue(nv) = &attr.meta
            && let Expr::Lit(ExprLit {
                lit: Lit::Str(s), ..
            }) = &nv.value
        {
            let line = s.value();
            lines.push(line.strip_prefix(' ').unwrap_or(&line).to_string());
        }
    }
    lines.join("\n").trim().to_string()
}

fn field_meta(field: &syn::Field) -> syn::Result<FieldMeta> {
    let ident = field.ident.clone().expect("named field");
    let mut meta = FieldMeta {
        ident,
        ty: field.ty.clone(),
        doc: doc_of(&field.attrs),
        label: None,
        unit: None,
        min: None,
        max: None,
        tunable: false,
        default: None,
        rename: None,
        variants: None,
    };
    for attr in reflect_attrs(&field.attrs) {
        attr.parse_nested_meta(|m| {
            if m.path.is_ident("tunable") {
                meta.tunable = true;
            } else if m.path.is_ident("label") {
                meta.label = Some(m.value()?.parse::<LitStr>()?.value());
            } else if m.path.is_ident("unit") {
                meta.unit = Some(m.value()?.parse::<LitStr>()?.value());
            } else if m.path.is_ident("min") {
                meta.min = Some(number(&m.value()?.parse::<Expr>()?)?);
            } else if m.path.is_ident("max") {
                meta.max = Some(number(&m.value()?.parse::<Expr>()?)?);
            } else if m.path.is_ident("name") {
                meta.rename = Some(m.value()?.parse::<LitStr>()?.value());
            } else if m.path.is_ident("variants") {
                let expr = m.value()?.parse::<Expr>()?;
                let Expr::Array(array) = &expr else {
                    return Err(syn::Error::new_spanned(
                        &expr,
                        "Reflect: variants takes an array of string literals, as [\"none\", \"left\"]",
                    ));
                };
                let mut names = Vec::new();
                for e in &array.elems {
                    match e {
                        Expr::Lit(ExprLit {
                            lit: Lit::Str(s), ..
                        }) => names.push(s.value()),
                        other => {
                            return Err(syn::Error::new_spanned(
                                other,
                                "Reflect: a variant name is a string literal",
                            ))
                        }
                    }
                }
                if names.is_empty() {
                    return Err(syn::Error::new_spanned(
                        &expr,
                        "Reflect: an enum field needs at least one variant",
                    ));
                }
                meta.variants = Some(names);
            } else if m.path.is_ident("default") {
                let expr = m.value()?.parse::<Expr>()?;
                meta.default = Some(match &expr {
                    Expr::Array(array) => {
                        array.elems.iter().map(number).collect::<syn::Result<Vec<_>>>()?
                    }
                    other => vec![number(other)?],
                });
            } else {
                return Err(m.error(
                    "unknown reflect attribute on a field; expected min, max, label, unit, tunable, default, name or variants",
                ));
            }
            Ok(())
        })?;
    }
    if let (Some(min), Some(max)) = (meta.min, meta.max)
        && min > max
    {
        return Err(syn::Error::new_spanned(
            &field.ty,
            format!(
                "Reflect: field `{}` has min {min} above max {max}",
                meta.ident
            ),
        ));
    }
    Ok(meta)
}

/// A finite numeric literal, optionally negated: `1`, `0.5`, `-2.0`.
fn number(expr: &Expr) -> syn::Result<f64> {
    let value = literal(expr)?;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(syn::Error::new_spanned(
            expr,
            "Reflect: the number is not finite",
        ))
    }
}

fn literal(expr: &Expr) -> syn::Result<f64> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Float(f), ..
        }) => f.base10_parse::<f64>(),
        Expr::Lit(ExprLit {
            lit: Lit::Int(i), ..
        }) => i.base10_parse::<f64>(),
        Expr::Unary(unary) if matches!(unary.op, UnOp::Neg(_)) => Ok(-number(&unary.expr)?),
        Expr::Group(group) => number(&group.expr),
        Expr::Paren(paren) => number(&paren.expr),
        other => Err(syn::Error::new_spanned(
            other,
            "Reflect: expected a number literal",
        )),
    }
}

fn f64_tokens(v: f64) -> TokenStream2 {
    let magnitude = v.abs();
    if v.is_sign_negative() {
        quote! { -#magnitude }
    } else {
        quote! { #magnitude }
    }
}

fn opt_string(value: &Option<String>) -> TokenStream2 {
    match value {
        Some(s) => quote! { ::std::option::Option::Some(::std::string::String::from(#s)) },
        None => quote! { ::std::option::Option::None },
    }
}

fn opt_f64(value: Option<f64>) -> TokenStream2 {
    match value {
        Some(v) => {
            let v = f64_tokens(v);
            quote! { ::std::option::Option::Some(#v) }
        }
        None => quote! { ::std::option::Option::None },
    }
}
