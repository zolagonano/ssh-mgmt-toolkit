// @generated automatically by Diesel CLI.

diesel::table! {
    logins (id) {
        id -> Int4,
        username -> Text,
        password_hash -> Text,
        admin -> Bool,
        register_date -> Timestamp,
    }
}

diesel::table! {
    nodes (id) {
        id -> Int4,
        address -> Text,
        token -> Text,
        status -> Int4,
    }
}

diesel::table! {
    sells (id) {
        id -> Int4,
        user_id -> Int8,
        ref_id -> Nullable<Int8>,
        service_id -> Int4,
        node_id -> Int4,
        firstbuy_date -> Nullable<Timestamp>,
        invoice_date -> Nullable<Timestamp>,
        username -> Nullable<Text>,
        password -> Nullable<Text>,
        password_hash -> Nullable<Text>,
        status -> Int4,
    }
}

diesel::table! {
    services (id) {
        id -> Int4,
        max_logins -> Int4,
        max_traffic -> Nullable<Int4>,
        price -> Int4,
        available -> Bool,
    }
}

diesel::table! {
    users (id) {
        id -> Int8,
        ref_id -> Nullable<Int8>,
        register_date -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    logins,
    nodes,
    sells,
    services,
    users,
);
