use sha256::digest;
use std::{sync::MutexGuard, time::{ SystemTime, UNIX_EPOCH }};

// in seconds
pub const BLOCK_GENERATION_INTERVAL: u8 = 10;
// in blocks
pub const DIFFICULTY_ADJUSTMENT_INTERVAL: u8 = 10;

pub trait IBlock {
    fn next_block(&self, block_data: &String, difficulty: u8) -> Block;
}

#[derive(Debug, PartialEq)]
#[derive(serde::Serialize)]
pub struct Block {
    pub index: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: u64,
    pub data: String,
    pub difficulty: u8,
    pub nonce: u64,
}

impl IBlock for Block {
    fn next_block(&self, block_data: &String, difficulty: u8) -> Block {
        let next_index: u64 = self.index + 1;
        let next_timestamp: u64 = get_current_timestamp();
        let next_hash: String = calculate_hash(next_index, self.hash.clone(), next_timestamp, block_data, difficulty, 0);

        Block {
            index: next_index,
            hash: next_hash,
            previous_hash: self.hash.clone(),
            timestamp: next_timestamp,
            data: block_data.clone(),
            difficulty: difficulty,
            nonce: 0,
        }
    }
}

impl Clone for Block {
    fn clone(&self) -> Self {
        Self {
            index: self.index.clone(),
            hash: self.hash.clone(),
            previous_hash: self.previous_hash.clone(),
            timestamp: self.timestamp.clone(),
            data: self.data.clone(),
            difficulty: self.difficulty.clone(),
            nonce: self.nonce.clone()
        }
    }
    
    fn clone_from(&mut self, source: &Self) {
        *self = source.clone()
    }
}

pub fn get_genesis_block() -> Block {
    return Block {
        index: 0,
        hash: String::from("000000000000000000000000000000000000000000000000000000000000000"),
        previous_hash: String::from(""),
        timestamp: 1741545545,
        data: String::from("my genesis block!!"),
        difficulty: 0,
        nonce: 0,
    }
}

pub fn calculate_hash_for_block(block: &Block) -> String {
    calculate_hash(block.index, block.clone().previous_hash, block.timestamp, &block.data, block.difficulty, block.nonce)
}

pub fn calculate_hash(index: u64, previous_hash: String, timestamp: u64, data: &String, difficulty: u8, nonce: u64) -> String {
    digest(index.to_string() + previous_hash.as_str() + timestamp.to_string().as_str() + data.as_str() + difficulty.to_string().as_str() + nonce.to_string().as_str())
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

    let new_block_hash: String = calculate_hash_for_block(&new_block);
    
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

pub fn generate_next_block(blockchain: MutexGuard<'_, Vec<Block>>, block_data: String) -> Block {
    let previous_block = &blockchain[blockchain.len() - 1];
    let difficulty = get_difficulty(blockchain.clone());

    println!("Current difficulty: {}", difficulty);

    let new_block = previous_block.next_block(&block_data, difficulty);
    let new_block = find_block(new_block.index, new_block.previous_hash, new_block.timestamp, new_block.data, new_block.difficulty);
    add_block(blockchain, new_block)
}

pub fn hash_matches_difficulty(hash: &String, difficulty: u8) -> bool {
    let hash_in_binary: String = {
        let mut binary_string = String::new();
        for c in hash.chars() {
            let digit = u8::from_str_radix(&c.to_string(), 16).unwrap();
            binary_string.push_str(&format!("{:04b}", digit));
        }
        binary_string
    };
    let required_prefix = '0'.to_string().repeat(difficulty as usize);
    return hash_in_binary.starts_with(&required_prefix);
}

fn find_block(index: u64, previous_hash: String, timestamp: u64, data: String, difficulty: u8) -> Block {
    let mut nonce: u64 = 0;
    loop {
        let hash: String = calculate_hash(index, previous_hash.clone(), timestamp, &data, difficulty, nonce);
        if hash_matches_difficulty(&hash, difficulty) {
            println!("Hash {} with difficulty {} {}", hash, difficulty, hash_matches_difficulty(&hash, difficulty));
            return Block {
                index, hash, previous_hash, timestamp, data, difficulty, nonce
            };
        }
        nonce = nonce + 1;
    };
}

fn hash_matches_block_content(block: &Block) -> bool {
    let original_hash = block.hash.clone();
    let block_hash = calculate_hash_for_block(block);
    return block_hash == original_hash;
}
    

pub fn get_difficulty(blockchain: Vec<Block>) -> u8 {
    let latest_block = &blockchain[blockchain.len() - 1];

    let difficulty = if latest_block.index % (DIFFICULTY_ADJUSTMENT_INTERVAL as u64) == 0 && latest_block.index != 0 {
        get_adjusted_difficulty(latest_block, &blockchain)
    } else {
        latest_block.difficulty
    };

    difficulty
}

fn get_adjusted_difficulty(latest_block: &Block, blockchain: &Vec<Block>) -> u8 {
    const TIME_EXPECTED: u8 = BLOCK_GENERATION_INTERVAL * DIFFICULTY_ADJUSTMENT_INTERVAL;
    let prev_adjustment_block = &blockchain[blockchain.len() - DIFFICULTY_ADJUSTMENT_INTERVAL as usize];
    let time_taken: u8 = (latest_block.timestamp - prev_adjustment_block.timestamp) as u8;
    
    if time_taken < TIME_EXPECTED / 2 {
        return prev_adjustment_block.difficulty + 1;
    } else if time_taken > TIME_EXPECTED * 2 {
        return prev_adjustment_block.difficulty - 1;
    } else {
        return prev_adjustment_block.difficulty;
    }
}

fn is_valid_timestamp(new_block: &Block, previous_block: &Block) -> bool {
    return ( previous_block.timestamp - 60 < new_block.timestamp )
        && new_block.timestamp - 60 < get_current_timestamp();
}

fn get_current_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

fn add_block(mut blockchain: MutexGuard<'_, Vec<Block>>, new_block: Block) -> Block {
    if is_valid_new_block(&new_block, &blockchain[blockchain.len() - 1]) {
        blockchain.push(new_block.clone());
    }

    new_block
}

fn is_valid_new_block(new_block: &Block, previous_block: &Block) -> bool {
    if previous_block.index + 1 != new_block.index {
        println!("invalid index");
        return false;
    } else if previous_block.hash != new_block.previous_hash {
        println!("invalid previoushash");
        return false;
    } else if !is_valid_timestamp(new_block, previous_block) {
        println!("invalid timestamp");
        return false;
    } else if !has_valid_hash(new_block) {
        return false;
    }
    
    return true;
}

fn has_valid_hash(block: &Block) -> bool {
    let block_hash = block.hash.clone();
    if !hash_matches_block_content(&block) {
        println!("invalid hash, got: {}", block_hash);
        return false;
    }

    if !hash_matches_difficulty(&block_hash, block.difficulty) {
        println!("block difficulty not satisfied. Expected: {} got: {}", block.difficulty, block.hash.clone());
    }
    return true;
}