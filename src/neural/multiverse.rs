use super::vm_network::Community;
use std::collections::HashMap;

struct Multiverse {
    // Communities indexed by coordinates or a simple ID
    spaces: HashMap<usize, Community>,
}
