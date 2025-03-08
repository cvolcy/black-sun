use sha256::digest;
use std::time::{ UNIX_EPOCH, SystemTime };


pub trait IBlock {
    fn next_block(&self, block_data: &String) -> Block;
}

#[derive(serde::Serialize)]
pub struct Block {
    pub index: u64,
    pub hash: String,
    pub previous_hash: Option<String>,
    pub timestamp: u64,
    pub data: String,
}

fn calculate_hash(index: u64, previous_hash: String, timestamp: u64, data: &String) -> String {
    digest(index.to_string() + previous_hash.as_str() + timestamp.to_string().as_str() + data.as_str())
}

impl IBlock for Block {
    fn next_block(&self, block_data: &String) -> Block {
        let next_index: u64 = self.index + 1;
        let next_timestamp: u64 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let next_hash: String = calculate_hash(next_index, self.hash.clone(), next_timestamp, block_data);

        Block {
            index: next_index,
            hash: next_hash,
            previous_hash: Some(self.hash.clone()),
            timestamp: next_timestamp,
            data: block_data.clone()
        }
    }
}