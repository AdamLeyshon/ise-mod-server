create table if not exists maintenance
(
    "checksum"       varchar(64) not null primary key,
    "in_progress"    bool        not null default false,
    "start_time"     timestamp,
    "execution_time" timestamp,
    "node_name"      varchar(64)
);
