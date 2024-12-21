mod images;
mod size;

#[macro_use] extern crate rocket;

use std::fmt::format;
use std::future::Future;
use std::num::ParseIntError;
use std::panic;
use std::panic::AssertUnwindSafe;
use rocket::{get, routes, Request};
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;
use rocket::serde::{Deserialize, Serialize};
use rocket_dyn_templates::{context, Template};
use rustemon::client::RustemonClient;
use rustemon::error::Error;
use rustemon::model::pokemon::Pokemon;
use crate::images::get_file;

#[derive(Serialize, Deserialize, Debug)]
struct Mon {
    name: String,
    image: String,
    size: f32
}

#[get("/")]
async fn index(cookies: &CookieJar<'_>) -> Template {
    let rustemon_client = rustemon::client::RustemonClient::default();
    Template::render("index", context! {
        mon_names: get_species(&rustemon_client).await,
        mons: get_mons(cookies, &rustemon_client).await
    })
}

async fn get_species(rustemon_client: &RustemonClient) -> Vec<String> {
    let species = rustemon::pokemon::pokemon::get_all_entries(&rustemon_client)
        .await
        .unwrap();
    species.into_iter().map(|species| convert_to_title_case(species.name)).collect()
}

async fn get_mons(cookies: &CookieJar<'_>, rustemon_client: &RustemonClient) -> Vec<Mon> {
    let mut pokemon: Vec<Mon> = Vec::new();
    let cookie = match cookies.get("mons") {
        Some(cookie) => {
            let mut parts: Vec<&str> = cookie.value().split(";").collect();
            for part in parts {
                if part == "" {
                    continue;
                }
                if part == "trainer" {
                    pokemon.push(Mon {
                        name: String::from("Trainer"),
                        image: get_file("trainer", &rustemon_client).await,
                        size: 18f32
                    });
                    continue;
                }
                match rustemon::pokemon::pokemon::get_by_name(part, &rustemon_client).await {
                    Ok(mon) => {
                        pokemon.push(Mon {
                            name: convert_to_title_case(mon.name),
                            image: get_file(part, &rustemon_client).await,
                            size: size::get_size(part, &rustemon_client).await,
                        })
                    }
                    Err(_) => {
                        pokemon.push(Mon {
                            name: String::from("MissingNo"),
                            image: get_file("missing-no", &rustemon_client).await,
                            size: size::get_size("missing-no", &rustemon_client).await,
                        })
                    }
                };
            }
        }
        _ => {
            pokemon.push(Mon {
                name: String::from("Trainer"),
                image: get_file("trainer", &rustemon_client).await,
                size: 18f32
            });
            cookies.add(Cookie::new("mons", "trainer"))
        }
    };
    pokemon.sort_by_key(|p| p.size as i16);
    pokemon
}


#[post("/add", data = "<mon>")]
async fn add_mon(cookies: &CookieJar<'_>, mon: Form<&str>) -> Redirect {
    let rustemon_client = rustemon::client::RustemonClient::default();
    let mon_str = mon.to_string().replace(" ", "-").to_lowercase();
    match rustemon::pokemon::pokemon::get_by_name(&mon_str, &rustemon_client).await {
        Ok(_) => {}
        Err(_) => {
            if &mon_str != "trainer" && &mon_str != "missing-no" {
                return Redirect::to(String::from("invalid/") + convert_to_title_case(mon.to_string()).as_str());
            }
        }
    }

    match cookies.get("mons") {
        Some(mons) => {
            if(mons.value().to_string() != "") {
                cookies.add(Cookie::new("mons", mons.value().to_string() + ";"+ mon_str.as_str()));
            } else {
                cookies.add(Cookie::new("mons", mon_str));
            }
        }
        None => cookies.add(Cookie::new("mons", mon_str))
    }
    Redirect::to(uri!(index))
}

#[post("/remove", data = "<mon>")]
fn remove_mon(cookies: &CookieJar<'_>, mon: Form<&str>) -> Redirect {
    match cookies.get("mons") {
        Some(cookie) => {
            let mut parts: Vec<&str> = cookie.value().split(";").collect();
            parts.retain(|&m| m != mon.to_string().to_lowercase().replace(" ", "-"));
            let mut mons: String = parts.join(";");
            cookies.add(Cookie::new("mons", mons));
        }
        None => return Redirect::to(uri!(index)),
    }
    Redirect::to(uri!(index))
}

#[post("/reset")]
fn reset_mons(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove("mons");
    Redirect::to(uri!(index))
}

#[catch(404)]
fn not_found(req: &Request) -> String {
    format!("Sorry, '{}' is not a valid path.", req.uri())
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, add_mon, remove_mon, reset_mons])
        .mount("/static", rocket::fs::FileServer::from("static"))
        .register("/", catchers![not_found])
        .attach(Template::fairing())
}

fn convert_to_title_case(input: String) -> String {
    input
        .split('-')  // Split the string by dashes
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first_char) => first_char.to_uppercase().collect::<String>() + chars.as_str(),  // Capitalize the first character and append the rest
                None => String::new(),
            }
        })
        .collect::<Vec<String>>()  // Collect the words into a vector
        .join(" ")  // Join the words with spaces
}
