use clap::{arg, command, ArgMatches, Command};
use reqwest::{Client, ClientBuilder};

mod poke;
mod sprite;
use poke::Pokemon;

#[tokio::main]
async fn main() {

    // Setup reqwest client for API queries
    let client = ClientBuilder::new()
        .connect_timeout(std::time::Duration::from_secs(10))
        .build().unwrap_or_default();

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
        Some(("catch", sub_matches)) => catch_pokemon(client, sub_matches.get_one::<String>("POKE").unwrap()).await,
        Some(("info", sub_matches)) => println!("Ca veut des infos sur le {:?}", sub_matches.get_one::<String>("POKE")),
        Some(("shiny", sub_matches)) => println!("Ca veut tenter le shiny sur le {:?}", sub_matches.get_one::<String>("POKE")),
        Some(("collection", sub_matches)) => println!("Ca veut voir la collection {:?}", sub_matches.get_one::<String>("GEN")),
        _ => unreachable!(),
    }
}

async fn catch_pokemon(client: Client, name: &String) {
    let rep = client.get(format!("{}{}", "https://pokeapi.co/api/v2/pokemon/", name)).send().await.expect("Request failed during get");
    if rep.status().is_success() {
        let res: Result<Pokemon, reqwest::Error> = rep.json().await;
        let poke = res.expect("Error while parsing json");
        println!("{}",poke);
    }
    else {
        println!("Err : {}", rep.status());
    }
}

// No async for the following => Retrieved from database (TODO)
fn info_pokemon(name: String) {
    
}

fn shiny_pokemon(name: String) {
    
}

fn collection_pokemon(gen: Option<u8>) {
    
}
