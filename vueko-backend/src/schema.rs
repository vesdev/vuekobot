// @generated automatically by Diesel CLI.

diesel::table! {
    ttv_commands (id) {
        id -> Uuid,
        channel -> Varchar,
        command -> Varchar,
        value -> Text,
    }
}
