use axum::{
    routing::get,
    Json,
    Router,
};
use models::block::{
    get_genesis_block,
    is_valid_chain,
    IBlock,
    Block
};
mod models;

static mut BLOCKCHAIN: Vec::<Block> = Vec::<Block>::new();

#[tokio::main]
async fn main() {

    initialize_blockchain();
    let app = Router::new()
        .route("/", get(handler));

    let addr = "0.0.0.0:3001";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

fn initialize_blockchain() {
    unsafe { BLOCKCHAIN.push(get_genesis_block()) };
}

async fn handler() -> Json<Vec::<Block>> {
    unsafe {
        let last_block =  BLOCKCHAIN [BLOCKCHAIN.len() - 1].clone();

        let new_block = last_block.next_block(&String::from("new block"));

        BLOCKCHAIN.push(new_block);

        if is_valid_chain(&BLOCKCHAIN.clone()) {
            return Json(BLOCKCHAIN.clone());
        }

        BLOCKCHAIN.pop();

        return Json(BLOCKCHAIN.clone())
    }
}



pub unsafe fn replace_chain(new_blocks: &Vec<Block>) {
    if is_valid_chain(new_blocks) && new_blocks.len() > BLOCKCHAIN.len() {
        BLOCKCHAIN = new_blocks.clone();
        // broadcast_latest_chain()
    }
}