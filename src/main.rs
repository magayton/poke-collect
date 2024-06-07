use std::{
    env,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
};

use clap::{arg, command, Arg, ArgMatches, Command};
use dotenv::dotenv;
use reqwest::{Client, ClientBuilder};
use serde_json::{from_value, Value};
use sha2::{Digest, Sha256};
use sqlx::{migrate, Pool, Postgres, Row};

mod poke;
mod sprite;
use poke::{DbPoke, Pokemon, PokemonType, Stat};
use tokio::task;

#[tokio::main]
async fn main() {
    // Init dotenv
    dotenv().ok();

    // Setup reqwest client for API queries
    let client = ClientBuilder::new()
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_default();

    // Setup postgres DB 
    let db_url = env::var("DB_URL").unwrap();
    let db_pool = sqlx::postgres::PgPool::connect(&db_url).await.unwrap();
    migrate!("./migrations").run(&db_pool).await.unwrap();

    // Setup clap CLI commands
    let cli_result: ArgMatches = command!()
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("catch")
                .about("Catch a pokemon")
                .arg(arg!(<POKE> "pokemon name").required(true)),
        )
        .subcommand(
            Command::new("info")
                .about("Get info on a pokemon")
                .arg(arg!(<POKE> "pokemon name").required(true)),
        )
        .subcommand(
            Command::new("shiny")
                .about("Catch a shiny version")
                .arg(arg!(<POKE> "pokemon name").required(true))
                .arg(
                    arg!(<DIFFICULTY> "Number of leading <NUMBER>")
                        .required(true)
                        .value_parser(parse_difficulty_and_number),
                )
                .arg(
                    arg!(<NUMBER> "Which number you want for the hash")
                        .required(true)
                        .value_parser(parse_difficulty_and_number),
                ),
        )
        .subcommand(
            Command::new("collection")
                .about("Show your pokemon collection")
                .arg(arg!(<GEN> "specify gen").required(false)),
        )
        .subcommand(
            Command::new("multi-catch")
                .about("Catch multiple pokemon")
                .arg(Arg::new("names").num_args(1..).required(true)),
        )
        .get_matches();

    match cli_result.subcommand() {
        Some(("catch", sub_matches)) => {
            catch_pokemon(
                client,
                sub_matches.get_one::<String>("POKE").unwrap(),
                &db_pool,
            )
            .await
        }
        Some(("info", sub_matches)) => {
            info_pokemon(sub_matches.get_one::<String>("POKE").unwrap(), &db_pool).await
        }
        Some(("shiny", sub_matches)) => {
            shiny_pokemon(
                sub_matches.get_one::<String>("POKE").unwrap(),
                *sub_matches.get_one::<usize>("DIFFICULTY").unwrap(),
                *sub_matches.get_one::<usize>("NUMBER").unwrap(),
                &db_pool,
            )
            .await
        }
        Some(("collection", sub_matches)) => println!(
            "Ca veut voir la collection {:?}",
            sub_matches.get_one::<String>("GEN")
        ),
        Some(("multi-catch", sub_matches)) => {
            if let Some(names) = sub_matches.get_many::<String>("names") {
                let names: Vec<String> = names.map(|name| name.to_string()).collect();
                multi_catch_pokemon(client, names, &db_pool).await;
            }
        }
        _ => unreachable!(),
    }
}

async fn catch_pokemon(client: Client, name: &String, db_co: &Pool<Postgres>) {
    let rep = client
        .get(format!("{}{}", "https://pokeapi.co/api/v2/pokemon/", name))
        .send()
        .await
        .expect("Request failed during get");
    if rep.status().is_success() {
        let res: Result<Pokemon, reqwest::Error> = rep.json().await;
        let poke = res.expect("Error while parsing json");
        println!("{}", poke);

        // DB insertion
        // Transform into json stats and types directly in query
        let db_insert = "INSERT INTO poke (poke_id, poke_name, poke_type, poke_base_experience, poke_stats) VALUES ($1, $2, $3::json, $4, $5::json)";
        // Optionnal DbPoke into
        let db_poke: DbPoke = poke.into();
        let stats_json = serde_json::to_string(&db_poke.stats).unwrap();
        let types_json = serde_json::to_string(&db_poke.types).unwrap();
        sqlx::query(&db_insert)
            .bind(db_poke.id)
            .bind(db_poke.name)
            .bind(types_json)
            .bind(db_poke.base_experience)
            .bind(stats_json)
            .execute(db_co)
            .await
            .unwrap();
    } else {
        println!("Err : {}", rep.status());
    }
}

async fn info_pokemon(name: &String, db_co: &Pool<Postgres>) {
    let db_select = "SELECT * FROM poke WHERE poke_name=$1";
    let row = sqlx::query(&db_select)
        .bind(name)
        .fetch_one(db_co)
        .await
        .unwrap();

    let stats_value: Value = row.try_get("poke_stats").unwrap();
    let stats: Vec<Stat> = from_value(stats_value).unwrap();
    let types_value: Value = row.try_get("poke_type").unwrap();
    let types: Vec<PokemonType> = from_value(types_value).unwrap();

    let pokemon = DbPoke {
        id: row.try_get("poke_id").unwrap(),
        name: row.try_get("poke_name").unwrap(),
        types: types,
        base_experience: row.try_get("poke_base_experience").unwrap(),
        stats: stats,
        is_shiny: row.try_get("poke_is_shiny").unwrap(),
    };

    println!("{}", pokemon);
}

async fn shiny_pokemon(name: &String, difficulty: usize, number: usize, db_co: &Pool<Postgres>) {
    let db_select = "SELECT * FROM poke WHERE poke_name=$1";
    if sqlx::query(&db_select)
        .bind(name)
        .fetch_one(db_co)
        .await
        .is_err()
    {
        println!("You need to catch a {} first", name);
        return;
    }

    let (tx_result, rx_result) = mpsc::channel();
    let mut handles = vec![];
    let solution_found = Arc::new(AtomicBool::new(false));

    for i in 0..8 {
        let tx_result = tx_result.clone();
        let solution_found = solution_found.clone();
        let handle = thread::spawn(move || {
            let mut counter = i as u64;
            loop {
                let hash = generate_hash(counter);
                if is_shiny(&hash, &difficulty, number) {
                    solution_found.store(true, Ordering::SeqCst);
                    tx_result.send(counter).unwrap();
                    return;
                }
                counter += 8;

                if solution_found.load(Ordering::SeqCst) {
                    break;
                }
            }
        });
        handles.push(handle);
    }

    drop(tx_result);

    if let Ok(result) = rx_result.recv() {
        println!("J'ai trouvé le shiny avec le : {}", result);
        for handle in handles {
            handle.join().unwrap();
        }
        let db_upadte = "UPDATE poke SET poke_is_shiny=true where poke_name=$1";
        sqlx::query(&db_upadte)
            .bind(name)
            .execute(db_co)
            .await
            .unwrap();
    } else {
        println!("error la team");
    }
}

fn collection_pokemon(gen: Option<u8>, db_co: &Pool<Postgres>) {}

async fn multi_catch_pokemon(client: Client, names: Vec<String>, db_co: &Pool<Postgres>) {
    let poke = names.into_iter().map(|name| {
        let client = client.clone();
        let db_co = db_co.clone();
        task::spawn(async move {
            catch_pokemon(client, &name, &db_co).await;
        })
    });

    for poke in poke {
        poke.await.unwrap();
    }
}

fn generate_hash(input: u64) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.to_string());
    let result = hasher.finalize();
    format!("{:x}", result)
}

fn is_shiny(hash: &str, difficulty: &usize, number: usize) -> bool {
    let number = format!("{}", number);
    hash.starts_with(&number.repeat(*difficulty))
}

fn parse_difficulty_and_number(input: &str) -> Result<usize, String> {
    let num: usize = input.parse().unwrap();
    if num > 0 && num < 10 {
        Ok(num)
    } else {
        Err(format!("Difficulty and number must be between 1 and 9"))
    }
}
