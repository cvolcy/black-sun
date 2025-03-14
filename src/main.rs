use std::sync::{Arc, Mutex};
use std::net::SocketAddr;
use axum::{
    extract::State, routing::get, Json, Router
};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use routes::{blocks::block_router, peers::peers_router};
use crate::models::block::{
    get_genesis_block, is_valid_chain, Block, IBlock
};
mod models;
mod routes;

// Define your global state struct
#[derive(Clone, Debug)]
pub struct AppState {
    blockchain: Arc<Mutex<Vec<Block>>>,
    peers: Arc<Mutex<Vec<SocketAddr>>>
}

#[tokio::main]
async fn main() {
    let state = AppState {
        blockchain: Arc::new(Mutex::new(Vec::<Block>::new().to_owned())),
        peers: Arc::new(Mutex::new(Vec::<SocketAddr>::new()))
    };

    let shared_state = Arc::new(Mutex::new(state));

    initialize_blockchain(&shared_state);
    let app = Router::new()
        .route("/", get(handler))
        .nest("/blocks", block_router(shared_state.clone()))
        .nest("/peers", peers_router(shared_state.clone()))
        // logging so we can see what's going on
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .with_state(shared_state);

    let addr = "0.0.0.0:3001";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("listening on http://{}", addr);
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}

fn initialize_blockchain(state: &Arc<Mutex<AppState>>) {
    let state = state.lock().unwrap();
    let mut blockchain = state.blockchain.lock().expect("blockchain was poisoned");

    blockchain.push(get_genesis_block());
}

async fn handler(State(state): State<Arc<Mutex<AppState>>>) -> Json<Vec::<Block>> {
    let state = state.lock().unwrap();
    let mut blockchain = state.blockchain.lock().expect("blockchain was poisoned");
    let last_block =  blockchain[blockchain.len() - 1].clone();

    let new_block = last_block.next_block(&String::from("new block"));

    blockchain.push(new_block);

    if is_valid_chain(&blockchain.clone()) {
        let mut response = blockchain.clone();
        response.reverse();
        return Json(response);
    }

    let mut response = blockchain.clone();
    response.reverse();

    return Json(response)
}

pub fn replace_chain(State(state): State<AppState>, new_blocks: &Vec<Block>) {
    let mut blockchain = state.blockchain.lock().expect("blockchain was poisoned");

    if is_valid_chain(new_blocks) && new_blocks.len() > blockchain.len() {
        *blockchain = new_blocks.clone();
        // broadcast_latest_chain()
    }
}
