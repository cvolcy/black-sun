use std::sync::{Arc, Mutex};
use axum::debug_handler;
use axum::{extract::State, response::IntoResponse, routing::{get, post}, Form, Json, Router};
use serde::Deserialize;
use crate::{models::block::{Block, IBlock}, AppState};

#[derive(Deserialize)]
struct MineBlockDataRequest {
    block_data: String
}

pub fn block_router(state: Arc<Mutex<AppState>>) -> Router<Arc<Mutex<AppState>>> {
    return Router::new()
        .route("/", get(get_blocks))
        .route("/mine", post(mine_block))
        .with_state(state);
    // router.route("/mineBlock",);
}

async fn get_blocks(State(state): State<Arc<Mutex<AppState>>>) -> Json<Vec::<Block>> {
    let state = state.lock().unwrap();
    let blockchain = state.blockchain.lock().expect("blockchain was poisoned");

    let mut response = blockchain.clone();
    response.reverse();

    return Json(response)
}

async fn mine_block(
    State(state): State<Arc<Mutex<AppState>>>,
    Form(mbdr): Form<MineBlockDataRequest>
) -> impl IntoResponse {
    let state = state.lock().unwrap();
    let mut blockchain = state.blockchain.lock().unwrap();

    let last_block = blockchain[blockchain.len() - 1].clone();
    let new_block = last_block.next_block(&mbdr.block_data);
    let response = new_block.clone();
    
    blockchain.push(new_block);
    Json(response)
}