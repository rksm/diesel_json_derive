//! [![Crates.io Version](https://img.shields.io/crates/v/diesel-json-derive)](https://crates.io/crates/diesel-json-derive)
//!
//! __NOTE:__ This is under active development. No guarantees for stability or usability. You probably want [diesel_json](https://crates.io/crates/diesel_json) instead. Please also note that this currently expects postgres. Pull requests to support other backends are welcome.
//!
//! ## diesel_json_derive
//!
//! What's this? This is a procedural macro that automatically derives `ToSql` and `FromSql` for Diesel's `Jsonb` type.
//!
//! Consider a table like
//!
//! ```sql
//! CREATE TABLE foo (
//!   id TEXT PRIMARY KEY,
//!   bar JSONB NOT NULL
//! );
//! ```
//!
//! which is in Rust can be represented as as (does not compile!):
//!
//! ```rust
//! #[derive(Debug, Queryable, Identifiable, Insertable, AsChangeset, Selectable)]
//! #[diesel(table_name = crate::schema::foo)]
//! #[diesel(check_for_backend(diesel::pg::Pg))]
//! #[diesel(primary_key(id))]
//! struct Foo {
//!     id: String,
//!     bar: Bar,
//! }
//! 
//! #[derive(Debug)]
//! struct Bar {
//!     x: i32,
//! }
//! ```
//!
//! In order to make `Bar` be represented as a jsonb blob you will need to implement the `diesel::deserialize::FromSql` and `diesel::deserialize::FromSql` traits, e.g. like this:
//!
//! ```rust
//! impl ToSql<Jsonb, Pg> for Foo {
//!     fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Pg>) -> serialize::Result {
//!         out.write_all(&[1])?;
//!         serde_json::to_writer(out, &self)?;
//!         Ok(serialize::IsNull::No)
//!     }
//! }
//!
//! impl FromSql<Jsonb, Pg> for Foo {
//!     fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
//!         let bytes = bytes.as_bytes();
//!         if bytes[0] != 1 {
//!             return Err("Unsupported JSONB encoding version".into());
//!         }
//!         serde_json::from_slice(&bytes[1..]).map_err(|_| "Invalid Json".into())
//!     }
//! }
//!
//! ```
//!
//! This gets tedious quickly so this create does it for you. So with this crate you can write:
//!
//! ```rust
//! use diesel::sql_types::Jsonb;
//! use diesel::{FromSqlRow, AsExpression};
//! use diesel_json_derive::DieselJsonb;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Serialize, Deserialize, AsExpression, FromSqlRow, DieselJsonb)]
//! #[diesel(sql_type = Jsonb)]
//! struct Bar {
//!     x: i32,
//! }
//! ```
//!
//! ### diesel_json_derive vs diesel_json
//!
//! The [diesel_json](https://crates.io/crates/diesel_json) crate solves the
//! same problem but uses a wrapper type for it. This has the disadvantage that
//! this type needs to be used when matching for example. This crate does not
//! have this disadvantage.

use heck::ToSnakeCase;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

#[proc_macro_derive(DieselJsonb)]
pub fn diesel_jsonb_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let type_name = input.ident;
    let generics = input.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let mod_name = format!("{}_diesel_jsonb", type_name.to_string().to_snake_case());
    let mod_name = Ident::new(&mod_name, type_name.span());

    (quote! {
        mod #mod_name {
            use super::#type_name;
            use diesel::deserialize::{self, FromSql};
            use diesel::pg::{Pg, PgValue};
            use diesel::serialize::{self, ToSql};
            use diesel::sql_types::*;
            use std::io::Write;

            impl #impl_generics ToSql<Jsonb, Pg> for #type_name #ty_generics #where_clause {
                fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Pg>) -> serialize::Result {
                    out.write_all(&[1])?;
                    serde_json::to_writer(out, &self)?;
                    Ok(serialize::IsNull::No)
                }
            }

            impl #impl_generics FromSql<Jsonb, Pg> for #type_name #ty_generics #where_clause {
                fn from_sql(value: PgValue<'_>) -> deserialize::Result<Self> {
                    let bytes = value.as_bytes();
                    if bytes[0] != 1 {
                        return Err("Unsupported JSONB encoding version".into());
                    }

                    serde_json::from_slice(&bytes[1..]).map_err(|_| "Invalid Json".into())
                }
            }
        }
    }).into()
}
