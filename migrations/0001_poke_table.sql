create table poke (
    poke_id int primary key not null,
    poke_name varchar not null,
    poke_type json not null,
    poke_base_experience int not null,
    poke_stats json not null
);