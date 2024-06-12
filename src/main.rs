#![allow(dead_code, unused_variables)]

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
use sqlx::{migrate, postgres::PgRow, Pool, Postgres, Row};

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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Init dotenv
    dotenv().ok();

    // Setup reqwest client for API queries
    let client = ClientBuilder::new()
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap_or_default();

    // Setup postgres DB
    let db_url = env::var("DB_URL")?;

    let db_pool = sqlx::postgres::PgPool::connect(&db_url).await?;

    migrate!("./migrations").run(&db_pool).await?;

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
            .await?;
        }
        Some(("info", sub_matches)) => {
            info_pokemon(sub_matches.get_one::<String>("POKE").unwrap(), &db_pool).await?;
        }
        Some(("shiny", sub_matches)) => {
            shiny_pokemon(
                sub_matches.get_one::<String>("POKE").unwrap(),
                *sub_matches.get_one::<usize>("DIFFICULTY").unwrap(),
                *sub_matches.get_one::<usize>("NUMBER").unwrap(),
                &db_pool,
            )
            .await?;
        }
        Some(("collection", sub_matches)) => {
            collection_pokemon(sub_matches.get_one::<usize>("GEN"), &db_pool).await?;
        }
        Some(("multi-catch", sub_matches)) => {
            if let Some(names) = sub_matches.get_many::<String>("names") {
                let names: Vec<String> = names.map(|name| name.to_string()).collect();
                multi_catch_pokemon(client, names, &db_pool).await?;
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}

async fn catch_pokemon(
    client: Client,
    name: &String,
    db_co: &Pool<Postgres>,
) -> Result<(), Box<dyn std::error::Error>> {
    let rep = client
        .get(format!("{}{}", "https://pokeapi.co/api/v2/pokemon/", name))
        .send()
        .await?;

    if rep.status().is_success() {
        let poke: Pokemon = rep.json().await?;
        println!("{}", poke);

        // DB insertion
        // Transform into json stats and types directly in query
        let db_insert = "INSERT INTO poke (poke_id, poke_name, poke_type, poke_base_experience, poke_stats) VALUES ($1, $2, $3::json, $4, $5::json)";

        // Optionnal DbPoke into (just to use it)
        let db_poke: DbPoke = poke.into();

        let stats_json = serde_json::to_string(&db_poke.stats)?;
        let types_json = serde_json::to_string(&db_poke.types)?;
        sqlx::query(db_insert)
            .bind(db_poke.id)
            .bind(db_poke.name)
            .bind(types_json)
            .bind(db_poke.base_experience)
            .bind(stats_json)
            .execute(db_co)
            .await?;
        Ok(())
    } else {
        // Manual way to handling errors, could use anyhow or thiserror
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("API Response error: {}", rep.status()),
        )))
    }
}

async fn info_pokemon(
    name: &String,
    db_co: &Pool<Postgres>,
) -> Result<DbPoke, Box<dyn std::error::Error>> {
    let db_select = "SELECT * FROM poke WHERE poke_name=$1";
    let row = sqlx::query(db_select).bind(name).fetch_one(db_co).await?;

    let stats_value: Value = row.try_get("poke_stats")?;
    let stats: Vec<Stat> = from_value(stats_value)?;
    let types_value: Value = row.try_get("poke_type")?;
    let types: Vec<PokemonType> = from_value(types_value)?;

    let pokemon = DbPoke {
        id: row.try_get("poke_id")?,
        name: row.try_get("poke_name")?,
        types,
        base_experience: row.try_get("poke_base_experience")?,
        stats,
        is_shiny: row.try_get("poke_is_shiny")?,
    };

    println!("{}", pokemon);

    Ok(pokemon)
}

async fn shiny_pokemon(
    name: &String,
    difficulty: usize,
    number: usize,
    db_co: &Pool<Postgres>,
) -> Result<(), Box<dyn std::error::Error>> {
    let db_select = "SELECT * FROM poke WHERE poke_name=$1";
    let poke = sqlx::query(db_select).bind(name).fetch_one(db_co).await?;

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
                    tx_result
                        .send(counter)
                        .expect("Could not send result for shiny hunt");
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

    let result = rx_result.recv()?;
    println!("Shiny found with : {}", result);
    for handle in handles {
        handle.join().expect("Could not join thread");
    }
    let db_upadte = "UPDATE poke SET poke_is_shiny=true where poke_name=$1";
    sqlx::query(db_upadte).bind(name).execute(db_co).await?;
    Ok(())
}

async fn collection_pokemon(
    gen: Option<&usize>,
    db_co: &Pool<Postgres>,
) -> Result<(), Box<dyn std::error::Error>> {
    let db_select: &str;
    let rows: Vec<PgRow>;

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

        db_select = "SELECT poke_name FROM poke WHERE poke_id BETWEEN $1 AND $2";
        rows = sqlx::query(db_select)
            .bind(start)
            .bind(end)
            .fetch_all(db_co)
            .await?;
    } else {
        db_select = "SELECT * FROM poke";
        rows = sqlx::query(db_select).fetch_all(db_co).await?;
    }

    for row in rows.iter() {
        let name: String = row.try_get("poke_name")?;
        println!("{}", name);
    }

    Ok(())
}

async fn multi_catch_pokemon(
    client: Client,
    names: Vec<String>,
    db_co: &Pool<Postgres>,
) -> Result<(), task::JoinError> {
    let poke = names.into_iter().map(|name| {
        let client = client.clone();
        let db_co = db_co.clone();
        task::spawn(async move {
            let res = catch_pokemon(client, &name, &db_co).await;
            match res {
                Ok(_) => (),
                Err(e) => eprintln!("Error during multi catch on {}", name),
            }
        })
    });

    for poke in poke {
        poke.await?;
    }

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use sqlx::{migrate, PgPool};
    use std::env;

    async fn setup_test_db() -> PgPool {
        // Init dotenv for DB
        dotenv().ok();

        let database_url = env::var("DB_URL_TEST").expect("DB_URL_TEST must be set");
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        migrate!("./migrations").run(&pool).await.unwrap();

        // Empty DB
        sqlx::query("TRUNCATE TABLE poke")
            .execute(&pool)
            .await
            .expect("Failed to truncate table");

        pool
    }

    fn setup_test_reqwest() -> Client {
        ClientBuilder::new()
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default()
    }

    #[tokio::test]
    async fn test_catch_pokemon() {
        let pool = setup_test_db().await;
        let client = setup_test_reqwest();
        let poke_name = String::from("pikachu");
        let res = catch_pokemon(client, &poke_name, &pool).await;
        assert!(res.is_ok());

        // Verify result in DB
        let db_select = "SELECT * FROM poke WHERE poke_name=$1";
        let row = sqlx::query(db_select)
            .bind(&poke_name)
            .fetch_one(&pool)
            .await
            .expect("Could not get the row");

        // Only looking if name match (not extensive test)
        let bdd_poke_name: String = row
            .try_get("poke_name")
            .expect("Could not find poke_name column value");

        assert_eq!(bdd_poke_name, poke_name);
    }
}
