use inflector::cases::snakecase::to_snake_case;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::{
    punctuated::Punctuated, token::Comma, Data, DataStruct, DeriveInput, Field, Fields, Ident,
    Meta, Type,
};

use super::attrs::HasManyAttr;
use super::attrs::{EntityAttr, IndexAttr};
use super::field_extras::*;

pub struct Props {
    input: DeriveInput,
    attrs: Option<EntityAttr>,
    indexes: Vec<IndexAttr>,
    has_many_attrs: Vec<HasManyAttr>,
}

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

    pub fn get_fields_all(&self) -> &Punctuated<Field, Comma> {
        match &self.input.data {
            Data::Struct(DataStruct {
                fields: Fields::Named(fields),
                ..
            }) => &fields.named,
            _ => panic!("expected a struct with named fields"),
        }
    }

    pub fn get_fields_all_names(&self) -> Vec<&Option<Ident>> {
        self.get_fields_all()
            .iter()
            .map(|field| &field.ident)
            .collect()
    }

    pub fn get_fields_all_types(&self) -> Vec<&Type> {
        self.get_fields_all()
            .iter()
            .map(|field| &field.ty)
            .collect()
    }

    pub fn get_fields_all_nullable(&self) -> Vec<bool> {
        self.get_fields_all()
            .iter()
            .map(|field| field.is_nullable())
            .collect()
    }

    pub fn get_fields_all_indexed(&self) -> Vec<bool> {
        self.get_fields_all()
            .iter()
            .map(|field| field.is_indexed())
            .collect()
    }

    fn build_db_types(&self, fields: &Punctuated<Field, Comma>) -> Vec<TokenStream2> {
        fields.iter().map(|field| field.get_db_type()).collect()
    }

    pub fn get_fields_all_db_types(&self) -> Vec<TokenStream2> {
        self.build_db_types(self.get_fields_all())
    }

    pub fn get_primary_key_field(&self) -> Option<&Field> {
        self.get_fields_all()
            .iter()
            .find(|field| field.is_primary_key())
    }

    pub fn get_fields_plain_names(&self) -> Vec<&Option<Ident>> {
        self.get_fields_all()
            .iter()
            .filter(|field| {
                for option in (&field.attrs).into_iter() {
                    let option = option.parse_meta().unwrap();
                    match option {
                        Meta::Path(path)
                            if path.get_ident().unwrap().to_string() == "primary_key" =>
                        {
                            return false;
                        }
                        _ => {}
                    }
                }
                return true;
            })
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

        let fields = self.get_fields_all();
        if fields.iter().filter(|field| field.is_primary_key()).count() > 1 {
            let last_primary_key = fields
                .iter()
                .filter(|field| field.is_primary_key())
                .last()
                .unwrap();
            let expanded = quote_spanned! {
                last_primary_key.ident.as_ref().unwrap().span() => compile_error!("Multiple primary keys defined")
            };
            return Some(TokenStream::from(expanded));
        }

        None
    }

    pub fn get_fields_foreign(&self) -> Vec<&Field> {
        self.get_fields_all()
            .iter()
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
