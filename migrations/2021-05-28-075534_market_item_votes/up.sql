alter table inventory
    add if not exists version varchar(32) not null default '';

create table new_inventory
(
    "item_code"  varchar(32)          not null, -- Can'T be Uuid, It's a hash of multiple fields.
    "thing_def"  varchar(200)         not null,
    "quality"    int,
    "minified"   bool   default false not null,
    "base_value" float4 default 0.0   not null,
    "stuff"      varchar(200),
    "weight"     float4 default 0.0   not null,
    "version"    varchar(32)          not null,
    "date_added" timestamp            not null,
    constraint new_inventory_pk
        primary key (version)
);

create table new_inventory_vote_tracker
(
    "client_bind_id" uuid        not null
        constraint new_inventory_vote_tracker_client_bind_fk
            references client_binds,
    "version"        varchar(32) not null
        constraint new_inventory_vote_hash_fk
            references new_inventory,
    constraint new_inventory_vote_tracker_pk
        primary key (client_bind_id, version)
);

create view summary_inventory_votes as
(
SELECT ni.item_code,
       ni.thing_def,
       ni.quality,
       ni.minified,
       ni.base_value,
       ni.stuff,
       ni.weight,
       ni.version,
       v.votes
FROM new_inventory ni
         LEFT JOIN (
    SELECT nivt.version, count(nivt.version) as votes
    FROM new_inventory_vote_tracker nivt
    GROUP BY nivt.version
) as v on v.version = ni.version
         LEFT JOIN inventory i on ni.version = i.version
WHERE i.version IS NULL
    );

alter table api_config
    drop column if exists is_maintenance;
alter table api_config
    drop column if exists is_online;
alter table api_config
    drop column if exists maintenance_duration;
alter table api_config
    drop column if exists maintenance_start;

alter table api_config add config_data jsonb not null default '{}';
