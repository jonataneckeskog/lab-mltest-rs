use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Serialize, Deserialize)]
pub struct Genome(pub Arc<Vec<u8>>);

impl Genome {
    pub fn new(data: Vec<u8>) -> Self {
        Self(Arc::new(data))
    }
}

impl Default for Genome {
    fn default() -> Self {
        Self(Arc::new(Vec::with_capacity(32)))
    }
}
