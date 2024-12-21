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

#[get("/invalid/<name>")]
async fn index_invalid(cookies: &CookieJar<'_>, name: &str) -> Template {
    let rustemon_client = rustemon::client::RustemonClient::default();
    Template::render("index", context! {
        mon_names: get_species(&rustemon_client).await,
        mons: get_mons(cookies, &rustemon_client).await,
        invalid: name
    })
}
async fn get_species(rustemon_client: &RustemonClient) -> Vec<String> {
    let species = rustemon::pokemon::pokemon::get_all_entries(&rustemon_client)
        .await
        .unwrap();
    species.into_iter().map(|species| species.name).collect()
}

async fn get_mons(cookies: &CookieJar<'_>, rustemon_client: &RustemonClient) -> Vec<Mon> {
    let mut pokemon: Vec<Mon> = Vec::new();
    pokemon.push(Mon {
        name: String::from("Trainer"),
        image: get_file("Trainer", &rustemon_client).await,
        size: 18f32
    });
    let cookie = match cookies.get("mons") {
        Some(cookie) => {
            let mut parts: Vec<&str> = cookie.value().split(";").collect();
            for part in parts {
                match rustemon::pokemon::pokemon::get_by_name(part, &rustemon_client).await {
                    Ok(mon) => {
                        pokemon.push(Mon {
                            name: mon.name,
                            image: get_file(part, &rustemon_client).await,
                            size: size::get_size(part, &rustemon_client).await,
                        })
                    }
                    Err(_) => {
                        pokemon.push(Mon {
                            name: String::from("MissingNo"),
                            image: get_file("MissingNo", &rustemon_client).await,
                            size: size::get_size("MissingNo", &rustemon_client).await,
                        })
                    }
                };
            }
        }
        _ => {}
    };
    pokemon.sort_by_key(|p| p.size as i16);
    let tallest_size: f32 = pokemon[pokemon.len() - 1].size;
    for p in &mut pokemon {
        p.size = (p.size / tallest_size) * 100f32;
    }
    pokemon
}

#[post("/add", data = "<mon>")]
async fn add_mon(cookies: &CookieJar<'_>, mon: Form<&str>) -> Redirect {
    let rustemon_client = rustemon::client::RustemonClient::default();
    match rustemon::pokemon::pokemon::get_by_name(&mon.to_string(), &rustemon_client).await {
        Ok(_) => {}
        Err(_) => { return Redirect::to(String::from("invalid/") + mon.to_string().as_str()); }
    }

    match cookies.get("mons") {
        Some(mons) => {
            println!("{}", mons);
            cookies.add(Cookie::new("mons", mons.value().to_string() + ";"+ &mon.to_string()));
        }
        None => { cookies.add(Cookie::new("mons", mon.to_string())); },
    }
    Redirect::to(uri!(index))
}

#[post("/remove", data = "<mon>")]
fn remove_mon(cookies: &CookieJar<'_>, mon: Form<&str>) -> Redirect {
    match cookies.get("mons") {
        Some(cookie) => {
            let mut parts: Vec<&str> = cookie.value().split(";").collect();
            parts.retain(|&m| m != mon.to_string());
            let mut mons: String = parts.join(";");
            if(mons == "") {
                cookies.remove("mons");
            } else{
                cookies.add(Cookie::new("mons", mons));
            }
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
    rocket::build().mount("/", routes![index, add_mon, remove_mon, reset_mons, index_invalid])
        .mount("/static", rocket::fs::FileServer::from("static"))
        .attach(Template::fairing())
}

