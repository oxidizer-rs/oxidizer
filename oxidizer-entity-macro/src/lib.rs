use proc_macro::TokenStream;

mod attrs;
mod entity_builder;
mod field_extras;
mod props;
mod sql_builder;
mod utils;

/// Entity derive macro
#[proc_macro_derive(
    Entity,
    attributes(
        primary_key,
        relation,
        entity,
        has_many,
        field_ignore,
        custom_type,
        increments,
    )
)]
pub fn entity_macro(item: TokenStream) -> TokenStream {
    entity_builder::EntityBuilder::new().build(item)
}
