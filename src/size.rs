use std::path::Path;
use rocket::tokio::fs;
use rustemon::client::RustemonClient;

async fn get_size(name: &str, rustemon_client: &RustemonClient) -> f32 {
    let path: String = "sizes/".to_owned() + name + ".txt";
    // let path = ;
    if !Path::new(&path).exists() {

    }
    let contents = fs::read_to_string(&path).await.unwrap();
    contents
}