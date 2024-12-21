mod images;
mod size;

#[macro_use] extern crate rocket;

use std::fmt::format;
use std::future::Future;
use std::num::ParseIntError;
use std::panic;
use std::panic::AssertUnwindSafe;
use rocket::{get, routes};
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;
use rocket::serde::{Deserialize, Serialize};
use rocket_dyn_templates::{context, Template};
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
    let species = rustemon::pokemon::pokemon::get_all_entries(&rustemon_client)
        .await
        .unwrap();
    let species_names: Vec<String> = species.into_iter().map(|species| species.name).collect();

    let mut pokemon: Vec<Mon> = Vec::new();
    pokemon.push(Mon {
        name: String::from("Trainer"),
        image: get_file("Trainer", &rustemon_client),
        size: 18f32
    });
    match cookies.get("mons") {
        Some(cookie) => {
            let mut parts: Vec<&str> = cookie.value().split(";").collect();
            for part in parts {
                match rustemon::pokemon::pokemon::get_by_name(part, &rustemon_client).await {
                    Ok(mon) => {
                        pokemon.push(Mon {
                            name: mon.name,
                            image: images::get_file(part, &rustemon_client).await,
                            size: mon.height as f32,
                        })
                    }
                    Err(_) => {
                        pokemon.push(Mon {
                        name: String::from("MissingNo"),
                        image: images::get_file("MissingNo", &rustemon_client).await,
                        size: 33f32,
                        })
                    }
                }
            }
        }
        _ => {}
    }
    pokemon.sort_by_key(|p| p.size as i16);
    let tallest_size: f32 = pokemon[pokemon.len()-1].size;
    for p in &mut pokemon {
        p.size = (p.size / tallest_size) * 100f32;
    }
    Template::render("index", context! {mon_names: species_names, mons: pokemon})
}

#[post("/add", data = "<mon>")]
fn add_mon(cookies: &CookieJar<'_>, mon: Form<&str>) -> Redirect {

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
#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, add_mon, remove_mon])
        .mount("/static", rocket::fs::FileServer::from("static"))
        .attach(Template::fairing())
}

