use darling::FromMeta;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Field, Meta, Path, PathSegment, TypePath};

use super::attrs::RelationAttr;
use super::utils::{check_type_order, iterate_path_arguments};

pub trait FieldExtras {
    fn is_primary_key(&self) -> bool;
    fn is_indexed(&self) -> bool;
    fn is_nullable(&self) -> bool;
    fn parse_relation(&self) -> Option<RelationAttr>;
    fn get_db_type(&self) -> TokenStream2;
}

fn is_typed_with(segment: &PathSegment, expected: Vec<&str>) -> bool {
    let expected = expected.iter().map(|v| v.to_string()).collect();
    iterate_path_arguments(segment, &expected, 0)
}

fn is_chrono_option(segment: &PathSegment) -> bool {
    let expected: Vec<&str> = vec!["Option", "DateTime", "Utc"];
    let no_option_expected: Vec<&str> = vec!["DateTime", "Utc"];

    is_typed_with(segment, expected) || is_typed_with(segment, no_option_expected)
}

fn search_attr_in_field(field: &Field, attr: &str) -> bool {
    for option in (&field.attrs).into_iter() {
        let option = option.parse_meta().unwrap();
        match option {
            Meta::Path(path) if path.get_ident().unwrap().to_string() == attr => {
                return true;
            }
            _ => {}
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
            }
            _ => false,
        }
    }

    fn get_db_type(&self) -> TokenStream2 {
        if self.is_primary_key() {
            return quote! {
                oxidizer::types::primary()
            };
        }

        if let Some(relation) = self.parse_relation() {
            let model = relation.model;
            let key = relation.key;

            let model_ident = format_ident!("{}", model);
            let table_name_acessor = quote! { <#model_ident>::get_table_name() };

            return quote! {
                oxidizer::types::foreign(#table_name_acessor, #key)
            };
        }

        let segments = match &self.ty {
            syn::Type::Path(TypePath {
                path: Path { segments, .. },
                ..
            }) => segments,
            _ => unimplemented!(),
        };

        match segments.first().unwrap() {
            PathSegment { ident, .. } if ident.to_string() == "String" => {
                quote! { oxidizer::types::text() }
            }
            segment if is_typed_with(segment, vec!["Option", "String"]) => {
                quote! { oxidizer::types::text() }
            }

            PathSegment { ident, .. } if ident.to_string() == "i8" => {
                quote! { oxidizer::types::custom("char") }
            }
            segment if is_typed_with(segment, vec!["Option", "i8"]) => {
                quote! { oxidizer::types::text() }
            }

            PathSegment { ident, .. } if ident.to_string() == "i16" => {
                quote! { oxidizer::types::custom("SMALLINT") }
            }
            segment if is_typed_with(segment, vec!["Option", "i16"]) => {
                quote! { oxidizer::types::text() }
            }

            PathSegment { ident, .. } if ident.to_string() == "i32" => {
                quote! { oxidizer::types::integer() }
            }
            segment if is_typed_with(segment, vec!["Option", "i32"]) => {
                quote! { oxidizer::types::text() }
            }

            PathSegment { ident, .. } if ident.to_string() == "u32" => {
                quote! { oxidizer::types::custom("OID") }
            }
            segment if is_typed_with(segment, vec!["Option", "u32"]) => {
                quote! { oxidizer::types::text() }
            }

            PathSegment { ident, .. } if ident.to_string() == "i64" => {
                quote! { oxidizer::types::custom("BIGINT") }
            }
            segment if is_typed_with(segment, vec!["Option", "i64"]) => {
                quote! { oxidizer::types::text() }
            }

            PathSegment { ident, .. } if ident.to_string() == "f32" => {
                quote! { oxidizer::types::custom("REAL") }
            }
            segment if is_typed_with(segment, vec!["Option", "f32"]) => {
                quote! { oxidizer::types::text() }
            }

            PathSegment { ident, .. } if ident.to_string() == "f64" => {
                quote! { oxidizer::types::custom("DOUBLE PRECISION") }
            }
            segment if is_typed_with(segment, vec!["Option", "f64"]) => {
                quote! { oxidizer::types::text() }
            }

            PathSegment { ident, .. } if ident.to_string() == "bool" => {
                quote! { oxidizer::types::boolean() }
            }
            segment if is_typed_with(segment, vec!["Option", "bool"]) => {
                quote! { oxidizer::types::text() }
            }

            segment if is_chrono_option(segment) => {
                quote! { oxidizer::types::custom("timestamp with time zone") }
            }
            _ => unimplemented!(),
        }
    }
}
