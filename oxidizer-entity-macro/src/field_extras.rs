use darling::FromMeta;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, quote_spanned};
use syn::{spanned::Spanned, Field, Type};

use super::attrs::{CustomTypeAttr, PrimaryKeyAttr, RelationAttr};
use super::utils::search_attr_in_field;
use super::utils::{check_type_order, is_integer_type};

pub trait FieldExtras {
    fn is_indexed(&self) -> bool;
    fn is_nullable(&self) -> bool;
    fn is_ignore(&self) -> bool;
    fn is_increments(&self) -> bool;
    fn parse_primary_key(&self) -> Option<PrimaryKeyAttr>;
    fn parse_relation(&self) -> Option<RelationAttr>;
    fn parse_custom_type(&self) -> Option<CustomTypeAttr>;
    fn get_type(&self) -> TokenStream2;
}

impl FieldExtras for Field {
    fn is_indexed(&self) -> bool {
        search_attr_in_field(self, "indexed")
    }

    fn is_ignore(&self) -> bool {
        search_attr_in_field(self, "field_ignore")
    }

    fn is_increments(&self) -> bool {
        if let Some(attr) = self.parse_primary_key() {
            return match attr.increments.as_ref() {
                Some(v) => *v,
                None => false,
            };
        }

        search_attr_in_field(self, "increments")
    }

    fn parse_primary_key(&self) -> Option<PrimaryKeyAttr> {
        for attr in (&self.attrs).into_iter() {
            let option = attr.parse_meta().unwrap();
            if let Ok(relation) = PrimaryKeyAttr::from_meta(&option) {
                return Some(relation);
            }
        }
        None
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

    fn parse_custom_type(&self) -> Option<CustomTypeAttr> {
        for attr in (&self.attrs).into_iter() {
            let option = attr.parse_meta().unwrap();
            if let Ok(ct) = CustomTypeAttr::from_meta(&option) {
                return Some(ct);
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

    fn get_type(&self) -> TokenStream2 {
        if let Some(ct) = self.parse_custom_type() {
            let ty = ct.ty;

            let ident = format_ident!("{}", ty);

            return quote! { #ident };
        }

        let ty = &self.ty;

        quote! { #ty }
    }
}
