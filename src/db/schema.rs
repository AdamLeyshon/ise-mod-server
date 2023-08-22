table! {
    account_binds (bind_id) {
        bind_id -> Uuid,
        steam_id -> Nullable<Text>,
        confirmed -> Bool,
        date_added -> Timestamp,
        date_expire -> Timestamp,
        account_fk -> Nullable<Int4>,
    }
}

table! {
    accounts (account_id) {
        account_id -> Int4,
        date_added -> Timestamp,
        username -> Nullable<Text>,
        password -> Nullable<Text>,
        e_mail -> Nullable<Text>,
        mfa_code -> Nullable<Text>,
        active -> Bool,
        steam_id -> Nullable<Text>,
    }
}

table! {
    api_config (version) {
        version -> Int4,
        config_data -> Jsonb,
    }
}

table! {
    bank_balances (colony_id, currency) {
        colony_id -> Uuid,
        currency -> Int4,
        balance -> Int4,
    }
}

table! {
    blocked_steam_accounts (steam_id) {
        steam_id -> Text,
        reason -> Int4,
        date_added -> Timestamp,
    }
}

table! {
    client_binds (client_bind_id) {
        client_bind_id -> Uuid,
        account_fk -> Int4,
        confirmed -> Bool,
        date_added -> Timestamp,
    }
}

table! {
    colonies (colony_id) {
        colony_id -> Uuid,
        name -> Text,
        faction_name -> Text,
        map_id -> Int4,
        tick -> Int4,
        used_dev_mode -> Bool,
        game_version -> Text,
        platform -> Int4,
        create_date -> Timestamp,
        client_bind_fk -> Uuid,
        update_date -> Timestamp,
        seed -> Varchar,
        location -> Varchar,
    }
}

table! {
    colony_mods (colony_id) {
        colony_id -> Uuid,
        mods -> Jsonb,
    }
}

table! {
    colony_tradables (colony_id) {
        colony_id -> Uuid,
        tradables -> Jsonb,
        update_date -> Timestamp,
    }
}

table! {
    inventory (item_code) {
        item_code -> Varchar,
        thing_def -> Text,
        quality -> Nullable<Int4>,
        quantity -> Int4,
        minified -> Bool,
        base_value -> Numeric,
        buy_at -> Numeric,
        sell_at -> Numeric,
        stuff -> Nullable<Text>,
        weight -> Numeric,
        version -> Varchar,
    }
}

table! {
    inventory_promises (colony_id) {
        colony_id -> Uuid,
        promise_id -> Uuid,
        private_key -> Varchar,
        expiry_date -> Timestamp,
        activated -> Bool,
    }
}

table! {
    maintenance (checksum) {
        checksum -> Varchar,
        in_progress -> Bool,
        start_time -> Nullable<Timestamp>,
        execution_time -> Nullable<Timestamp>,
        node_name -> Nullable<Varchar>,
    }
}

table! {
    new_inventory (version) {
        item_code -> Varchar,
        thing_def -> Varchar,
        quality -> Nullable<Int4>,
        minified -> Bool,
        base_value -> Numeric,
        stuff -> Nullable<Varchar>,
        weight -> Numeric,
        version -> Varchar,
        date_added -> Timestamp,
    }
}

table! {
    colony_inventory_staging (colony_id, version) {
        colony_id -> Uuid,
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

table! {
    new_inventory_vote_tracker (version, client_bind_id, colony_id) {
        client_bind_id -> Uuid,
        version -> Varchar,
        colony_id -> Uuid,
    }
}

table! {
    orders (order_id) {
        order_id -> Uuid,
        colony_id -> Uuid,
        manifest -> Jsonb,
        status -> Int4,
        start_tick -> Int4,
        end_tick -> Int4,
        order_stats -> Jsonb,
        create_date -> Timestamp,
        update_date -> Timestamp,
    }
}

table! {
    price_tracker (item_code, value) {
        item_code -> Varchar,
        value -> Numeric,
        create_date -> Timestamp,
    }
}

table! {
    stock_config (version) {
        version -> Int4,
        config_data -> Jsonb,
    }
}

table! {
    trade_statistics (item_code, buy, date) {
        item_code -> Varchar,
        buy -> Bool,
        quantity -> Int8,
        date -> Date,
    }
}

table! {
    trade_statistics_monthly (item_code, buy, date) {
        item_code -> Varchar,
        buy -> Bool,
        quantity -> Int8,
        date -> Date,
    }
}

joinable!(account_binds -> accounts (account_fk));
joinable!(client_binds -> accounts (account_fk));
joinable!(colonies -> client_binds (client_bind_fk));
joinable!(new_inventory_vote_tracker -> client_binds (client_bind_id));
joinable!(new_inventory_vote_tracker -> new_inventory (version));

allow_tables_to_appear_in_same_query!(
    account_binds,
    accounts,
    api_config,
    bank_balances,
    blocked_steam_accounts,
    client_binds,
    colonies,
    colony_mods,
    colony_tradables,
    inventory,
    inventory_promises,
    maintenance,
    new_inventory,
    new_inventory_vote_tracker,
    orders,
    price_tracker,
    stock_config,
    trade_statistics,
    trade_statistics_monthly,
);
