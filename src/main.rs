mod images;
mod size;

#[macro_use] extern crate rocket;

use std::fs;
use std::fs::File;
use std::path::Path;
use rocket::{get, routes, Request};
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;
use rocket::serde::{Deserialize, Serialize};
use rocket::yansi::Paint;
use rocket_dyn_templates::{context, Template};
use rustemon::client::RustemonClient;
use crate::images::get_file;

#[derive(Serialize, Deserialize, Debug)]
struct Mon {
    name: String,
    image: String,
    size: f32,
    index: u8,
}

#[get("/")]
async fn index() -> Template {
    Template::render("index", context!{})
}

#[get("/compare")]
async fn compare(cookies: &CookieJar<'_>) -> Template {
    let rustemon_client = rustemon::client::RustemonClient::default();
    Template::render("compare", context! {
        mon_names: get_species(&rustemon_client).await,
        mons: get_mons(cookies, &rustemon_client).await
    })
}

async fn get_species(rustemon_client: &RustemonClient) -> Vec<String> {
    let species = rustemon::pokemon::pokemon::get_all_entries(rustemon_client)
        .await
        .unwrap();
    species.into_iter().map(|species| convert_to_title_case(species.name)).collect()
}

async fn get_mons(cookies: &CookieJar<'_>, rustemon_client: &RustemonClient) -> Vec<Mon> {
    let mut pokemon: Vec<Mon> = Vec::new();
    match cookies.get("mons") {
        Some(cookie) => {
            let parts: Vec<&str> = cookie.value().split(';').collect();
            for part in parts {
                if part.is_empty() {
                    continue;
                }
                if part == "trainer" {
                    pokemon.push(Mon {
                        name: String::from("Trainer"),
                        image: get_file("trainer", rustemon_client).await,
                        size: 18f32,
                        index: 0,
                    });
                    continue;
                }
                match rustemon::pokemon::pokemon::get_by_name(part, rustemon_client).await {
                    Ok(mon) => {
                        pokemon.push(Mon {
                            name: convert_to_title_case(mon.name),
                            image: get_file(part, rustemon_client).await,
                            size: size::get_size(part, rustemon_client).await,
                            index: 0,
                        })
                    }
                    Err(_) => {
                        pokemon.push(Mon {
                            name: String::from("MissingNo"),
                            image: get_file("missing-no", rustemon_client).await,
                            size: size::get_size("missing-no", rustemon_client).await,
                            index: 0,
                        })
                    }
                };
            }
        }
        _ => {
            pokemon.push(Mon {
                name: String::from("Trainer"),
                image: get_file("trainer", rustemon_client).await,
                size: 18f32,
                index: 0,
            });
            cookies.add(Cookie::new("mons", "trainer"))
        }
    };
    for (index, mon) in pokemon.iter_mut().enumerate() {
        mon.index = index as u8;
    }
    pokemon
}


#[post("/add", data = "<mon>")]
async fn add_mon(cookies: &CookieJar<'_>, mon: Form<&str>) -> Redirect {
    let rustemon_client = rustemon::client::RustemonClient::default();
    let mon_str = mon.to_string().replace(' ', "-").to_lowercase();
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
            if !mons.value().is_empty() {
                cookies.add(Cookie::new("mons", mons.value().to_string() + ";"+ mon_str.as_str()));
            } else {
                cookies.add(Cookie::new("mons", mon_str));
            }
        }
        None => cookies.add(Cookie::new("mons", mon_str))
    }
    Redirect::to(uri!(compare))
}
#[derive(FromForm)]
struct Report {
    mon: String,
    reason: String,
}

#[derive(Serialize, Deserialize)]
struct MonReport {
    name: String,
    small: i16,
    big: i16,
    other: i16,
}

#[derive(Serialize, Deserialize)]
struct ReportArray {
    reports: Vec<MonReport>,
}
#[post("/report", data = "<report>")]
fn report_mon(report: Form<Report>) -> Redirect {
    if !(report.reason == "+" || report.reason == "-" || report.reason == "!") {
        return Redirect::to(uri!(compare))
    }
    println!("{} has issue: {}", report.mon, report.reason);
    let path = "reports.json";
    let json: String;
    let mut a: ReportArray = ReportArray { reports: Vec::new() };
    if !Path::new(&path).exists() {
        match File::create(&path) {
            Ok(_) => { json = "".to_owned() }
            Err(e) => { error!("Error creating new file: {}", e); return Redirect::to(uri!(compare)); }
        }
    } else {
        json = match fs::read_to_string(path) {
            Ok(s) => {s}
            Err(e) => {error!("Error reading file: {}", e); return Redirect::to(uri!(compare));}
        }
    }
    
    if !json.is_empty() {
        a = match serde_json::from_str(&json) {
            Ok(a) => a,
            Err(e) => {error!("Error deserializing array: {}", e); return Redirect::to(uri!(compare));}
        };
    }
    
    if !a.reports.iter().any(|r| r.name == report.mon) {
        a.reports.push(MonReport {name: report.mon.to_owned(), small: 0, big: 0, other: 0});
    }
    match report.reason.as_str() {
        "+" => {a.reports.iter_mut().for_each(|r| if r.name == report.mon {r.big += 1});}
        "-" => {a.reports.iter_mut().for_each(|r| if r.name == report.mon {r.small += 1});}
        "!" => {a.reports.iter_mut().for_each(|r| if r.name == report.mon {r.other += 1});}
        _ => {}
    }
    let out:String = match serde_json::to_string(&a) {
        Ok(s) => s,
        Err(e) => {error!("Error serializing array: {}", e); return Redirect::to(uri!(compare))}
    };
    println!("Wrote to: {}", path);
    match fs::write(path, out) {
        Ok(_) => {Redirect::to(uri!(compare))}
        Err(e) => {error!("Error writing to file: {}", e); Redirect::to(uri!(compare))}
    }
}

#[post("/remove", data = "<mon>")]
fn remove_mon(cookies: &CookieJar<'_>, mon: Form<&str>) -> Redirect {
    match cookies.get("mons") {
        Some(cookie) => {
            let mut parts: Vec<&str> = cookie.value().split(';').collect();
            if let Ok(i) = mon.parse::<u8>() {parts.remove(i as usize);};
            let mons: String = parts.join(";");
            cookies.add(Cookie::new("mons", mons));
        }
        None => return Redirect::to(uri!(compare)),
    }
    Redirect::to(uri!(compare))
}

#[post("/reset")]
fn reset_mons(cookies: &CookieJar<'_>) -> Redirect {
    cookies.remove("mons");
    Redirect::to(uri!(compare))
}

#[catch(404)]
fn not_found(req: &Request) -> String {
    format!("Sorry, '{}' is not a valid path.", req.uri())
}

#[derive(FromForm)]
struct Order {
    from: usize,
    to: usize
}
#[post("/order", data = "<order>")]
fn order_mons(cookies: &CookieJar<'_>, order: Form<Order>) -> Redirect {
    let cookie: String = match cookies.get("mons") {
        None => {return Redirect::to(uri!(compare));}
        Some(c) => {c.value().to_string()}
    };
    let mut parts: Vec<&str> = cookie.split(';').collect();
    let element = parts.remove(order.from);
    
    parts.insert(order.to, element);
    let mons: String = parts.join(";");
    cookies.add(Cookie::new("mons", mons));
    Redirect::to(uri!(compare))
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, compare, add_mon, reset_mons, order_mons, remove_mon, report_mon])
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
