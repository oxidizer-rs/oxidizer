use proc_macro::TokenStream;

mod field_extras;
mod entity_builder;

#[proc_macro_derive(Entity, attributes(primary_key))]
pub fn entity_macro(item: TokenStream) -> TokenStream {
    entity_builder::EntityBuilder::new().build(item)
}


