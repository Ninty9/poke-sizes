use std::fs::File;
use std::io::Write;
use std::path::Path;
use rocket::tokio::fs;
use rustemon::client::RustemonClient;

pub(crate) async fn get_size(name: &str, rustemon_client: &RustemonClient) -> f32 {
    let path: String = "data/sizes/".to_owned() + name;
    // let path = ;
    if !Path::new(&path).exists() {
        let mon = match rustemon::pokemon::pokemon::get_by_name(name, rustemon_client).await {
            Ok(mon) => mon,
            Err(_) => {return 666f32}
        };
        let mut file = match File::create(&path) {
            Ok(file) => file,
            Err(_) => {
                eprintln!("Failed to create file: {}", &path);
                return mon.height as f32
            }
        };
        match file.write_all(mon.height.to_string().as_bytes()) {
            Ok(_) => {}
            Err(_) => {eprintln!("Failed to write file: {}", &path);}
        }
        return mon.height as f32
    }
    let contents = fs::read_to_string(&path).await.unwrap();
    contents.parse::<f32>().unwrap_or(66.6f32)
}