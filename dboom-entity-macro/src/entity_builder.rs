use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, Meta, Type, MetaNameValue, LitStr, Ident, punctuated::{Punctuated, Iter}, Field, token::Comma};

use super::field_extras::*;

pub struct EntityBuilder {
}

impl EntityBuilder {
    pub fn new() -> Self {
        EntityBuilder{}
    }

    fn build_db_types(&self, fields: &Punctuated<Field, Comma>) -> Vec<TokenStream2> {
        fields.iter().map(|field| field.get_db_type()).collect()
    }

    pub fn build(&self, item: TokenStream) -> TokenStream {
        let input = parse_macro_input!(item as DeriveInput);
        let name = &input.ident;
        let table_name = name.to_string().to_lowercase();

         let fields = match &input.data {
            Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) => &fields.named,
            _ => panic!("expected a struct with named fields"),
        };
        let field_name: Vec<&std::option::Option<Ident>> = fields.iter().map(|field| &field.ident).collect();
        let field_type: Vec<&Type> = fields.iter().map(|field| &field.ty).collect();
        let field_db_type = self.build_db_types(fields);

        let primary_key_field = fields.iter().find(|field| field.is_primary_key()).unwrap();
        let primary_key_ident = &primary_key_field.ident;
        let primary_key_ty = &primary_key_field.ty;

        let field_name_plain: Vec<&std::option::Option<Ident>> = fields.iter().filter(|field| {
            for option in (&field.attrs).into_iter() {
                let option = option.parse_meta().unwrap();
                match option {
                    Meta::Path(path) if path.get_ident().unwrap().to_string() == "primary_key" => {
                        return false;
                    },
                    _ => {},
                }
            }
            return true;
        }).map(|field| &field.ident).collect();
        let field_type_plain: Vec<&Type> = fields.iter().filter(|field| {
            for option in (&field.attrs).into_iter() {
                let option = option.parse_meta().unwrap();
                match option {
                    Meta::Path(path) if path.get_ident().unwrap().to_string() == "primary_key" => {
                        return false;
                    },
                    _ => {},
                }
            }
            return true;
        }).map(|field| &field.ty).collect();

        let fields_numbered_plain: Vec<String> = field_name_plain.iter().enumerate().map(|(i, _)| "$".to_string() + &(i+1).to_string()).collect();
        let fields_numbered_plain_next_index = (fields_numbered_plain.len() + 1).to_string();

        eprintln!("{:#?}", field_type);

        let expanded = quote! {
            pub use dboom::entity::Entity;

            #[dboom::async_trait]
            impl dboom::entity::Entity for #name {
                async fn save(&mut self, db: &dboom::db::DB) -> dboom::db::DBResult<bool> {
                    let mut creating = false;
                    let _result = match self.id {
                        0 => {
                            creating = true;
                            let rows = db.query(
                                concat!(
                                    "INSERT INTO ",
                                    #table_name,
                                    " (",
                                    stringify!(#(#field_name_plain),*),
                                    ") values(",
                                    #(#fields_numbered_plain),*,
                                    ") RETURNING ",
                                    stringify!(#primary_key_ident),
                                    ";"
                                ),
                                &[#( &self.#field_name_plain),*]
                            ).await?;
                            let first_row = rows.first().ok_or(dboom::db::Error::Other)?;
                            self.#primary_key_ident = first_row.get::<&str, #primary_key_ty>(stringify!(#primary_key_ident));
                            1
                        },
                        id => {
                            db.execute(
                                concat!(
                                    "UPDATE ",
                                    #table_name,
                                    " SET ",
                                    #(stringify!(#field_name_plain =), #fields_numbered_plain),*,
                                    " WHERE ",
                                    stringify!(#primary_key_ident),
                                    "= $",
                                    #fields_numbered_plain_next_index
                                ),
                                &[#( &self.#field_name_plain),*, &self.#primary_key_ident],
                            ).await?
                        }
                    };

                    Ok(creating)
                }

                async fn delete(&self, db: &dboom::db::DB) -> dboom::db::DBResult<bool> {
                    todo!();
                }

                async fn from_row(row: &dboom::tokio_postgres::Row) -> Self {
                    let mut obj: Self = Self{
                        #(
                            #field_name: row.get::<&str, #field_type>(concat!(stringify!(#field_name))),
                        )*
                    };
                    obj
                }

                async fn create_migration() -> dboom::db::DBResult<dboom::Migration> {
                    let mut m = dboom::Migration::new();
                    m.create_table(#table_name, |t| {
                        #(t.add_column(stringify!(#field_name), #field_db_type);)*
                    });
                    Ok(m)
                }
            }
        };

        // Hand the output tokens back to the compiler
        let r = TokenStream::from(expanded);

        println!("{}", r);

        r
    }
}