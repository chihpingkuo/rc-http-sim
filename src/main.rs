use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use std::{
    fs::File,
    io::Read,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::{spawn, time::interval, time::Duration};

#[derive(Debug, Clone)]
struct AppState {
    timer: Arc<Mutex<u32>>,
    timex: Arc<Mutex<Vec<Value>>>,
    temp1: Arc<Mutex<Vec<Value>>>,
    temp2: Arc<Mutex<Vec<Value>>>,
    extratemp1: Arc<Mutex<Vec<Value>>>,
}

async fn root(State(state): State<AppState>) -> (StatusCode, Json<Metrics>) {
    let mut timer = state.timer.lock().expect("Mutex was poisoned");
    let mut timex = state.timex.lock().expect("Mutex was poisoned");
    let mut temp1 = state.temp1.lock().expect("Mutex was poisoned");
    let mut temp2 = state.temp2.lock().expect("Mutex was poisoned");
    let mut extratemp1 = state.extratemp1.lock().expect("Mutex was poisoned");

    let mut index = 0;

    // compare artisan timex with current timer,
    for i in 0..(*timex).len() {
        if ((*timex).get(i).unwrap().as_f64().unwrap() < (*timer).into()) {
            index = i;
        }
    }

    println!("index: {}", index);

    let metrics = Metrics {
        timer: *timer,
        ET: (*temp1).get(index).unwrap().as_f64().unwrap() as f32,
        BT: (*temp2).get(index).unwrap().as_f64().unwrap() as f32,
        inlet: (*extratemp1).get(index).unwrap().as_f64().unwrap() as f32,
    };

    (StatusCode::OK, Json(metrics))
}

#[derive(Serialize)]
struct Metrics {
    timer: u32,
    BT: f32,
    ET: f32,
    inlet: f32,
}

#[tokio::main]
async fn main() {
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

    let timex = result
        .as_object()
        .unwrap()
        .get("timex")
        .unwrap()
        .as_array()
        .unwrap();

    let temp1 = result
        .as_object()
        .unwrap()
        .get("temp1")
        .unwrap()
        .as_array()
        .unwrap();

    let temp2 = result
        .as_object()
        .unwrap()
        .get("temp2")
        .unwrap()
        .as_array()
        .unwrap();

    let extratemp1 = result
        .as_object()
        .unwrap()
        .get("extratemp1")
        .unwrap()
        .as_array()
        .unwrap()
        .get(0)
        .unwrap()
        .as_array()
        .unwrap();

    println!("timex: {}", timex.len()); // time
    println!("temp1: {}", temp1.len()); // ET
    println!("temp2: {}", temp2.len()); // BT
    println!("extratemp1: {}", extratemp1.len()); // inlet

    let state = AppState {
        timer: Arc::new(Mutex::new(0)),
        timex: Arc::new(Mutex::new(timex.clone())),
        temp1: Arc::new(Mutex::new(temp1.clone())),
        temp2: Arc::new(Mutex::new(temp2.clone())),
        extratemp1: Arc::new(Mutex::new(extratemp1.clone())),
    };

    let state2 = state.clone();
    spawn(async move {
        let mut interval = interval(Duration::from_secs(1));

        loop {
            interval.tick().await;
            let mut timer = state2.timer.lock().expect("Mutex was poisoned");
            *timer += 1;
        }
    });

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
