use darling::FromMeta;
use inflector::cases::snakecase::to_snake_case;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Type};

use crate::attrs::HasManyAttr;
use crate::attrs::{EntityAttr, IndexAttr, RelationAttr};
use crate::field_extras::*;
use crate::props::*;
use crate::utils::is_integer_type;

use super::Builder;

pub struct PostgresBuilder {}

impl Builder for PostgresBuilder {
    fn new() -> Self {
        PostgresBuilder {}
    }

    fn build_save_query(props: &crate::props::Props) -> TokenStream2 {
        let table_name = props.get_table_name();

        let fields_ident: Vec<&Option<syn::Ident>> =
            props.get_fields_all().map(|field| &field.ident).collect();
        let mut current_index = 1;
        let fields_query_values = props
            .get_fields_all()
            .map(|field| {
                let v = current_index;
                current_index += 1;

                match field.parse_primary_key().is_some() && field.is_increments() {
                    true => {
                        let bigserial_types = vec!["i64"];
                        let cast = match bigserial_types.contains(&is_integer_type(&field.ty).1) {
                            true => "int8",
                            false => "int4",
                        };

                        format!(
                            "COALESCE(${}, CAST(nextval(pg_get_serial_sequence('{}', '{}')) AS {}))",
                            v,
                            table_name,
                            field.ident.as_ref().unwrap().to_string(),
                            cast,
                        )
                    }
                    false => format!("${}", v),
                }
            })
            .collect::<Vec<String>>()
            .join(",");

        let mut current_index = 0;
        let mut comma_index = 0;
        let fields_plain_to_set: Vec<TokenStream2> = props
            .get_fields_all()
            .filter_map(|field| {
                //if field.is_increments() {
                //return None;
                //}

                current_index += 1;

                if field.parse_primary_key().is_some() {
                    return None;
                }

                let ident = &field.ident;
                let v = format!("${}", current_index);
                let comma = match comma_index {
                    0 => quote! {},
                    _ => quote! {,},
                };
                comma_index += 1;

                Some(quote! {
                    concat!(stringify!(#comma #ident =), #v)
                })
            })
            .collect();

        let on_conflict_do = match fields_plain_to_set.len() {
            0 => quote! {"NOTHING"},
            _ => quote! {"UPDATE SET ", #(#fields_plain_to_set),* },
        };

        let primary_key = props.get_primary_key_field().unwrap();
        let primary_key_ident = &primary_key.ident;

        quote! {
            let query = concat!("INSERT INTO \"", #table_name, "\"",
                " (", stringify!(#(#fields_ident),*),
                ") values (", #fields_query_values,
                ") ON CONFLICT (", stringify!(#primary_key_ident), ") DO ", #on_conflict_do,
                " RETURNING ", stringify!(#primary_key_ident), ";"
            );
        }
    }

    fn build_find_query(props: &Props) -> TokenStream2 {
        let table_name = props.get_table_name();
        quote! {
            let query = format!("SELECT * FROM \"{}\" WHERE {}", #table_name, condition)
        }
    }

    fn build_first_query(props: &Props) -> TokenStream2 {
        let table_name = props.get_table_name();
        quote! {
            let query = format!("SELECT * FROM \"{}\" WHERE {} LIMIT 1", #table_name, condition);
        }
    }

    fn build_delete_query(props: &Props) -> TokenStream2 {
        let primary_key_ident = &props.get_primary_key_field().unwrap().ident;
        let table_name = props.get_table_name();
        quote! {
            let condition = format!("{} = $1", stringify!(#primary_key_ident));
            let query = format!("DELETE FROM \"{}\" WHERE {}", #table_name, condition);
        }
    }

    fn build_relation_get_query(props: &Props, relation: &RelationAttr) -> TokenStream2 {
        let model = format_ident!("{}", relation.model);
        let key = format_ident!("{}", relation.key);

        quote! {
            let table_name = <#model>::get_table_name();
            let query = format!("select * from \"{}\" where {} = $1 limit 1", &table_name, stringify!(#key));
        }
    }

    fn build_relation_has_many_get_condition(props: &Props, attr: &HasManyAttr) -> TokenStream2 {
        let field = &attr.field;

        quote! {
            let query = format!("{} = $1", #field);
        }
    }
}
