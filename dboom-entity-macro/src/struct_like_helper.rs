use syn::{Data, DataStruct, DeriveInput, Fields, Meta, Type, Ident, punctuated::{Punctuated}, Field, token::Comma, ItemStruct, Attribute};

pub trait StructLike {
    fn get_ident(&self) -> &Ident;
    fn get_fields_all(&self) -> &Punctuated<Field, Comma>;
    fn get_attrs(&self) -> Vec<Attribute>;
}

impl StructLike for DeriveInput {
    fn get_ident(&self) -> &Ident {
        &self.ident
    }

    fn get_fields_all(&self) -> &Punctuated<Field, Comma> {
        match &self.data {
            Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) => &fields.named,
            _ => panic!("expected a struct with named fields"),
        }
    }

    fn get_attrs(&self) -> Vec<Attribute> {
        self.attrs
    }
}

impl StructLike for ItemStruct {
    fn get_ident(&self) -> &Ident {
        &self.ident
    }

    fn get_fields_all(&self) -> &Punctuated<Field, Comma> {
        match &self.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("expected a struct with named fields"),
        }
    }

    fn get_attrs(&self) -> Vec<Attribute> {
        self.attrs
    }
}