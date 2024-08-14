// @generated automatically by Diesel CLI.

diesel::table! {
    user_info (id) {
        id -> Nullable<Unsigned<Integer>>,
        #[max_length = 50]
        username -> Varchar,
        #[max_length = 50]
        header_url -> Varchar,
        ch3 -> Integer,
        ch4 -> Integer,
        ch5 -> Integer,
        ch6 -> Integer,
        ch8 -> Integer,
    }
}