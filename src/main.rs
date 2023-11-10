use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, net::SocketAddr};

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root));

    let mut file_content = String::new();
    let mut file = File::open("23-11-05_1013.alog").unwrap();
    file.read_to_string(&mut file_content).unwrap();

    // change some character before parse
    let json_str = file_content
        .replace("\"", "\\\"")
        .replace("'", "\"")
        .replace("False", "false")
        .replace("True", "true");

    let result: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    println!("{:?}", result.as_object().unwrap().get("timex"));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root() -> (StatusCode, Json<Metrics>) {
    let metrics = Metrics {
        BT: 20.0,
        ET: 20.0,
        inlet: 40.0,
    };

    (StatusCode::OK, Json(metrics))
}

#[derive(Serialize)]
struct Metrics {
    BT: f32,
    ET: f32,
    inlet: f32,
}
