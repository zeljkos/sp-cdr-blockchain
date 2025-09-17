// Validator set management for SP consortium
use serde::{Deserialize, Serialize};
use crate::primitives::primitives::{Blake2bHash};
use crate::crypto::{PublicKey, ValidatorKey};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    pub validator_address: Blake2bHash,
    pub signing_key: PublicKey,
    pub voting_power: u64,
    pub network_operator: String,
    pub joined_at_height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSet {
    validators: Vec<ValidatorInfo>,
    total_voting_power: u64,
}

impl ValidatorSet {
    pub fn new(validators: Vec<ValidatorInfo>) -> Self {
        let total_voting_power = validators.iter().map(|v| v.voting_power).sum();
        Self {
            validators,
            total_voting_power,
        }
    }

    pub fn add_validator(&mut self, validator: ValidatorInfo) {
        self.total_voting_power += validator.voting_power;
        self.validators.push(validator);
    }

    pub fn remove_validator(&mut self, address: &Blake2bHash) {
        if let Some(pos) = self.validators.iter().position(|v| &v.validator_address == address) {
            let validator = self.validators.remove(pos);
            self.total_voting_power -= validator.voting_power;
        }
    }

    pub fn get_validator(&self, address: &Blake2bHash) -> Option<&ValidatorInfo> {
        self.validators.iter().find(|v| &v.validator_address == address)
    }

    pub fn validators(&self) -> &[ValidatorInfo] {
        &self.validators
    }

    pub fn total_voting_power(&self) -> u64 {
        self.total_voting_power
    }

    pub fn update_validators(&mut self, new_validators: Vec<ValidatorInfo>) {
        self.validators = new_validators;
        self.total_voting_power = self.validators.iter().map(|v| v.voting_power).sum();
    }

    pub fn finalize_epoch(&mut self) {
        // Placeholder for epoch finalization logic
    }
}