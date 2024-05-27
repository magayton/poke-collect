use reqwest::ClientBuilder;

mod poke;
mod sprite;
use poke::Pokemon;

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
