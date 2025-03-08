use axum::{
    routing::get,
    Json,
    Router,
};
use models::block::IBlock;
use std::time::UNIX_EPOCH;
use std::time::SystemTime;
use models::block::Block;
mod models;


#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(handler));

    let addr = "0.0.0.0:3001";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> Json<[Block; 2]> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let genesis_block = Block {
        index: 0,
        hash: String::from("816534932c2b7154836da6afc367695e6337db8a921823784c14378abed4f7d7"),
        previous_hash: None,
        timestamp: since_the_epoch.as_secs(),
        data: String::from("my genesis block!!")
    };

    let second_block = genesis_block.next_block(&String::from("my second block"));

    Json([genesis_block, second_block])
}