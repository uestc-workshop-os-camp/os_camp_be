// @generated automatically by Diesel CLI.

diesel::table! {
    user_info (id) {
        id -> Unsigned<Integer>,
        #[max_length = 50]
        username -> Varchar,
        #[max_length = 50]
        header_url -> Varchar,
        ch3 -> Nullable<Double>,
        ch4 -> Nullable<Double>,
        ch5 -> Nullable<Double>,
        ch6 -> Nullable<Double>,
        ch8 -> Nullable<Double>,
    }
}
