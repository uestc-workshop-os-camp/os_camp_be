// @generated automatically by Diesel CLI.

diesel::table! {
    user_info (id) {
        id -> Integer,
        #[max_length = 50]
        username -> Varchar,
        #[max_length = 50]
        header_url -> Varchar,
        ch3 -> Nullable<Integer>,
        ch4 -> Nullable<Integer>,
        ch5 -> Nullable<Integer>,
        ch6 -> Nullable<Integer>,
        ch8 -> Nullable<Integer>,
    }
}
