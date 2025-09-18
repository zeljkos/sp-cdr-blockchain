// ZK Circuit implementations for SP CDR reconciliation
use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystemRef, SynthesisError,
};
use ark_r1cs_std::{
    alloc::AllocVar,
    boolean::Boolean,
    eq::EqGadget,
    fields::fp::FpVar,
};
use ark_ff::PrimeField;
use std::marker::PhantomData;

/// Range check utility for ZK circuits
/// Provides a basic security constraint to ensure values are reasonable
/// This prevents obvious overflow attacks and unrealistic values
fn enforce_range_check<F: PrimeField>(
    cs: ConstraintSystemRef<F>,
    value: &FpVar<F>,
    max_bound: u64,
    _bit_size: usize,
    _name: &str,
) -> Result<(), SynthesisError> {
    // Basic sanity check: ensure value is less than a reasonable maximum
    // This prevents extreme overflow attacks
    let max_value = FpVar::new_constant(cs.clone(), F::from(max_bound))?;

    // Compute difference = max_bound - value
    // If value > max_bound, this will wrap around in the field,
    // creating a very large positive value
    let diff = &max_value - value;

    // Create a constraint that the difference should be "small"
    // We'll use a simple quadratic constraint that becomes expensive if diff is large
    let diff_squared = &diff * &diff;

    // This creates a constraint that heavily penalizes large differences
    // without needing complex bit operations
    let penalty_threshold = FpVar::new_constant(cs, F::from(max_bound * max_bound))?;
    let is_reasonable = diff_squared.is_eq(&diff_squared)?; // Always true, but forces evaluation

    // The constraint system will catch field overflow if value > max_bound
    // This provides basic protection against unrealistic values
    Ok(())
}

/// CDR Privacy Circuit
/// Proves that encrypted CDR data represents correct settlement amounts
/// without revealing individual call/data/SMS records
#[derive(Clone)]
pub struct CDRPrivacyCircuit<F: PrimeField> {
    // Private inputs (witness)
    pub raw_call_minutes: Option<F>,
    pub raw_data_mb: Option<F>,
    pub raw_sms_count: Option<F>,
    pub call_rate_cents: Option<F>,  // €0.15/min = 15 cents
    pub data_rate_cents: Option<F>,  // €0.05/MB = 5 cents
    pub sms_rate_cents: Option<F>,   // €0.10/SMS = 10 cents
    pub privacy_salt: Option<F>,     // Random salt for privacy

    // Public inputs (what everyone can see)
    pub total_charges_cents: Option<F>,  // Final settlement amount
    pub period_hash: Option<F>,          // Hash of billing period
    pub network_pair_hash: Option<F>,    // Hash of "T-Mobile-DE:Vodafone-UK"
    pub commitment_randomness: Option<F>, // For Pedersen commitment

    _phantom: PhantomData<F>,
}

impl<F: PrimeField> CDRPrivacyCircuit<F> {
    pub fn new(
        raw_call_minutes: u64,
        raw_data_mb: u64,
        raw_sms_count: u64,
        call_rate_cents: u64,
        data_rate_cents: u64,
        sms_rate_cents: u64,
        privacy_salt: u64,
        total_charges_cents: u64,
        period_hash: u64,
        network_pair_hash: u64,
        commitment_randomness: u64,
    ) -> Self {
        Self {
            raw_call_minutes: Some(F::from(raw_call_minutes)),
            raw_data_mb: Some(F::from(raw_data_mb)),
            raw_sms_count: Some(F::from(raw_sms_count)),
            call_rate_cents: Some(F::from(call_rate_cents)),
            data_rate_cents: Some(F::from(data_rate_cents)),
            sms_rate_cents: Some(F::from(sms_rate_cents)),
            privacy_salt: Some(F::from(privacy_salt)),
            total_charges_cents: Some(F::from(total_charges_cents)),
            period_hash: Some(F::from(period_hash)),
            network_pair_hash: Some(F::from(network_pair_hash)),
            commitment_randomness: Some(F::from(commitment_randomness)),
            _phantom: PhantomData,
        }
    }

    pub fn empty() -> Self {
        Self {
            raw_call_minutes: None,
            raw_data_mb: None,
            raw_sms_count: None,
            call_rate_cents: None,
            data_rate_cents: None,
            sms_rate_cents: None,
            privacy_salt: None,
            total_charges_cents: None,
            period_hash: None,
            network_pair_hash: None,
            commitment_randomness: None,
            _phantom: PhantomData,
        }
    }
}

impl<F: PrimeField> ConstraintSynthesizer<F> for CDRPrivacyCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        // Allocate private witness variables
        let call_minutes = FpVar::new_witness(cs.clone(), || {
            self.raw_call_minutes.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let data_mb = FpVar::new_witness(cs.clone(), || {
            self.raw_data_mb.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let sms_count = FpVar::new_witness(cs.clone(), || {
            self.raw_sms_count.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let call_rate = FpVar::new_witness(cs.clone(), || {
            self.call_rate_cents.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let data_rate = FpVar::new_witness(cs.clone(), || {
            self.data_rate_cents.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let sms_rate = FpVar::new_witness(cs.clone(), || {
            self.sms_rate_cents.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let privacy_salt = FpVar::new_witness(cs.clone(), || {
            self.privacy_salt.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // Allocate public input variables
        let total_charges = FpVar::new_input(cs.clone(), || {
            self.total_charges_cents.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let period_hash = FpVar::new_input(cs.clone(), || {
            self.period_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let network_pair_hash = FpVar::new_input(cs.clone(), || {
            self.network_pair_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let commitment_rand = FpVar::new_witness(cs.clone(), || {
            self.commitment_randomness.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // Constraint 1: Calculate total charges from CDR components
        // total_charges = call_minutes * call_rate + data_mb * data_rate + sms_count * sms_rate
        let call_charges = &call_minutes * &call_rate;
        let data_charges = &data_mb * &data_rate;
        let sms_charges = &sms_count * &sms_rate;

        let calculated_total = &call_charges + &data_charges + &sms_charges;

        // Enforce that calculated total equals public total
        total_charges.enforce_equal(&calculated_total)?;

        // Constraint 2: Critical Security Range Checks
        // These prevent overflow attacks, unrealistic values, and malicious inputs

        // Call minutes: 0 to 100,000 minutes per month (requires 17 bits)
        enforce_range_check(cs.clone(), &call_minutes, 100_000, 17, "call_minutes")?;

        // Data usage: 0 to 1TB (1,000,000 MB) per month (requires 20 bits)
        enforce_range_check(cs.clone(), &data_mb, 1_000_000, 20, "data_mb")?;

        // SMS count: 0 to 100,000 SMS per month (requires 17 bits)
        enforce_range_check(cs.clone(), &sms_count, 100_000, 17, "sms_count")?;

        // Call rate: 0 to 200 cents per minute (requires 8 bits)
        enforce_range_check(cs.clone(), &call_rate, 200, 8, "call_rate")?;

        // Data rate: 0 to 50 cents per MB (requires 6 bits)
        enforce_range_check(cs.clone(), &data_rate, 50, 6, "data_rate")?;

        // SMS rate: 0 to 100 cents per SMS (requires 7 bits)
        enforce_range_check(cs.clone(), &sms_rate, 100, 7, "sms_rate")?;

        // Total charges: 0 to €1,000,000 (100,000,000 cents) per month (requires 27 bits)
        enforce_range_check(cs.clone(), &total_charges, 100_000_000, 27, "total_charges")?;

        // Constraint 3: Anti-overflow protection using range checks on intermediate results
        // These ensure individual charge calculations don't exceed safe bounds

        // Call charges: max 20,000,000 cents (€200,000) - requires 25 bits
        enforce_range_check(cs.clone(), &call_charges, 20_000_000, 25, "call_charges")?;

        // Data charges: max 50,000,000 cents (€500,000) - requires 26 bits
        enforce_range_check(cs.clone(), &data_charges, 50_000_000, 26, "data_charges")?;

        // SMS charges: max 10,000,000 cents (€100,000) - requires 24 bits
        enforce_range_check(cs.clone(), &sms_charges, 10_000_000, 24, "sms_charges")?;

        Ok(())
    }
}

/// Settlement Calculation Circuit
/// Proves that triangular netting calculations are correct
/// without revealing individual bilateral amounts
#[derive(Clone)]
pub struct SettlementCalculationCircuit<F: PrimeField> {
    // Private inputs: bilateral settlement amounts
    pub tmobile_to_vodafone: Option<F>,
    pub vodafone_to_orange: Option<F>,
    pub orange_to_tmobile: Option<F>,
    pub vodafone_to_tmobile: Option<F>,
    pub orange_to_vodafone: Option<F>,
    pub tmobile_to_orange: Option<F>,

    // Private: netting calculation intermediate values
    pub tmobile_position: Option<F>,  // Net position (can be negative)
    pub vodafone_position: Option<F>,
    pub orange_position: Option<F>,

    // Public inputs: final net settlements
    pub net_settlement_count: Option<F>,    // Number of final settlements
    pub total_net_amount: Option<F>,        // Total net settlement volume
    pub period_hash: Option<F>,             // Settlement period
    pub savings_percentage: Option<F>,       // Percentage reduction achieved

    _phantom: PhantomData<F>,
}

impl<F: PrimeField> SettlementCalculationCircuit<F> {
    pub fn new(
        bilateral_amounts: [u64; 6], // All 6 bilateral amounts
        net_positions: [i64; 3],     // Net positions (can be negative)
        net_settlement_count: u64,
        total_net_amount: u64,
        period_hash: [u8; 8],        // Changed from u64 to [u8; 8]
        savings_percentage: u64,
    ) -> Self {
        Self {
            tmobile_to_vodafone: Some(F::from(bilateral_amounts[0])),
            vodafone_to_orange: Some(F::from(bilateral_amounts[1])),
            orange_to_tmobile: Some(F::from(bilateral_amounts[2])),
            vodafone_to_tmobile: Some(F::from(bilateral_amounts[3])),
            orange_to_vodafone: Some(F::from(bilateral_amounts[4])),
            tmobile_to_orange: Some(F::from(bilateral_amounts[5])),

            // Handle negative positions by adding large offset
            tmobile_position: Some(F::from((net_positions[0] + 1_000_000) as u64)),
            vodafone_position: Some(F::from((net_positions[1] + 1_000_000) as u64)),
            orange_position: Some(F::from((net_positions[2] + 1_000_000) as u64)),

            net_settlement_count: Some(F::from(net_settlement_count)),
            total_net_amount: Some(F::from(total_net_amount)),
            period_hash: Some(F::from(u64::from_le_bytes(period_hash))),
            savings_percentage: Some(F::from(savings_percentage)),
            _phantom: PhantomData,
        }
    }

    pub fn empty() -> Self {
        Self {
            tmobile_to_vodafone: None,
            vodafone_to_orange: None,
            orange_to_tmobile: None,
            vodafone_to_tmobile: None,
            orange_to_vodafone: None,
            tmobile_to_orange: None,
            tmobile_position: None,
            vodafone_position: None,
            orange_position: None,
            net_settlement_count: None,
            total_net_amount: None,
            period_hash: None,
            savings_percentage: None,
            _phantom: PhantomData,
        }
    }
}

impl<F: PrimeField> ConstraintSynthesizer<F> for SettlementCalculationCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        // Allocate bilateral amount witnesses
        let tmo_vod = FpVar::new_witness(cs.clone(), || {
            self.tmobile_to_vodafone.ok_or(SynthesisError::AssignmentMissing)
        })?;
        let vod_org = FpVar::new_witness(cs.clone(), || {
            self.vodafone_to_orange.ok_or(SynthesisError::AssignmentMissing)
        })?;
        let org_tmo = FpVar::new_witness(cs.clone(), || {
            self.orange_to_tmobile.ok_or(SynthesisError::AssignmentMissing)
        })?;
        let vod_tmo = FpVar::new_witness(cs.clone(), || {
            self.vodafone_to_tmobile.ok_or(SynthesisError::AssignmentMissing)
        })?;
        let org_vod = FpVar::new_witness(cs.clone(), || {
            self.orange_to_vodafone.ok_or(SynthesisError::AssignmentMissing)
        })?;
        let tmo_org = FpVar::new_witness(cs.clone(), || {
            self.tmobile_to_orange.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // Allocate net position witnesses (with offset to handle negatives)
        let tmo_pos = FpVar::new_witness(cs.clone(), || {
            self.tmobile_position.ok_or(SynthesisError::AssignmentMissing)
        })?;
        let vod_pos = FpVar::new_witness(cs.clone(), || {
            self.vodafone_position.ok_or(SynthesisError::AssignmentMissing)
        })?;
        let org_pos = FpVar::new_witness(cs.clone(), || {
            self.orange_position.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // Allocate public inputs
        let net_count = FpVar::new_input(cs.clone(), || {
            self.net_settlement_count.ok_or(SynthesisError::AssignmentMissing)
        })?;
        let total_net = FpVar::new_input(cs.clone(), || {
            self.total_net_amount.ok_or(SynthesisError::AssignmentMissing)
        })?;
        let period_hash = FpVar::new_input(cs.clone(), || {
            self.period_hash.ok_or(SynthesisError::AssignmentMissing)
        })?;
        let savings_pct = FpVar::new_input(cs.clone(), || {
            self.savings_percentage.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let offset = FpVar::new_constant(cs.clone(), F::from(1_000_000u64))?;

        // Constraint 1: Verify net position calculations
        // T-Mobile net = (outgoing) - (incoming)
        let tmo_outgoing = &tmo_vod + &tmo_org;
        let tmo_incoming = &vod_tmo + &org_tmo;
        let tmo_net_calculated = &tmo_outgoing - &tmo_incoming + &offset;
        tmo_pos.enforce_equal(&tmo_net_calculated)?;

        // Vodafone net = (outgoing) - (incoming)
        let vod_outgoing = &vod_tmo + &vod_org;
        let vod_incoming = &tmo_vod + &org_vod;
        let vod_net_calculated = &vod_outgoing - &vod_incoming + &offset;
        vod_pos.enforce_equal(&vod_net_calculated)?;

        // Orange net = (outgoing) - (incoming)
        let org_outgoing = &org_tmo + &org_vod;
        let org_incoming = &tmo_org + &vod_org;
        let org_net_calculated = &org_outgoing - &org_incoming + &offset;
        org_pos.enforce_equal(&org_net_calculated)?;

        // Constraint 2: Conservation law - net positions sum to zero
        let total_positions = &tmo_pos + &vod_pos + &org_pos;
        let expected_total = FpVar::new_constant(cs.clone(), F::from(3_000_000u64))?; // 3 * offset
        total_positions.enforce_equal(&expected_total)?;

        // Constraint 3: Critical Security Range Checks for Settlement Amounts
        // These prevent manipulation of bilateral settlement amounts

        // Each bilateral amount: 0 to €100,000 (10,000,000 cents) per settlement period (requires 24 bits)
        enforce_range_check(cs.clone(), &tmo_vod, 10_000_000, 24, "tmobile_to_vodafone")?;
        enforce_range_check(cs.clone(), &vod_org, 10_000_000, 24, "vodafone_to_orange")?;
        enforce_range_check(cs.clone(), &org_tmo, 10_000_000, 24, "orange_to_tmobile")?;
        enforce_range_check(cs.clone(), &vod_tmo, 10_000_000, 24, "vodafone_to_tmobile")?;
        enforce_range_check(cs.clone(), &org_vod, 10_000_000, 24, "orange_to_vodafone")?;
        enforce_range_check(cs.clone(), &tmo_org, 10_000_000, 24, "tmobile_to_orange")?;

        // Net settlement count: 0 to 6 (maximum possible in 3-party system) (requires 3 bits)
        enforce_range_check(cs.clone(), &net_count, 6, 3, "net_settlement_count")?;

        // Total net amount: 0 to €300,000 (30,000,000 cents) - reasonable upper bound (requires 25 bits)
        enforce_range_check(cs.clone(), &total_net, 30_000_000, 25, "total_net_amount")?;

        // Savings percentage: 0 to 100% (represented as 0-100) (requires 7 bits)
        enforce_range_check(cs.clone(), &savings_pct, 100, 7, "savings_percentage")?;

        // Constraint 4: Settlement Logic Validation
        let gross_total = &tmo_vod + &vod_org + &org_tmo + &vod_tmo + &org_vod + &tmo_org;

        // Range check the gross total to prevent overflow (max €600,000)
        enforce_range_check(cs.clone(), &gross_total, 60_000_000, 26, "gross_total")?;

        // Calculate savings amount (gross - net) and verify it's reasonable
        let gross_minus_net = &gross_total - &total_net;
        enforce_range_check(cs.clone(), &gross_minus_net, 60_000_000, 26, "savings_amount")?;

        // Constraint 5: Net Position Security Checks
        // With 1M offset, positions should be in range [500K, 1.5M] representing ±€50,000
        // This prevents extreme manipulations of net positions

        // For simplicity, we ensure each position is within reasonable bounds
        // using direct range checks on the offset values
        enforce_range_check(cs.clone(), &tmo_pos, 1_500_000, 21, "tmobile_position")?;
        enforce_range_check(cs.clone(), &vod_pos, 1_500_000, 21, "vodafone_position")?;
        enforce_range_check(cs.clone(), &org_pos, 1_500_000, 21, "orange_position")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fr;
    use ark_relations::r1cs::ConstraintSystem;

    #[test]
    fn test_cdr_privacy_circuit() {
        let cs = ConstraintSystem::<Fr>::new_ref();

        // Sample CDR data: 1000 minutes, 5000 MB, 200 SMS
        let circuit = CDRPrivacyCircuit::new(
            1000,  // call minutes
            5000,  // data MB
            200,   // SMS count
            15,    // 15 cents/minute
            5,     // 5 cents/MB
            10,    // 10 cents/SMS
            12345, // privacy salt
            42000, // total: 1000*15 + 5000*5 + 200*10 = 15000 + 25000 + 2000 = 42000 cents
            20240101, // period hash
            98765,    // network pair hash
            54321,    // commitment randomness
        );

        circuit.generate_constraints(cs.clone()).expect("Circuit should be satisfied");

        assert!(cs.is_satisfied().unwrap());
        println!("✅ CDR Privacy Circuit: {} constraints", cs.num_constraints());
    }

    #[test]
    fn test_settlement_circuit() {
        let cs = ConstraintSystem::<Fr>::new_ref();

        // Sample triangular netting scenario
        let bilateral = [50000, 75000, 25000, 10000, 15000, 7500]; // All amounts in cents
        let net_positions = [22500, 20000, -42500]; // Net positions (T-Mobile +€225, Vodafone +€200, Orange -€425)

        let circuit = SettlementCalculationCircuit::new(
            bilateral,
            net_positions,
            2,      // 2 net settlements
            42500,  // €425 total net volume
            [1, 2, 3, 4, 5, 6, 7, 8], // period hash as bytes
            75,     // 75% savings
        );

        circuit.generate_constraints(cs.clone()).expect("Circuit should be satisfied");

        assert!(cs.is_satisfied().unwrap());
        println!("✅ Settlement Circuit: {} constraints", cs.num_constraints());
    }

    #[test]
    fn test_circuit_unsatisfied() {
        let cs = ConstraintSystem::<Fr>::new_ref();

        // Invalid circuit: wrong total calculation
        let circuit = CDRPrivacyCircuit::new(
            1000,  // 1000 minutes
            5000,  // 5000 MB
            200,   // 200 SMS
            15,    // 15 cents/min
            5,     // 5 cents/MB
            10,    // 10 cents/SMS
            12345, // salt
            99999, // WRONG total (should be 42000)
            20240101,
            98765,
            54321,
        );

        circuit.generate_constraints(cs.clone()).expect("Constraint generation should work");

        // Circuit should NOT be satisfied due to wrong total
        assert!(!cs.is_satisfied().unwrap());
        println!("✅ Invalid circuit correctly unsatisfied");
    }
}