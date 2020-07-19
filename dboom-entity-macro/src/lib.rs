use proc_macro::TokenStream;

mod field_extras;
mod entity_builder;
mod props;
mod utils;
mod attrs;

#[proc_macro_derive(Entity, attributes(primary_key, indexed, relation, entity, index))]
pub fn entity_macro(item: TokenStream) -> TokenStream {
    entity_builder::EntityBuilder::new().build(item)
}


