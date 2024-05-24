use reqwest::ClientBuilder;
use serde::Deserialize;


#[derive(Deserialize, Debug)]
struct TypeInfo {
    name: String,
    url: String,
}


#[derive(Deserialize, Debug)]
struct PokemonType {
    slot: u8,
    #[serde(rename = "type")]
    type_info: TypeInfo,
}

#[derive(Deserialize, Debug)]
struct Pokemon {
    name: String,
    height: u32,
    weight: u32,
    base_experience: u32,
    types: Vec<PokemonType>,
}

#[tokio::main]
async fn main() {
    let client = ClientBuilder::new()
        .connect_timeout(std::time::Duration::from_secs(10))
        .build().unwrap_or_default();

    let rep = client.get(String::from("https://pokeapi.co/api/v2/pokemon/ditto")).send().await.expect("Request failed during get");
    if rep.status().is_success() {
        let poke: Result<Pokemon, reqwest::Error> = rep.json().await;
        println!("{:?}",poke.unwrap());
    }
    else {
        println!("Err : {}", rep.status());
    }
}
