use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;

#[derive(Deserialize)]
struct ItemList {
    items: Vec<String>,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

fn get_prompt(items: &Vec<String>) -> String {
    format!(
        "Please suggest me at most 5 recipes based on the items I provide and feel free to add some condiments I did not mention to make it possible: {:?}. Please only send me an array or json objects with keys: name (name of the dish), ingredients (all ingredients in that dish), additional_condiments (all ingredients that was not included in the initial list but added by you), prep_instrucions (an array of steps for the instructions)",
        items
    )
}

#[post("/recipes")]
async fn process_items(payload: web::Json<ItemList>) -> impl Responder {
    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => return HttpResponse::InternalServerError().body("API key not set"),
    };

    let messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: "You are a helpful assistant.".to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: get_prompt(&payload.items),
        },
    ];

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap(),
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let client = reqwest::Client::new();
    let body = json!({
        "model": "gpt-3.5-turbo", // Replace with your fine-tuned model name
        "messages": messages
    });

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .headers(headers)
        .json(&body)
        .send()
        .await;

    match response {
        Ok(resp) => match resp.text().await {
            Ok(text) => HttpResponse::Ok().body(text),
            Err(err) => {
                log::error!("Error reading response from OpenAI API: {:?}", err);
                HttpResponse::InternalServerError()
                    .body(format!("Error reading response from OpenAI API: {:?}", err))
            }
        },
        Err(err) => {
            log::error!("Error contacting OpenAI API: {:?}", err);
            HttpResponse::InternalServerError()
                .body(format!("Error contacting OpenAI API: {:?}", err))
        }
    }
}

#[get("/models")]
async fn get_models() -> impl Responder {
    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => return HttpResponse::InternalServerError().body("API key not set"),
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap(),
    );

    let client = reqwest::Client::new();
    let response = client
        .get("https://api.openai.com/v1/models")
        .headers(headers)
        .send()
        .await;

    match response {
        Ok(resp) => match resp.text().await {
            Ok(text) => HttpResponse::Ok().body(text),
            Err(err) => {
                log::error!("Error reading response from OpenAI API: {:?}", err);
                HttpResponse::InternalServerError()
                    .body(format!("Error reading response from OpenAI API: {:?}", err))
            }
        },
        Err(err) => {
            log::error!("Error contacting OpenAI API: {:?}", err);
            HttpResponse::InternalServerError()
                .body(format!("Error contacting OpenAI API: {:?}", err))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .service(process_items) // Register the POST handler
            .service(get_models) // Register the GET handler
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
