use darling::FromMeta;
use inflector::cases::snakecase::to_snake_case;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Type};

use super::attrs::HasManyAttr;
use super::attrs::{EntityAttr, IndexAttr};
use super::field_extras::*;
use super::props::*;

pub struct EntityBuilder {}

impl EntityBuilder {
    pub fn new() -> Self {
        EntityBuilder {}
    }

    fn build_save_fn(&self, props: &Props) -> TokenStream2 {
        let table_name = props.get_table_name();

        let fields_ident: Vec<&Option<syn::Ident>> = props.get_fields_all().map(|field| &field.ident).collect();
        let mut current_index = 1;
        let fields_query_values = props.get_fields_all().map(|field| {
            match field.is_increments() {
                true => "DEFAULT".to_string(),
                false => {
                    let v = current_index;
                    current_index += 1;
                    format!("${}", v)
                },
            }
        }).collect::<Vec<String>>().join(",");

        let fields_value_acessors: Vec<TokenStream2> = props
            .get_fields_all()
            .filter(|field| !field.is_increments())
            .map(|field| {
                let name = &field.ident;
                if let Some(ct) = field.parse_custom_type() {
                    let ty = ct.ty;

                    let ty_ident = format_ident!("{}", ty);

                    return quote! { &<#ty_ident>::try_from(&self.#name)? };
                }

                quote! { &self.#name }
            })
            .collect();

        let mut current_index = 0;
        let mut comma_index = 0;
        let fields_plain_to_set: Vec<TokenStream2> = props.get_fields_all()
            .filter_map(|field| {
                if field.is_increments() {
                    return None;
                }

                current_index += 1;

                if field.parse_primary_key().is_some() {
                    return None;
                }

                let ident = &field.ident;
                let v = format!("${}", current_index);
                let comma = match comma_index {
                    0 => quote!{},
                    _ => quote! {,},
                };
                comma_index += 1;

                Some(quote! {
                    concat!(stringify!(#comma #ident =), #v)
                })
            })
            .collect();

        let on_conflict_do = match fields_plain_to_set.len() {
            0 => quote!{"NOTHING"},
            _ => quote! {"UPDATE SET ", #(#fields_plain_to_set),* },
        };

        let primary_key = props.get_primary_key_field().unwrap();
        let primary_key_ident = &primary_key.ident;
        let primary_key_type = &primary_key.ty;

        quote! {
            async fn save(&mut self, db: &oxidizer::db::DB) -> oxidizer::db::DBResult<bool> {
                let mut creating = false;
                let primary_key_default: #primary_key_type = Default::default();

                if self.#primary_key_ident == primary_key_default {
                    creating = true;
                }

                let query = concat!("INSERT INTO ", #table_name,
                        " (", stringify!(#(#fields_ident),*),
                        ") values (", #fields_query_values,
                        ") ON CONFLICT (", stringify!(#primary_key_ident), ") DO ", #on_conflict_do,
                        " RETURNING ", stringify!(#primary_key_ident), ";"
                    );
                println!("{}", query);
                let rows = db.query(
                    query,
                    &[#( #fields_value_acessors ),*]
                ).await?;
                let first_row = rows.first().ok_or(oxidizer::db::Error::Other("Error while saving entity".to_string()))?;
                self.#primary_key_ident = first_row.get::<&str, #primary_key_type>(stringify!(#primary_key_ident));

                Ok(creating)
            }
        }
    }

    fn build_from_row_fn(&self, props: &Props) -> TokenStream2 {
        let fields_all_loaders: Vec<TokenStream2> = props
            .get_fields_all()
            .map(|field| {
                let name = &field.ident;

                let ty = field.get_type();

                let mut converter = quote! {};
                let mut converter_pos = quote! {};

                if let Some(_) = field.parse_custom_type() {
                    let custom_ty = &field.ty;
                    converter = quote! { <#custom_ty>::try_from };
                    converter_pos  = quote! {?};
                }

                quote! {
                    #name: #converter(row.get::<&str, #ty>(concat!(stringify!(#name))))#converter_pos,
                }
            })
            .collect();

        let fields_ignored_names: Vec<&Option<syn::Ident>> = props
            .get_ignored_fields()
            .map(|field| &field.ident)
            .collect();
        let fields_ignored_types: Vec<&syn::Type> =
            props.get_ignored_fields().map(|field| &field.ty).collect();

        quote! {
            fn from_row(row: &oxidizer::tokio_postgres::Row) -> oxidizer::db::DBResult<Self> {
                let mut obj: Self = Self{
                    #( #fields_all_loaders )*
                    #(
                        #fields_ignored_names: <#fields_ignored_types>::default(),
                    )*
                };
                Ok(obj)
            }
        }
    }

    fn build_create_migration_fn(&self, props: &Props) -> TokenStream2 {
        let table_name = props.get_table_name();
        let fields_all_names = props.get_fields_all_names();
        let fields_all_db_types = props.get_fields_all_db_types();
        let fields_all_nullable = props.get_fields_all_nullable();
        let fields_all_indexed = props.get_fields_all_indexed();
        let fields_all_primary: Vec<bool> = props.get_fields_all_primary().iter().map(|attr| attr.is_some()).collect();

        let indexes: Vec<TokenStream2> = props
            .get_indexes()
            .iter()
            .map(|index| {
                let index_name = &index.name;
                let columns: Vec<&str> = index.columns.split(",").map(|c| c.trim()).collect();
                let unique = index.unique;
                quote! {
                    t.add_index(
                        #index_name,
                        oxidizer::types::index(vec![ #(#columns),* ]).unique(#unique)
                    );
                }
            })
            .collect();

        quote! {
             fn create_migration() -> oxidizer::db::DBResult<oxidizer::migration::Migration> {
                let mut m = oxidizer::migration::Migration::new(#table_name);
                m.raw.create_table(#table_name, |t| {
                    #(t
                        .add_column(
                            stringify!(#fields_all_names),
                            #fields_all_db_types
                                .nullable(#fields_all_nullable)
                                .indexed(#fields_all_indexed)
                                .primary(#fields_all_primary)
                        )
                    ;)*

                    #(#indexes)*
                });

                Ok(m)
            }
        }
    }

    fn build_find_fn(&self, props: &Props) -> TokenStream2 {
        let name = props.get_name();
        let table_name = props.get_table_name();
        quote! {
            async fn find(db: &oxidizer::db::DB, condition: &str, params: &'_ [&'_ (dyn oxidizer::db_types::ToSql + Sync)]) -> oxidizer::db::DBResult<Vec<#name>> {
                let query_str = format!("SELECT * FROM {} WHERE {}", #table_name, condition);
                let rows = db.query(&query_str, params).await?;

                let mut results: Vec<#name> = Vec::with_capacity(rows.len());

                for row in rows.iter() {
                    results.push(Self::from_row(row)?);
                }

                Ok(results)
            }
        }
    }

    fn build_first_fn(&self, props: &Props) -> TokenStream2 {
        let name = props.get_name();
        let table_name = props.get_table_name();
        quote! {
            async fn first(db: &oxidizer::db::DB, condition: &str, params: &'_ [&'_ (dyn oxidizer::db_types::ToSql + Sync)]) -> oxidizer::db::DBResult<std::option::Option<#name>> {
                let query_str = format!("SELECT * FROM {} WHERE {} LIMIT 1", #table_name, condition);
                let rows = db.query(&query_str, params).await?;

                let mut results: Vec<#name> = Vec::with_capacity(rows.len());
                for row in rows.iter() {
                    results.push(Self::from_row(row)?);
                }

                match results.len() {
                    0 => Ok(None),
                    _ => Ok(Some(results.remove(0))),
                }
            }
        }
    }

    fn build_delete_fn(&self, props: &Props) -> TokenStream2 {
        let primary_key_ident = &props.get_primary_key_field().unwrap().ident;
        let primary_key_type = &props.get_primary_key_field().unwrap().ty;
        let table_name = props.get_table_name();
        quote! {
            async fn delete(&mut self, db: &oxidizer::db::DB) -> oxidizer::db::DBResult<bool> {
                let key_default: #primary_key_type = Default::default();
                if self.#primary_key_ident == key_default {
                    return Ok(false);
                }

                let condition = format!("{} = $1", stringify!(#primary_key_ident));
                let query_str = format!("DELETE FROM {} WHERE {}", #table_name, condition);
                match db.execute(&query_str, &[&self.#primary_key_ident]).await? {
                    0 => Ok(false),
                    _ => {
                        self.#primary_key_ident = key_default;
                        Ok(true)
                    },
                }
            }
        }
    }

    fn build_is_synced_with_db_fn(&self, props: &Props) -> TokenStream2 {
        let primary_key_ident = &props.get_primary_key_field().unwrap().ident;
        let primary_key_type = &props.get_primary_key_field().unwrap().ty;
        quote! {
            fn is_synced_with_db(&self) -> bool {
                let key_default: #primary_key_type = Default::default();
                self.#primary_key_ident != key_default
            }
        }
    }

    fn build_foreign_helpers(&self, props: &Props) -> Vec<TokenStream2> {
        let name = props.get_name();

        let foreign_fields = props.get_fields_foreign();

        foreign_fields.iter().map(|field| {
            let relation = field.parse_relation().unwrap();
            let local_key = field.ident.clone().unwrap();
            let local_key_type = &field.ty;
            let get_ident = format_ident!("get_{}", to_snake_case(&relation.model));
            let set_ident = format_ident!("set_{}", to_snake_case(&relation.model));
            let trait_ident = format_ident!("__Accessor{}To{}", name, relation.model);
            let model = format_ident!("{}", relation.model);
            let key = format_ident!("{}", relation.key);

            let local_key_set = match field.is_nullable() {
                true => quote! {
                    self.#local_key = Some(v.#key);
                },
                false => quote! {
                    self.#local_key = v.#key;
                },
            };

            quote! {
                #[oxidizer::async_trait]
                pub trait #trait_ident {
                    async fn #get_ident(&self, db: &oxidizer::db::DB) -> oxidizer::db::DBResult<#model>;
                    async fn #set_ident(&mut self, db: &oxidizer::db::DB, v: &#model) -> oxidizer::db::DBResult<()>;
                }

                #[oxidizer::async_trait]
                impl #trait_ident for #name {
                    async fn #get_ident(&self, db: &oxidizer::db::DB) -> oxidizer::db::DBResult<#model> {
                        if self.#local_key == <#local_key_type>::default() {
                            return Err(oxidizer::db::Error::DoesNotExist);
                        }

                        let table_name = <#model>::get_table_name();
                        let query = format!("select * from {} where {} = $1 limit 1", &table_name, stringify!(#key));
                        let results = db.query(&query, &[&self.#local_key]).await?;
                        if results.len() == 0 {
                            return Err(oxidizer::db::Error::DoesNotExist);
                        }

                        #model::from_row(&results[0])
                    }

                    async fn #set_ident(&mut self, db: &oxidizer::db::DB, v: &#model) -> oxidizer::db::DBResult<()> {
                        if !v.is_synced_with_db() {
                            return Err(oxidizer::db::Error::ReferencedModelIsNotInDB);
                        }

                        #local_key_set
                        self.save(db).await?;
                        Ok(())
                    }
                }
            }
        }).collect()
    }

    fn build_has_many_helpers(&self, props: &Props) -> Vec<TokenStream2> {
        let name = props.get_name();

        props.get_has_many_attrs().iter().map(|attr| {
            let model_snake_cased = to_snake_case(&attr.model);

            let get_ident = format_ident!("get_all_{}", model_snake_cased);

            let trait_ident = format_ident!("__AccessorHasMany{}To{}", name, attr.model);

            let model = match attr.through.as_ref() {
                Some(m) => format_ident!("{}", m),
                None => format_ident!("{}", attr.model),
            };

            let field = &attr.field;

            let pk = &props.get_primary_key_field().unwrap().ident;

            quote! {
                #[oxidizer::async_trait]
                pub trait #trait_ident {
                    async fn #get_ident(&self, db: &oxidizer::db::DB) -> oxidizer::db::DBResult<Vec<#model>>;
                }

                #[oxidizer::async_trait]
                impl #trait_ident for #name {
                    async fn #get_ident(&self, db: &oxidizer::db::DB) -> oxidizer::db::DBResult<Vec<#model>> {
                        let query = format!("{} = $1", #field);
                        <#model>::find(db, &query, &[ &self.#pk ]).await
                    }
                }
            }

        }).collect()
    }

    pub fn build(&self, item: TokenStream) -> TokenStream {
        let input = parse_macro_input!(item as DeriveInput);

        let mut attrs: Option<EntityAttr> = None;

        let mut indexes: Vec<IndexAttr> = vec![];

        let mut has_many_attrs: Vec<HasManyAttr> = vec![];

        for option in input.attrs.iter() {
            let option = option.parse_meta().unwrap();
            if let Ok(v) = EntityAttr::from_meta(&option) {
                attrs = Some(v);
            }

            if let Ok(v) = IndexAttr::from_meta(&option) {
                indexes.push(v);
            }

            if let Ok(v) = HasManyAttr::from_meta(&option) {
                has_many_attrs.push(v);
            }
        }

        // eprintln!("{:#?}", input);
        // eprintln!("{:#?}", attrs);

        let props = Props::new(input, attrs, indexes, has_many_attrs);

        if let Some(ts) = props.check() {
            return ts;
        }

        let save_fn = self.build_save_fn(&props);
        let delete_fn = self.build_delete_fn(&props);
        let is_synced_with_db = self.build_is_synced_with_db_fn(&props);
        let from_row_fn = self.build_from_row_fn(&props);
        let create_migration_fn = self.build_create_migration_fn(&props);
        let find_fn = self.build_find_fn(&props);
        let first_fn = self.build_first_fn(&props);

        let name = props.get_name();
        let table_name = props.get_table_name();

        let foreign_helpers = self.build_foreign_helpers(&props);

        let has_many_helpers = self.build_has_many_helpers(&props);

        let expanded = quote! {
            #[oxidizer::async_trait]
            impl oxidizer::entity::IEntity for #name {
                #save_fn

                #delete_fn

                #is_synced_with_db

                #find_fn

                #first_fn

                #from_row_fn

                #create_migration_fn

                fn get_table_name() -> String {
                    #table_name.to_string()
                }
            }

            #(#foreign_helpers)*

            #(#has_many_helpers)*
        };

        // Hand the output tokens back to the compiler
        let r = TokenStream::from(expanded);

        // println!("{}", r);

        r
    }
}
