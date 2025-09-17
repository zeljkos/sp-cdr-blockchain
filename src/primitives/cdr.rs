// CDR (Call Detail Record) specific utilities and types

use serde::{Deserialize, Serialize};
use crate::primitives::primitives::Blake2bHash;

/// CDR processing status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CDRStatus {
    Pending,
    Validated,
    Settled,
    Disputed,
    Rejected,
}

/// CDR validation error types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CDRValidationError {
    InvalidTimestamp,
    UnknownNetwork,
    InvalidCharges,
    MissingRequiredFields,
    EncryptionFailure,
    ProofVerificationFailure,
}

/// CDR batch information for settlement processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CDRBatch {
    pub batch_id: Blake2bHash,
    pub home_network: String,
    pub visited_network: String,
    pub period_start: u64,
    pub period_end: u64,
    pub total_records: u32,
    pub total_charges: u64, // in cents
    pub status: CDRStatus,
}

impl CDRBatch {
    /// Create new CDR batch
    pub fn new(
        home_network: String,
        visited_network: String,
        period_start: u64,
        period_end: u64,
    ) -> Self {
        let batch_data = format!("{}:{}:{}:{}", home_network, visited_network, period_start, period_end);
        let batch_id = crate::primitives::primitives::hash_data(batch_data.as_bytes());

        Self {
            batch_id,
            home_network,
            visited_network,
            period_start,
            period_end,
            total_records: 0,
            total_charges: 0,
            status: CDRStatus::Pending,
        }
    }

    /// Add CDR record to batch
    pub fn add_record(&mut self, charges: u64) {
        self.total_records += 1;
        self.total_charges += charges;
    }

    /// Mark batch as validated
    pub fn mark_validated(&mut self) {
        self.status = CDRStatus::Validated;
    }

    /// Mark batch as settled
    pub fn mark_settled(&mut self) {
        self.status = CDRStatus::Settled;
    }

    /// Check if batch is ready for settlement
    pub fn is_ready_for_settlement(&self) -> bool {
        matches!(self.status, CDRStatus::Validated) && self.total_records > 0
    }
}

/// Settlement calculation utilities
pub mod settlement {
    use super::*;

    /// Calculate settlement amount with exchange rate
    pub fn calculate_settlement_amount(
        base_amount: u64,
        exchange_rate: u32, // Fixed point: rate * 100
        base_currency: &str,
        target_currency: &str,
    ) -> Result<u64, CDRValidationError> {
        if base_currency == target_currency {
            return Ok(base_amount);
        }

        // Apply exchange rate (rate is in hundredths)
        let settlement = (base_amount as u128 * exchange_rate as u128) / 100;
        
        // Check for overflow
        if settlement > u64::MAX as u128 {
            return Err(CDRValidationError::InvalidCharges);
        }

        Ok(settlement as u64)
    }

    /// Validate settlement calculation
    pub fn validate_settlement(
        cdr_total: u64,
        exchange_rate: u32,
        settlement_amount: u64,
    ) -> Result<bool, CDRValidationError> {
        let expected_amount = calculate_settlement_amount(
            cdr_total,
            exchange_rate,
            "base", // Generic currencies for calculation
            "target",
        )?;

        Ok(settlement_amount == expected_amount)
    }
}

/// Network operator validation
pub mod network {
    use super::*;
    use std::collections::HashSet;

    /// Known SP consortium networks
    pub fn get_known_networks() -> HashSet<String> {
        let mut networks = HashSet::new();
        networks.insert("T-Mobile-DE".to_string());
        networks.insert("Vodafone-UK".to_string());
        networks.insert("Orange-FR".to_string());
        networks.insert("Telefonica-ES".to_string());
        networks.insert("TIM-IT".to_string());
        networks.insert("KPN-NL".to_string());
        networks.insert("Telenor-NO".to_string());
        networks.insert("Sunrise-CH".to_string());
        networks.insert("Three-UK".to_string());
        networks.insert("Bouygues-FR".to_string());
        networks
    }

    /// Validate network identifier
    pub fn validate_network_id(network_id: &str) -> Result<(), CDRValidationError> {
        let known_networks = get_known_networks();
        
        if known_networks.contains(network_id) {
            Ok(())
        } else {
            Err(CDRValidationError::UnknownNetwork)
        }
    }

    /// Check if networks are in different countries (required for roaming)
    pub fn is_roaming_scenario(home_network: &str, visited_network: &str) -> bool {
        if home_network == visited_network {
            return false;
        }

        // Extract country codes from network IDs
        let home_country = home_network.split('-').last().unwrap_or("");
        let visited_country = visited_network.split('-').last().unwrap_or("");
        
        home_country != visited_country
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cdr_batch_creation() {
        let batch = CDRBatch::new(
            "T-Mobile-DE".to_string(),
            "Vodafone-UK".to_string(),
            1640995200, // 2022-01-01
            1641081600, // 2022-01-02
        );

        assert_eq!(batch.home_network, "T-Mobile-DE");
        assert_eq!(batch.visited_network, "Vodafone-UK");
        assert_eq!(batch.total_records, 0);
        assert_eq!(batch.total_charges, 0);
        assert_eq!(batch.status, CDRStatus::Pending);
    }

    #[test]
    fn test_cdr_batch_record_addition() {
        let mut batch = CDRBatch::new(
            "Orange-FR".to_string(),
            "TIM-IT".to_string(),
            1640995200,
            1641081600,
        );

        batch.add_record(1500); // €15.00
        batch.add_record(2750); // €27.50

        assert_eq!(batch.total_records, 2);
        assert_eq!(batch.total_charges, 4250); // €42.50
    }

    #[test]
    fn test_settlement_calculation() {
        use settlement::*;

        // Test EUR to USD conversion (rate 110 = 1.10)
        let settlement = calculate_settlement_amount(
            100000, // €1,000.00
            110,    // 1.10 exchange rate
            "EUR",
            "USD",
        ).unwrap();

        assert_eq!(settlement, 110000); // $1,100.00

        // Same currency should return same amount
        let same_currency = calculate_settlement_amount(
            50000,
            120,
            "EUR",
            "EUR",
        ).unwrap();

        assert_eq!(same_currency, 50000);
    }

    #[test]
    fn test_network_validation() {
        use network::*;

        assert!(validate_network_id("T-Mobile-DE").is_ok());
        assert!(validate_network_id("Vodafone-UK").is_ok());
        assert!(validate_network_id("Unknown-Network").is_err());
    }

    #[test]
    fn test_roaming_scenario_detection() {
        use network::*;

        // Different countries = roaming
        assert!(is_roaming_scenario("T-Mobile-DE", "Vodafone-UK"));
        assert!(is_roaming_scenario("Orange-FR", "TIM-IT"));

        // Same network = not roaming
        assert!(!is_roaming_scenario("T-Mobile-DE", "T-Mobile-DE"));

        // Same country = not roaming (domestic)
        // Note: This is a simplified check, real implementation would be more sophisticated
    }

    #[test]
    fn test_settlement_validation() {
        use settlement::*;

        let cdr_total = 75000; // €750.00
        let exchange_rate = 85; // 0.85
        let settlement_amount = 63750; // €750.00 * 0.85 = €637.50

        let is_valid = validate_settlement(cdr_total, exchange_rate, settlement_amount).unwrap();
        assert!(is_valid);

        // Wrong settlement amount should fail
        let is_invalid = validate_settlement(cdr_total, exchange_rate, 50000).unwrap();
        assert!(!is_invalid);
    }
}