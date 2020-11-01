use inflector::cases::snakecase::to_snake_case;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::{
    punctuated::Punctuated, spanned::Spanned, token::Comma, Data, DataStruct, DeriveInput, Field,
    Fields, Ident, Meta, PathArguments, PathSegment, Type,
};

use super::attrs::HasManyAttr;
use super::attrs::{EntityAttr, IndexAttr, PrimaryKeyAttr};
use super::field_extras::*;
use super::utils::is_integer_type;

pub struct Props {
    input: DeriveInput,
    attrs: Option<EntityAttr>,
    indexes: Vec<IndexAttr>,
    has_many_attrs: Vec<HasManyAttr>,
}

type GetFieldsIter<'a> = std::iter::Filter<syn::punctuated::Iter<'a, Field>, fn(&&Field) -> bool>;

impl Props {
    pub fn new(
        input: DeriveInput,
        attrs: Option<EntityAttr>,
        indexes: Vec<IndexAttr>,
        has_many_attrs: Vec<HasManyAttr>,
    ) -> Self {
        Props {
            input: input,
            attrs: attrs,
            indexes: indexes,
            has_many_attrs: has_many_attrs,
        }
    }

    pub fn get_name(&self) -> &Ident {
        &self.input.ident
    }

    pub fn get_table_name(&self) -> String {
        let snaked_name = to_snake_case(&self.get_name().to_string());

        match self.attrs.as_ref() {
            Some(attrs) => match attrs.table_name.as_ref() {
                Some(name) => name.to_string(),
                None => snaked_name,
            },
            None => snaked_name,
        }
    }

    pub fn get_fields_all(&self) -> GetFieldsIter {
        let fields = match &self.input.data {
            Data::Struct(DataStruct {
                fields: Fields::Named(fields),
                ..
            }) => &fields.named,
            _ => panic!("expected a struct with named fields"),
        };

        fields.iter().filter(|field| !field.is_ignore())
    }

    pub fn get_ignored_fields(&self) -> GetFieldsIter {
        let fields = match &self.input.data {
            Data::Struct(DataStruct {
                fields: Fields::Named(fields),
                ..
            }) => &fields.named,
            _ => panic!("expected a struct with named fields"),
        };

        fields.iter().filter(|field| field.is_ignore())
    }

    pub fn get_fields_all_names(&self) -> Vec<&Option<Ident>> {
        self.get_fields_all().map(|field| &field.ident).collect()
    }

    pub fn get_fields_all_types(&self) -> Vec<TokenStream2> {
        self.get_fields_all()
            .map(|field| field.get_type())
            .collect()
    }

    pub fn get_fields_all_nullable(&self) -> Vec<bool> {
        self.get_fields_all()
            .map(|field| field.is_nullable())
            .collect()
    }

    pub fn get_fields_all_indexed(&self) -> Vec<bool> {
        self.get_fields_all()
            .map(|field| field.is_indexed())
            .collect()
    }

    pub fn get_fields_all_primary(&self) -> Vec<Option<PrimaryKeyAttr>> {
        self.get_fields_all()
            .map(|field| field.parse_primary_key())
            .collect()
    }

    pub fn get_fields_all_increments(&self) -> Vec<bool> {
        self.get_fields_all()
            .map(|field| field.is_increments())
            .collect()
    }

    fn build_db_types(&self, fields: GetFieldsIter) -> Vec<TokenStream2> {
        fields.map(|field| field.get_db_type()).collect()
    }

    pub fn get_fields_all_db_types(&self) -> Vec<TokenStream2> {
        self.build_db_types(self.get_fields_all())
    }

    pub fn get_primary_key_field(&self) -> Option<&Field> {
        self.get_fields_all()
            .find(|field| field.parse_primary_key().is_some())
    }

    pub fn get_fields_plain(&self) -> Vec<&Field> {
        self.get_fields_all()
            .filter(|field| field.parse_primary_key().is_none())
            .collect()
    }

    pub fn get_fields_plain_names(&self) -> Vec<&Option<Ident>> {
        self.get_fields_plain()
            .iter()
            .map(|field| &field.ident)
            .collect()
    }

    pub fn get_fields_plain_numbered(&self) -> Vec<String> {
        self.get_fields_plain_names()
            .iter()
            .enumerate()
            .map(|(i, _)| "$".to_string() + &(i + 1).to_string())
            .collect()
    }

    pub fn get_fields_plain_numbered_next_index(&self) -> String {
        (self.get_fields_plain_numbered().len() + 1).to_string()
    }

    pub fn check(&self) -> Option<TokenStream> {
        if let None = self.get_primary_key_field() {
            return Some(TokenStream::from(
                quote! { compile_error!("No primary key defined") },
            ));
        }

        // checks auto-increments
        for field in self.get_fields_all() {
            if !field.is_increments() {
                continue;
            }

            if field.parse_primary_key().is_none() {
                return Some(TokenStream::from(quote_spanned! {
                    field.ty.span() => compile_error!(
                        "Increments can only be used with primary keys for now"
                    )
                }));
            }

            let allowed_increments_types = vec!["i32"];

            let (check, ty) = is_integer_type(&field.ty);
            if !check || !allowed_increments_types.contains(&ty) {
                return Some(TokenStream::from(quote_spanned! {
                    field.ty.span() => compile_error!(
                        "Increments can only be used with integer types: 'i32'"
                    )
                }));
            }
        }

        // TODO this limitation should go away eventually
        if self
            .get_fields_all()
            .filter(|field| field.parse_primary_key().is_some())
            .count()
            == 1
        {
            return None;
        }

        let last_primary_key = self
            .get_fields_all()
            .filter(|field| field.parse_primary_key().is_some())
            .last()
            .unwrap();
        let expanded = quote_spanned! {
            last_primary_key.ident.as_ref().unwrap().span() => compile_error!("Multiple primary keys defined")
        };
        Some(TokenStream::from(expanded))
    }

    pub fn get_fields_foreign(&self) -> Vec<&Field> {
        self.get_fields_all()
            .filter(|field| field.parse_relation().is_some())
            .collect()
    }

    pub fn get_indexes(&self) -> Vec<IndexAttr> {
        self.indexes.clone()
    }

    pub fn get_has_many_attrs(&self) -> Vec<HasManyAttr> {
        self.has_many_attrs.clone()
    }
}
