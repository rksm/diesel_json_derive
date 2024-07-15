// @generated automatically by Diesel CLI.

diesel::table! {
    foo (id) {
        id -> Text,
        bar -> Jsonb,
        bar_with_cowed_lifetimes -> Jsonb,
    }
}
