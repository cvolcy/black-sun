use std::sync::{Arc, Mutex};
use axum::{extract::State, routing::get, Json, Router};
use crate::{models::block::Block, AppState};

pub fn block_router(state: Arc<Mutex<AppState>>) -> Router<Arc<Mutex<AppState>>> {
    return Router::new()
        .route("/", get(get_blocks))
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