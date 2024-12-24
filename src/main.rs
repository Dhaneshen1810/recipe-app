use actix_cors::Cors;
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
        "Please suggest me at most 5 recipes based on the items I provide. Feel free to add condiments I did not mention to make it possible. Respond with an array of exactly 1 to 5 JSON objects, each containing the following keys and their respective values: \
        - name: The name of the dish (string). \
        - ingredients: A list of all ingredients in the dish (array of strings). \
        - additional_condiments: A list of condiments added to the recipe that were not included in the provided items (array of strings). \
        - prep_instructions: An array of steps for preparing the dish (array of strings). \
        - nutritional_info: An object with keys protein, carbs, fat, and total_calories, with values as numbers representing the quantities (in grams or calories). \
        Items provided: {:?}. Ensure the response strictly adheres to this structure.",
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

    // Get the port from the environment variable or default to 8080
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let port: u16 = port.parse().expect("PORT must be a valid u16 number");

    // Start the HTTP server with CORS enabled
    HttpServer::new(|| {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin() // Allow requests from any origin
                    .allow_any_method() // Allow GET, POST, OPTIONS, etc.
                    .allow_any_header(), // Allow any headers (e.g., Content-Type)
            )
            .service(process_items) // Register the POST /recipes handler
            .service(get_models) // Register the GET /models handler
    })
    .bind(("0.0.0.0", port))? // Bind to 0.0.0.0 and the specified port
    .run()
    .await
}
