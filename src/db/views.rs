table! {
    summary_inventory_votes (version) {
        item_code -> Varchar,
        thing_def -> Varchar,
        quality -> Nullable<Int4>,
        minified -> Bool,
        base_value -> Numeric,
        stuff -> Nullable<Varchar>,
        weight -> Numeric,
        version -> Varchar,
        votes -> Int8,
    }
}

table! {
    temporary_vote_data (position) {
        position -> Int8,
        item_code -> Varchar,
        thing_def -> Varchar,
        quality -> Nullable<Int4>,
        minified -> Bool,
        base_value -> Numeric,
        stuff -> Nullable<Varchar>,
        weight -> Numeric,
        version -> Varchar,
    }
}
