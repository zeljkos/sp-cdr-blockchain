// Blockchain core module - extracted from core-rs-albatross
// This module contains the core blockchain structures and logic

pub mod block;
pub mod chain;
pub mod transaction;
pub mod validator_set;

// Specific imports to avoid conflicts
pub use block::{Block, MicroBlock, MacroBlock, MicroHeader, MacroHeader, MicroBody, MacroBody};
pub use chain::{ChainInfo, ChainState};
pub use transaction::{Transaction, CDRTransaction, SettlementTransaction, NetworkJoinTransaction};
pub use validator_set::{ValidatorInfo, ValidatorSet};