use syn::{Ident, Meta, Field, TypePath, Path, punctuated::Punctuated, GenericArgument, Token, PathSegment, PathArguments::AngleBracketed, AngleBracketedGenericArguments};
use proc_macro2::{TokenStream as TokenStream2};
use quote::{format_ident, quote};
use darling::FromMeta;

use super::utils::{iterate_path_arguments, check_type_order};
use super::relation_attr::RelationAttr;

pub trait FieldExtras {
    fn is_primary_key(&self) -> bool;
    fn is_indexed(&self) -> bool;
    fn is_nullable(&self) -> bool;
    fn parse_relation(&self) -> Option<RelationAttr>;
    fn get_db_type(&self) -> TokenStream2;
}

fn is_chrono_option(segment: &PathSegment) -> bool {
    let expected: Vec<String> = vec!["Option".to_owned(), "DateTime".to_owned(), "Utc".to_owned()];
    let res = iterate_path_arguments(segment, &expected, 0);

    res
}

fn search_attr_in_field(field: &Field, attr: &str) -> bool {
    for option in (&field.attrs).into_iter() {
        let option = option.parse_meta().unwrap();
        match option {
            Meta::Path(path) if path.get_ident().unwrap().to_string() == attr => {
                return true;
            },
            _ => {},
        }
    }
    return false;
}

impl FieldExtras for Field {
    fn is_primary_key(&self) -> bool {
        search_attr_in_field(self, "primary_key")
    }

    fn is_indexed(&self) -> bool {
        search_attr_in_field(self, "indexed")
    }

    fn parse_relation(&self) -> Option<RelationAttr> {
        for attr in (&self.attrs).into_iter() {
            let option = attr.parse_meta().unwrap();
            if let Ok(relation) = RelationAttr::from_meta(&option) {
                return Some(relation);
            }
        }
        None
    }

    fn is_nullable(&self) -> bool {
        match &self.ty {
            syn::Type::Path(tp) => {
                let expected: Vec<String> = vec!["Option".to_owned()];
                check_type_order(&tp, &expected, 0)
            },
            _ => false,
        }
    }

    fn get_db_type(&self) -> TokenStream2 {
        if self.is_primary_key() {
            return quote!{
                dboom::types::primary()
            }
        }

        if let Some(relation) = self.parse_relation() {
            let model = relation.model;
            let key = relation.key;

            let model_ident = format_ident!("{}", model);
            let table_name_acessor = quote!{ <#model_ident>::get_table_name() };

            return quote! {
                dboom::types::foreign(#table_name_acessor, #key)
            }
        }

        let segments = match &self.ty {
            syn::Type::Path(TypePath{path: Path{segments, ..}, ..}) => segments,
            _ => unimplemented!(),
        };

        match segments.first().unwrap() {
            PathSegment{ident, ..} if ident.to_string() == "String" => {
                quote! { dboom::types::text() }
            },
            PathSegment{ident, ..} if ident.to_string() == "i8" => {
                quote! { dboom::types::custom("char") }
            },
            PathSegment{ident, ..} if ident.to_string() == "i16" => {
                quote! { dboom::types::custom("SMALLINT") }
            },
            PathSegment{ident, ..} if ident.to_string() == "i32" => {
                quote! { dboom::types::integer() }
            },
            PathSegment{ident, ..} if ident.to_string() == "u32" => {
                quote! { dboom::types::custom("OID") }
            },
            PathSegment{ident, ..} if ident.to_string() == "i64" => {
                quote! { dboom::types::custom("BIGINT") }
            },
            PathSegment{ident, ..} if ident.to_string() == "f32" => {
                quote! { dboom::types::custom("REAL") }
            },
            PathSegment{ident, ..} if ident.to_string() == "f64" => {
                quote! { dboom::types::custom("DOUBLE PRECISION") }
            },
            PathSegment{ident, ..} if ident.to_string() == "bool" => {
                quote! { dboom::types::boolean() }
            },
            segment if is_chrono_option(segment) => {
                quote! { dboom::types::custom("timestamp with time zone") }
            },
            _ => unimplemented!(),
        }
    }
}