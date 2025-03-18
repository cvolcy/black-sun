use std::sync::{Arc, Mutex};
use axum::{extract::State, response::IntoResponse, routing::{get, post}, Form, Json, Router};
use serde::Deserialize;
use crate::{models::block::{generate_next_block, Block, IBlock}, AppState};

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
    let blockchain: std::sync::MutexGuard<'_, Vec<Block>> = state.blockchain.lock().unwrap();

    let new_block = generate_next_block(blockchain, mbdr.block_data);
    let response = new_block.clone();
    
    Json(response)
}