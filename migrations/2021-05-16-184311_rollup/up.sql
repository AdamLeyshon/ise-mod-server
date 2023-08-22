create table accounts
(
    account_id serial
        constraint accounts_pk
            primary key,
    date_added timestamp          not null,
    username   text,
    password   text,
    e_mail     text,
    mfa_code   text,
    active     bool default false not null,
    steam_id   text
);

create table blocked_steam_accounts
(
    steam_id   text      not null
        constraint blocked_steam_accounts_pk
            primary key,
    reason     int       not null,
    date_added timestamp not null
);

create unique index blocked_steam_accounts_steam_id_uindex
    on blocked_steam_accounts (steam_id);

create table account_binds
(
    bind_id     uuid
        constraint account_binds_pk
            primary key,
    steam_id    text,
    confirmed   bool      not null,
    date_added  timestamp not null,
    date_expire timestamp not null,
    account_fk  int,
    constraint account_binds_accounts_account_id_fk
        foreign key (account_fk) references accounts
);

create index account_binds_account_fk_index
    on account_binds (account_fk);

create unique index accounts_steam_id_uindex
    on accounts (steam_id);

create table client_binds
(
    client_bind_id uuid
        constraint client_binds_pk
            primary key,
    account_fk     int                not null
        constraint client_binds_accounts_id_fk
            references accounts,
    confirmed      bool default false not null,
    date_added     timestamp          not null
);

create index client_binds_account_fk_index
    on client_binds (account_fk);

create table colonies
(
    colony_id      uuid
        constraint colonies_pk
            primary key,
    name           text        not null,
    faction_name   text        not null,
    map_id         int         not null,
    tick           int         not null,
    used_dev_mode  bool                 default false not null,
    game_version   text        not null,
    platform       int                  default 0 not null,
    create_date    timestamp   not null,
    client_bind_fk uuid        not null
        constraint colonies_client_binds_client_bind_id_fk
            references client_binds,
    update_date    timestamp   not null default '1970-01-01T00:00:00Z',
    seed           varchar(32) not null default 'NOTSET',
    location       varchar(32) not null default '0,0'
);

create index colonies_client_bind_fk_index
    on colonies (client_bind_fk);

create table bank_balances
(
    colony_id uuid    not null,
    currency  integer not null,
    balance   integer not null,
    constraint bank_balances_pk
        primary key (colony_id, currency)
);

create index bank_balances_colony_id_index
    on bank_balances (colony_id);

create table colony_mods
(
    colony_id uuid  not null,
    mods      jsonb not null,
    constraint colony_mods_pk
        primary key (colony_id)
);

create table inventory
(
    "item_code"  varchar(32)          not null, -- Can't be Uuid, It's a hash of multiple fields.
    "thing_def"  text                 not null,
    "quality"    int,
    "quantity"   int    default 0     not null,
    "minified"   bool   default false not null,
    "base_value" float4 default 0.0   not null,
    "buy_at"     float4 default 0.0   not null,
    "sell_at"    float4 default 0.0   not null,
    "stuff"      text,
    "weight"     float4 default 0.0   not null,
    "version"    varchar(32)          not null,
    constraint inventory_pk
        primary key (item_code)
);

create table orders
(
    order_id    uuid              not null
        constraint orders_pk
            primary key,
    colony_id   uuid              not null,
    manifest    jsonb             not null,
    status      integer default 0 not null,
    start_tick  integer default 0 not null,
    end_tick    integer default 0 not null,
    order_stats jsonb             not null,
    create_date timestamp         not null,
    update_date timestamp         not null
);

create index orders_colony_id_status_index
    on orders (colony_id, status);

create unique index orders_order_id_uindex
    on orders (order_id);

CREATE TABLE colony_tradables
(
    colony_id   uuid      not null,
    tradables   jsonb     not null,
    update_date timestamp not null, -- We track this so we purge old tradables after 24 hours
    constraint colony_id_pk
        primary key ("colony_id")
);

create table price_tracker
(
    item_code   varchar(32)      not null,
    value       real default 0.0 not null,
    create_date timestamp        not null,
    constraint price_tracker_pk
        primary key (item_code, value)
);

create table inventory_promises
(
    "colony_id"   uuid        not null,
    "promise_id"  uuid        not null,
    "private_key" varchar(32) not null,
    "expiry_date" timestamp   not null,

    constraint inventory_promises_pk
        primary key ("colony_id")
);

create unique index inventory_promises_colony_promise_uindex
    on inventory_promises ("colony_id", "promise_id");

INSERT INTO public.accounts (account_id, date_added, username, password, e_mail, mfa_code, active, steam_id)
VALUES (1, '2021-05-17 12:15:02.678296', 'UnitTest', null, null, null, true, '[U:1:13457876]');
INSERT INTO public.account_binds (bind_id, steam_id, confirmed, date_added, date_expire, account_fk)
VALUES ('44960fcc-54b9-4abb-b7ec-2ecaca5bba80', '[U:1:13457876]', true, '2021-05-17 12:14:45.416413',
        '2021-05-17 12:19:45.416413', 1);
INSERT INTO public.client_binds (client_bind_id, account_fk, confirmed, date_added)
VALUES ('939eddf8-0a5f-42d9-b6b3-c2a8b9ebfc7d', 1, true, '2021-05-17 12:15:02.682794');
INSERT INTO public.colonies (colony_id, name, faction_name, map_id, tick, used_dev_mode, game_version, platform,
                             create_date, client_bind_fk, update_date, seed, location)
VALUES ('e946511c-ab8f-4d3d-8ec4-6408ffe4ad1c', 'UnitTest', 'UnitTestFaction', 100, 2222, false, '1.0.0', 1,
        '2021-05-17 12:15:28.221789', '939eddf8-0a5f-42d9-b6b3-c2a8b9ebfc7d', '2021-05-17 12:15:28.221789', '', '');
