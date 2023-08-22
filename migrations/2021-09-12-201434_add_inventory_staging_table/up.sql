CREATE TABLE IF NOT EXISTS colony_inventory_staging
(
    colony_id  uuid                         not null,
    item_code  varchar(32)                  not null,
    thing_def  varchar(200)                 not null,
    quality    integer,
    minified   boolean        default false not null,
    base_value numeric(10, 2) default 0.0   not null,
    stuff      varchar(200),
    weight     numeric(10, 2) default 0.0   not null,
    version    varchar(32)                  not null,
    CONSTRAINT colony_inventory_staging_pk PRIMARY KEY (colony_id, version)
);

create index colony_inventory_staging_colony_id_index
    on colony_inventory_staging (colony_id);
