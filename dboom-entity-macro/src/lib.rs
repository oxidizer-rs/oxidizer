use proc_macro::TokenStream;

mod field_extras;
mod entity_builder;
mod props;
mod utils;

#[proc_macro_derive(Entity, attributes(primary_key, indexed))]
pub fn entity_macro(item: TokenStream) -> TokenStream {
    entity_builder::EntityBuilder::new().build(item)
}


