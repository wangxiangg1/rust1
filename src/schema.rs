diesel::table! {
    users (id) {
        id -> Integer,
        username -> Text,
        password_hash -> Text,
    }
}

diesel::table! {
    credentials (id) {
        id -> Integer,
        email -> Text,
        token -> Text,
    }
}

diesel::table! {
    api_tokens (id) {
        id -> Integer,
        token -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    users,
    credentials,
    api_tokens,
);

