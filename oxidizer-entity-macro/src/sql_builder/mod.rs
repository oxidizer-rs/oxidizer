use proc_macro2::TokenStream as TokenStream2;

use super::attrs::HasManyAttr;
use super::attrs::RelationAttr;
use super::props::*;

mod postgres;

pub trait Builder {
    fn new() -> Self;

    fn build_save_query(props: &Props) -> TokenStream2;

    fn build_find_query(props: &Props) -> TokenStream2;

    fn build_first_query(props: &Props) -> TokenStream2;

    fn build_delete_query(props: &Props) -> TokenStream2;

    fn build_relation_get_query(props: &Props, relation: &RelationAttr) -> TokenStream2;

    fn build_relation_has_many_get_condition(props: &Props, attr: &HasManyAttr) -> TokenStream2;
}

pub type DefaultBuilder = postgres::PostgresBuilder;

#[derive(Debug)]
pub struct DBType(String);

pub trait ToDBType {
    fn to_db_type() -> DBType;
}
