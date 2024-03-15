#![allow(unused)]
mod class;
mod schedule;

use axum::{Extension, Json, Router};
use axum::extract::Path;
use axum::routing::get;
use constcat::concat;
use axum::http::StatusCode;
use scraper::node::Classes;
use crate::class::Class;
use crate::schedule::Schedule;

const ZSEM_URL: &str = "https://zsem.edu.pl/plany";
const TEACHERS_URL: &str = concat!(ZSEM_URL, "/nnn.php");
const CLASSES_URL: &str = concat!(ZSEM_URL, "/lll.php");
const CLASSROOMS_URL: &str = concat!(ZSEM_URL, "/sss.php");

const PRACTICE_POSTFIX: &str = "prakt.";
const DEFAULT_PORT: u32 = 3000;


async fn get_html(url: &str) -> Result<String, reqwest::Error> {
    reqwest::get(url).await?.text().await
}

#[tokio::main]
async fn main() {
    dotenv::dotenv();
    let app = Router::new()
        .route("/class/list", get(get_classes_list))
        .route("/schedule/:schedule_id", get(get_schedule));

    let port = std::env::var("PORT").unwrap_or(DEFAULT_PORT.to_string());
    let host = format!("0.0.0.0:{}", port);
    
    let listener = tokio::net::TcpListener::bind(host).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_classes_list() -> Result<Json<Vec<Class>>, (StatusCode, String)> {
    match class::get_classes_list().await {
        Ok(v) => Ok(Json(v)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
    }
}

async fn get_schedule(Path(schedule_id): Path<String>) -> Json<Schedule> {
    let url = format!("https://zsem.edu.pl/plany/plany/{}.html", schedule_id);
    
    Json(schedule::get_schedule(&url).await)
}
