use std::sync::{Arc, Mutex};

const ZERO: u64 = 0_u64;
const ONE: u64 = 1_u64;

#[derive(Clone)]
pub struct IdGenerator {
    count_arc: Arc<Mutex<u64>>,
}

impl IdGenerator {
    pub fn new() -> Self {
        let count_arc: Arc<Mutex<u64>> = Arc::new(Mutex::new(ONE));

        return IdGenerator { count_arc };
    }

    pub fn generate(&self) -> u64 {
        let mut count_mutex = self.count_arc.lock().unwrap();
        let id = *count_mutex;

        // if count is equal to the maximum value...
        if id == u64::MAX {
            // we shall reset the count...
            *count_mutex = ZERO;
        }

        // increments the count by 1...
        *count_mutex += ONE;

        return id;
    }
}
