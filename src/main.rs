use clap::{arg, command, ArgMatches, Command};
use reqwest::{Client, ClientBuilder};
use sqlx::{migrate, Pool, Postgres};

mod poke;
mod sprite;
use poke::{DbPoke, Pokemon};

// TODO : multi catch command for async multi queries (provide a file with pokemon name and query them all)

#[tokio::main]
async fn main() {

    // Setup reqwest client for API queries
    let client = ClientBuilder::new()
        .connect_timeout(std::time::Duration::from_secs(10))
        .build().unwrap_or_default();


    // TODO : ENV file
    // docker run -e POSTGRES_PASSWORD=mysecretpassword -e POSTGRES_USER=dbuser -e POSTGRES_DB=pokestore  -p 5432:5432 postgres
    let db_url = "postgres://dbuser:mysecretpassword@localhost:5432/pokestore";
    let db_pool = sqlx::postgres::PgPool::connect(&db_url).await.unwrap();
    migrate!("./migrations").run(&db_pool).await.unwrap();

    // Setup clap CLI commands
    let cli_result: ArgMatches = command!()
    .subcommand_required(true)
    .arg_required_else_help(true)
    .subcommand(
        Command::new("catch")
        .about("Catch a pokemon")
        .arg(
        arg!(<POKE> "pokemon name").required(true)
    ))
    .subcommand(
        Command::new("info")
        .about("Get info on a pokemon")
        .arg(
        arg!(<POKE> "pokemon name").required(true)
    ))
    .subcommand(
        Command::new("shiny")
        .about("Catch a shiny version")
        .arg(
        arg!(<POKE> "pokemon name").required(true)
    ))
    .subcommand(
        Command::new("collection")
        .about("Show your pokemon collection")
        .arg(
            arg!(<GEN> "specify gen").required(false)
    ))
    .get_matches();

    match cli_result.subcommand() {
        Some(("catch", sub_matches)) => catch_pokemon(client, sub_matches.get_one::<String>("POKE").unwrap(), &db_pool).await,
        Some(("info", sub_matches)) => println!("Ca veut des infos sur le {:?}", sub_matches.get_one::<String>("POKE")),
        Some(("shiny", sub_matches)) => println!("Ca veut tenter le shiny sur le {:?}", sub_matches.get_one::<String>("POKE")),
        Some(("collection", sub_matches)) => println!("Ca veut voir la collection {:?}", sub_matches.get_one::<String>("GEN")),
        _ => unreachable!(),
    }
}

async fn catch_pokemon(client: Client, name: &String, db_co: &Pool<Postgres>) {
    let rep = client.get(format!("{}{}", "https://pokeapi.co/api/v2/pokemon/", name)).send().await.expect("Request failed during get");
    if rep.status().is_success() {
        let res: Result<Pokemon, reqwest::Error> = rep.json().await;
        let poke = res.expect("Error while parsing json");
        println!("{}",poke);

        // DB insertion
        // Transform into json stats and types directly in query
        let db_insert = "INSERT INTO poke (poke_id, poke_name, poke_type, poke_base_experience, poke_stats) VALUES ($1, $2, $3::json, $4, $5::json)";
        // Optionnal DbPoke into
        let db_poke: DbPoke = poke.into();
        let stats_json = serde_json::to_string(&db_poke.stats).unwrap();
        let types_json = serde_json::to_string(&db_poke.types).unwrap();
        sqlx::query(&db_insert).bind(i64::from(db_poke.id)).bind(db_poke.name).bind(types_json).bind(i64::from(db_poke.base_experience)).bind(stats_json).execute(db_co).await.unwrap();
    }
    else {
        println!("Err : {}", rep.status());
    }
}

// No async for the following => Retrieved from database (TODO)
fn info_pokemon(name: String, db_co: &Pool<Postgres>) {
    
}

fn shiny_pokemon(name: String, db_co: &Pool<Postgres>) {
    
}

fn collection_pokemon(gen: Option<u8>, db_co: &Pool<Postgres>) {
    
}
