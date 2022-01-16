use std::{fs::File, io, path::Path, sync::Mutex};

use actix_web::{
    error::ErrorBadRequest,
    get, post,
    web::{self, JsonConfig},
    App, HttpServer, Responder, Result,
};
use deck::Deck;
use error::ApiError;
use log::LevelFilter;
use rand::prelude::StdRng;
use serde::Deserialize;

mod deck;
mod error;

#[derive(Deserialize)]
struct Info {
    entries: Vec<String>,
}

fn serialize(deck: &Deck<String, StdRng>) -> Result<(), io::Error> {
    let file = File::options()
        .write(true)
        .truncate(true)
        .create(true)
        .open("data.json")?;
    let writer = io::BufWriter::new(file);
    serde_json::to_writer(writer, deck)?;
    Ok(())
}

fn deserialize() -> Result<Deck<String, StdRng>, io::Error> {
    let path = Path::new("data.json");
    Ok(if path.exists() {
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);
        serde_json::from_reader(reader)?
    } else {
        Deck::<String, _>::default()
    })
}

#[post("/register")]
async fn register(
    info: web::Json<Info>,
    deck: web::Data<Mutex<Deck<String, StdRng>>>,
) -> Result<impl Responder> {
    let mut deck = deck.lock().unwrap();
    for entry in &info.entries {
        if !deck.contains(entry) {
            deck.discard(entry.clone());
        }
    }
    serialize(&deck).unwrap();
    Ok(web::Json("Registered new entries"))
}

#[post("/remove")]
async fn remove(
    info: web::Json<Info>,
    deck: web::Data<Mutex<Deck<String, StdRng>>>,
) -> Result<impl Responder> {
    let mut deck = deck.lock().unwrap();
    for entry in &info.entries {
        deck.remove(entry.clone());
    }
    serialize(&deck).unwrap();
    Ok(web::Json("Removed entries"))
}

#[get("/registered")]
async fn registered(deck: web::Data<Mutex<Deck<String, StdRng>>>) -> Result<impl Responder> {
    let deck = deck.lock().unwrap();
    let (deck, discard) = deck.entries();
    let mut result = Vec::with_capacity(deck.len() + discard.len());
    result.extend(deck.iter().cloned());
    result.extend(discard.iter().cloned());
    result.sort();
    Ok(web::Json(result))
}

#[post("/draw")]
async fn draw(deck: web::Data<Mutex<Deck<String, StdRng>>>) -> Result<impl Responder> {
    let mut deck = deck.lock().unwrap();
    let cur = deck
        .draw()
        .ok_or_else(|| ErrorBadRequest("Deck was empty"))?;
    deck.discard(cur.clone());
    serialize(&deck).unwrap();
    Ok(web::Json(cur))
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    setup_logging().unwrap();
    let data = web::Data::new(Mutex::new(deserialize()?));
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .app_data(ApiError::json_error(JsonConfig::default()))
            .service(register)
            .service(registered)
            .service(remove)
            .service(draw)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

fn setup_logging() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                chrono::Utc::now().to_rfc3339(),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}
