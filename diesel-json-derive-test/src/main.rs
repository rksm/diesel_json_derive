mod schema;

use std::borrow::Cow;

use diesel::prelude::*;
use diesel::sql_types::Jsonb;
use diesel::{FromSqlRow, AsExpression};
use diesel_json_derive::DieselJsonb;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Serialize, Deserialize, Queryable, Identifiable, Insertable, AsChangeset, Selectable,
)]
#[diesel(table_name = crate::schema::foo)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(id))]
struct Foo<'a> {
    id: String,
    bar: Bar,
    bar_with_cowed_lifetimes: BarWithCowedLifetimes<'a>,
}

#[derive(Debug, Serialize, Deserialize, AsExpression, FromSqlRow, DieselJsonb)]
#[diesel(sql_type = Jsonb)]
struct Bar {
    x: i32,
}

#[derive(Debug, Serialize, Deserialize, AsExpression, FromSqlRow, DieselJsonb)]
#[diesel(sql_type = Jsonb)]
struct BarWithCowedLifetimes<'a> {
    x: Cow<'a, str>,
}

fn main() {
    unimplemented!()
}
