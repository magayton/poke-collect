create table poke (
    poke_id bigint primary key not null,
    poke_name varchar not null,
    poke_type json not null,
    poke_base_experience bigint not null,
    poke_stats json not null
);