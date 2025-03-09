use sha256::digest;
use std::time::{ UNIX_EPOCH, SystemTime };

pub trait IBlock {
    fn next_block(&self, block_data: &String) -> Block;
}

#[derive(Clone)]
#[derive(PartialEq)]
#[derive(serde::Serialize)]
pub struct Block {
    pub index: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: u64,
    pub data: String,
}

impl IBlock for Block {
    fn next_block(&self, block_data: &String) -> Block {
        let next_index: u64 = self.index + 1;
        let next_timestamp: u64 = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let next_hash: String = calculate_hash(next_index, self.hash.clone(), next_timestamp, block_data);

        Block {
            index: next_index,
            hash: next_hash,
            previous_hash: self.hash.clone(),
            timestamp: next_timestamp,
            data: block_data.clone()
        }
    }
}

pub fn get_genesis_block() -> Block {
    return Block {
        index: 0,
        hash: String::from("000000000000000000000000000000000000000000000000000000000000000"),
        previous_hash: String::from(""),
        timestamp: 1741545545,
        data: String::from("my genesis block!!")
    }
}

pub fn calculate_hash_for_block(block: Block) -> String {
    calculate_hash(block.index, block.previous_hash, block.timestamp, &block.data)
}

pub fn calculate_hash(index: u64, previous_hash: String, timestamp: u64, data: &String) -> String {
    digest(index.to_string() + previous_hash.as_str() + timestamp.to_string().as_str() + data.as_str())
}

pub fn is_block_valid(new_block: Block, previous_block: Block) -> bool {
    if previous_block.index + 1 != new_block.index {
        println!("invalid index");
        return false;
    }
    
    if previous_block.hash != new_block.previous_hash {
        println!("invalid previoushash");
        return false;
    }

    let new_block_hash: String = calculate_hash_for_block(new_block.clone());
    
    if new_block_hash != new_block.hash {
        println!("invalid hash: {} {}", new_block_hash, new_block.hash);
        return false;
    }

    return true;
}

pub fn is_valid_chain(blockchain: &Vec<Block>) -> bool {
    if get_genesis_block() != blockchain[0] {
        return false;
    }

    for i in 1..blockchain.len() {
        if !is_block_valid(blockchain[i].clone(), blockchain[i - 1].clone()) {
            return false;
        }
    }

    return true;
}