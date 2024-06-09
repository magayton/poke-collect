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

const GEN1: std::ops::Range<i32> = 1..151;
const GEN2: std::ops::Range<i32> = 152..251;
const GEN3: std::ops::Range<i32> = 252..386;
const GEN4: std::ops::Range<i32> = 387..493;
const GEN5: std::ops::Range<i32> = 494..649;
const GEN6: std::ops::Range<i32> = 650..721;
const GEN7: std::ops::Range<i32> = 722..809;
const GEN8: std::ops::Range<i32> = 810..905;
const GEN9: std::ops::Range<i32> = 906..1025;

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
                .arg(
                    arg!(<GEN> "specify gen")
                        .required(false)
                        .value_parser(parse_generation),
                ),
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
        Some(("collection", sub_matches)) => {
            collection_pokemon(sub_matches.get_one::<usize>("GEN"), &db_pool).await
        }
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
        sqlx::query(db_insert)
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
    let row = sqlx::query(db_select)
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
        types,
        base_experience: row.try_get("poke_base_experience").unwrap(),
        stats,
        is_shiny: row.try_get("poke_is_shiny").unwrap(),
    };

    println!("{}", pokemon);
}

async fn shiny_pokemon(name: &String, difficulty: usize, number: usize, db_co: &Pool<Postgres>) {
    let db_select = "SELECT * FROM poke WHERE poke_name=$1";
    if sqlx::query(db_select)
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
        sqlx::query(db_upadte)
            .bind(name)
            .execute(db_co)
            .await
            .unwrap();
    } else {
        println!("error la team");
    }
}

async fn collection_pokemon(gen: Option<&usize>, db_co: &Pool<Postgres>) {
    if let Some(gen) = gen {
        let start;
        let end = match gen {
            1 => {
                start = GEN1.start;
                GEN1.end
            }
            2 => {
                start = GEN2.start;
                GEN2.end
            }
            3 => {
                start = GEN3.start;
                GEN3.end
            }
            4 => {
                start = GEN4.start;
                GEN4.end
            }
            5 => {
                start = GEN5.start;
                GEN5.end
            }
            6 => {
                start = GEN6.start;
                GEN6.end
            }
            7 => {
                start = GEN7.start;
                GEN7.end
            }
            8 => {
                start = GEN8.start;
                GEN8.end
            }
            9 => {
                start = GEN9.start;
                GEN9.end
            }
            _ => unreachable!(),
        };

        let db_select = "SELECT poke_name FROM poke WHERE poke_id BETWEEN $1 AND $2";
        let rows = sqlx::query(db_select)
            .bind(start)
            .bind(end)
            .fetch_all(db_co)
            .await
            .unwrap();
        rows.iter()
            .for_each(|row| println!("{}", row.try_get::<String, &str>("poke_name").unwrap()));
    } else {
        let db_select = "SELECT * FROM poke";
        let rows = sqlx::query(db_select).fetch_all(db_co).await.unwrap();

        rows.iter()
            .for_each(|row| println!("{}", row.try_get::<String, &str>("poke_name").unwrap()));
    }
}

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


// Shiny imitate "blockchain mining" => Leading 0 (or other number) to find in a hash
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


// CLI parsers 

fn parse_difficulty_and_number(input: &str) -> Result<usize, String> {
    let num: usize = input.parse().unwrap();
    if num > 0 && num < 10 {
        Ok(num)
    } else {
        Err("Difficulty and number must be between 1 and 9".to_string())
    }
}

// Different because generation can update
fn parse_generation(input: &str) -> Result<usize, String> {
    let num: usize = input.parse().unwrap();
    if num > 0 && num <= 9 {
        Ok(num)
    } else {
        Err("Generation must be between 1 and 9".to_string())
    }
}
