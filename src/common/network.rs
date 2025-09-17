// Network layer - using real SP network implementation
pub use crate::network::{SPNetworkManager, NetworkCommand, NetworkEvent, SPNetworkMessage};

// Legacy wrapper for compatibility
pub struct NetworkManager {
    _placeholder: (),
}

impl NetworkManager {
    pub fn new() -> Self {
        Self {
            _placeholder: (),
        }
    }
}