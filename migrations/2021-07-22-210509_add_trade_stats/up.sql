create table trade_statistics
(
    item_code varchar              not null,
    buy       bool   default false not null,
    quantity  bigint default 0     not null,
    date      date                 not null
);

create index trade_statistics_item_code_uindex
    on trade_statistics (item_code);

create index trade_statistics_date_index
    on trade_statistics (date desc);

alter table trade_statistics
    add constraint trade_statistics_pk
        primary key (item_code, buy, date);

create table trade_statistics_monthly
(
    item_code varchar              not null,
    buy       bool   default false not null,
    quantity  bigint default 0     not null,
    date      date                 not null
);

create index trade_statistics_monthly_item_code_uindex
    on trade_statistics_monthly (item_code);

create index trade_statistics_monthly_date_index
    on trade_statistics_monthly (date desc);

alter table trade_statistics_monthly
    add constraint trade_statistics_monthly_pk
        primary key (item_code, buy, date);

create table if not exists stock_config
(
    version     serial not null primary key,
    config_data jsonb  not null default '{}'
);
