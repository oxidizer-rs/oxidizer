use syn::{Meta, Field, TypePath, Path, PathSegment};
use proc_macro2::{TokenStream as TokenStream2};
use quote::quote;

pub trait FieldExtras {
    fn is_primary_key(&self) -> bool;
    fn get_db_type(&self) -> TokenStream2;
}

impl FieldExtras for Field {
    fn is_primary_key(&self) -> bool {
        for option in (&self.attrs).into_iter() {
            let option = option.parse_meta().unwrap();
            match option {
                Meta::Path(path) if path.get_ident().unwrap().to_string() == "primary_key" => {
                    return true;
                },
                _ => {},
            }
        }
        return false;
    }

    fn get_db_type(&self) -> TokenStream2 {
        if self.is_primary_key() {
            return quote!{
                dboom::types::primary()
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
            _ => unimplemented!(),
        }
    }
}