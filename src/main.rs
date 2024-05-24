use reqwest::{Client, ClientBuilder};

#[tokio::main]
async fn main() {
    let client = ClientBuilder::new()
        .connect_timeout(std::time::Duration::from_secs(10))
        .build().unwrap_or_default();

    let result = client.get(String::from("https://pokeapi.co/api/v2/pokemon/ditto")).build()
        .send().await;

    println!("{:?}", result);
}
