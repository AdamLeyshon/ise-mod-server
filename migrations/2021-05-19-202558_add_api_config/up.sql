create table api_config
(
    version serial not null primary key,
    is_online boolean default false not null,
    is_maintenance boolean default false not null,
    maintenance_start int default 0 not null,
    maintenance_duration int default 3600 not null
);

INSERT INTO public.api_config (version, is_online, is_maintenance, maintenance_start, maintenance_duration) VALUES (1, false, false, 0, 0);
