use std::path::PathBuf;
use syn;
use proc_macro2::TokenStream;

use super::Error;

pub fn generate_migration_from_file(file_path: &PathBuf) -> Result<String, Error> {
    let content = std::fs::read_to_string(file_path)?;

    let syntax = syn::parse_file(&content)?;

    let structs: Vec<TokenStream> = syntax.items.iter()
    .filter_map(|item| {
        match item {
            syn::Item::Struct(s) => Some(s),
            _ => None,
        }
    })
    .map(|item| {

    })
    .collect();

    Ok("".to_string())
}