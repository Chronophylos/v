table! {
    albums (id) {
        id -> Int4,
        token -> Varchar,
        deletion_token -> Varchar,
        title -> Nullable<Varchar>,
    }
}

table! {
    images (id) {
        id -> Int4,
        album_id -> Int4,
        token -> Varchar,
        deletion_token -> Varchar,
        url -> Varchar,
        index -> Int4,
    }
}

joinable!(images -> albums (album_id));

allow_tables_to_appear_in_same_query!(
    albums,
    images,
);
