// @generated automatically by Diesel CLI.

diesel::table! {
    work_data (id) {
        id -> Text,
        external_id -> Text,
        file_name -> Text,
        base_dir -> Text,
        try_count -> Int4,
        created -> Timestamp,
        updated -> Timestamp,
    }
}
