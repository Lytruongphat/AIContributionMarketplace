#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, Map, String, Vec
};

#[contracttype]
#[derive(Clone, Debug)]
pub struct Task {
    pub uploader: Address,
    pub image_hash: String,
    pub is_completed: bool,
    pub reward_amount: i128,
    pub label_counts: Map<String, u32>, 
    pub contributors: Map<String, Vec<Address>>, 
    pub workers_voted: Vec<Address>, 
}

const THRESHOLD: u32 = 3;

#[contract]
pub struct AiMarketplaceContract;

#[contractimpl]
impl AiMarketplaceContract {
    
    pub fn create_task(env: Env, task_id: u64, uploader: Address, image_hash: String, reward_amount: i128) {
        uploader.require_auth();

        let task = Task {
            uploader,
            image_hash,
            is_completed: false,
            reward_amount,
            label_counts: Map::new(&env),
            contributors: Map::new(&env),
            workers_voted: Vec::new(&env),
        };
        
        env.storage().persistent().set(&task_id, &task);
    }

    pub fn submit_label(env: Env, task_id: u64, worker: Address, label: String) {
        worker.require_auth();

        let mut task: Task = env.storage().persistent().get(&task_id).unwrap_or_else(|| {
            panic!("Task không tồn tại")
        });
        
        assert!(!task.is_completed, "Nhiệm vụ này đã hoàn thành");
        assert!(!task.workers_voted.contains(&worker), "Bạn đã gán nhãn cho nhiệm vụ này rồi");
        
        task.workers_voted.push_back(worker.clone());

        let mut label_contributors = task.contributors.get(label.clone()).unwrap_or_else(|| Vec::new(&env));
        
        let current_count = task.label_counts.get(label.clone()).unwrap_or(0);
        let new_count = current_count + 1;
        task.label_counts.set(label.clone(), new_count);
        
        label_contributors.push_back(worker.clone());
        task.contributors.set(label.clone(), label_contributors);

        if new_count >= THRESHOLD {
            task.is_completed = true;
            let _payout_per_worker = task.reward_amount / (THRESHOLD as i128);
            env.events().publish((symbol_short!("payout"), task_id), label.clone());
        }

        env.storage().persistent().set(&task_id, &task);
    }
}