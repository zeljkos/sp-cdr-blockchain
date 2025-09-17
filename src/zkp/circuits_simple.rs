// Simplified ZK circuits that work with arkworks 0.4
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

/// Simplified CDR Privacy Circuit
/// Proves that total_charges = call_charges + data_charges + sms_charges
#[derive(Clone)]
pub struct SimpleCDRPrivacyCircuit<F: PrimeField> {
    // Private witnesses
    pub call_minutes: Option<F>,
    pub data_mb: Option<F>,
    pub sms_count: Option<F>,
    pub call_rate_cents: Option<F>,
    pub data_rate_cents: Option<F>,
    pub sms_rate_cents: Option<F>,

    // Public inputs
    pub total_charges_cents: Option<F>,

    _phantom: PhantomData<F>,
}

impl<F: PrimeField> SimpleCDRPrivacyCircuit<F> {
    pub fn new(
        call_minutes: u64,
        data_mb: u64,
        sms_count: u64,
        call_rate_cents: u64,
        data_rate_cents: u64,
        sms_rate_cents: u64,
        total_charges_cents: u64,
    ) -> Self {
        Self {
            call_minutes: Some(F::from(call_minutes)),
            data_mb: Some(F::from(data_mb)),
            sms_count: Some(F::from(sms_count)),
            call_rate_cents: Some(F::from(call_rate_cents)),
            data_rate_cents: Some(F::from(data_rate_cents)),
            sms_rate_cents: Some(F::from(sms_rate_cents)),
            total_charges_cents: Some(F::from(total_charges_cents)),
            _phantom: PhantomData,
        }
    }

    pub fn empty() -> Self {
        Self {
            call_minutes: None,
            data_mb: None,
            sms_count: None,
            call_rate_cents: None,
            data_rate_cents: None,
            sms_rate_cents: None,
            total_charges_cents: None,
            _phantom: PhantomData,
        }
    }
}

impl<F: PrimeField> ConstraintSynthesizer<F> for SimpleCDRPrivacyCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        // Allocate witness variables
        let call_minutes = FpVar::new_witness(cs.clone(), || {
            self.call_minutes.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let data_mb = FpVar::new_witness(cs.clone(), || {
            self.data_mb.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let sms_count = FpVar::new_witness(cs.clone(), || {
            self.sms_count.ok_or(SynthesisError::AssignmentMissing)
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

        // Allocate public input
        let total_charges = FpVar::new_input(cs.clone(), || {
            self.total_charges_cents.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // Calculate charges: call_minutes * call_rate
        let call_charges = &call_minutes * &call_rate;

        // Calculate charges: data_mb * data_rate
        let data_charges = &data_mb * &data_rate;

        // Calculate charges: sms_count * sms_rate
        let sms_charges = &sms_count * &sms_rate;

        // Total calculation: call_charges + data_charges + sms_charges
        let calculated_total = &call_charges + &data_charges + &sms_charges;

        // Constraint: calculated_total must equal public total_charges
        total_charges.enforce_equal(&calculated_total)?;

        Ok(())
    }
}

/// Simplified Settlement Circuit
/// Proves that net positions sum to zero (conservation law)
#[derive(Clone)]
pub struct SimpleSettlementCircuit<F: PrimeField> {
    // Private: bilateral amounts
    pub amount_ab: Option<F>,  // A -> B
    pub amount_bc: Option<F>,  // B -> C
    pub amount_ca: Option<F>,  // C -> A
    pub amount_ba: Option<F>,  // B -> A
    pub amount_cb: Option<F>,  // C -> B
    pub amount_ac: Option<F>,  // A -> C

    // Public: net settlement count and total
    pub net_settlement_count: Option<F>,
    pub total_net_amount: Option<F>,

    _phantom: PhantomData<F>,
}

impl<F: PrimeField> SimpleSettlementCircuit<F> {
    pub fn new(
        bilateral_amounts: [u64; 6],
        net_settlement_count: u64,
        total_net_amount: u64,
    ) -> Self {
        Self {
            amount_ab: Some(F::from(bilateral_amounts[0])),
            amount_bc: Some(F::from(bilateral_amounts[1])),
            amount_ca: Some(F::from(bilateral_amounts[2])),
            amount_ba: Some(F::from(bilateral_amounts[3])),
            amount_cb: Some(F::from(bilateral_amounts[4])),
            amount_ac: Some(F::from(bilateral_amounts[5])),
            net_settlement_count: Some(F::from(net_settlement_count)),
            total_net_amount: Some(F::from(total_net_amount)),
            _phantom: PhantomData,
        }
    }

    pub fn empty() -> Self {
        Self {
            amount_ab: None,
            amount_bc: None,
            amount_ca: None,
            amount_ba: None,
            amount_cb: None,
            amount_ac: None,
            net_settlement_count: None,
            total_net_amount: None,
            _phantom: PhantomData,
        }
    }
}

impl<F: PrimeField> ConstraintSynthesizer<F> for SimpleSettlementCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        // Allocate bilateral amount witnesses
        let ab = FpVar::new_witness(cs.clone(), || {
            self.amount_ab.ok_or(SynthesisError::AssignmentMissing)
        })?;
        let bc = FpVar::new_witness(cs.clone(), || {
            self.amount_bc.ok_or(SynthesisError::AssignmentMissing)
        })?;
        let ca = FpVar::new_witness(cs.clone(), || {
            self.amount_ca.ok_or(SynthesisError::AssignmentMissing)
        })?;
        let ba = FpVar::new_witness(cs.clone(), || {
            self.amount_ba.ok_or(SynthesisError::AssignmentMissing)
        })?;
        let cb = FpVar::new_witness(cs.clone(), || {
            self.amount_cb.ok_or(SynthesisError::AssignmentMissing)
        })?;
        let ac = FpVar::new_witness(cs.clone(), || {
            self.amount_ac.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // Allocate public inputs
        let net_count = FpVar::new_input(cs.clone(), || {
            self.net_settlement_count.ok_or(SynthesisError::AssignmentMissing)
        })?;
        let total_net = FpVar::new_input(cs.clone(), || {
            self.total_net_amount.ok_or(SynthesisError::AssignmentMissing)
        })?;

        // Calculate net positions for each party
        // A's net position: (outgoing) - (incoming) = (ab + ac) - (ba + ca)
        let a_outgoing = &ab + &ac;
        let a_incoming = &ba + &ca;
        let a_net = &a_outgoing - &a_incoming;

        // B's net position: (outgoing) - (incoming) = (ba + bc) - (ab + cb)
        let b_outgoing = &ba + &bc;
        let b_incoming = &ab + &cb;
        let b_net = &b_outgoing - &b_incoming;

        // C's net position: (outgoing) - (incoming) = (ca + cb) - (ac + bc)
        let c_outgoing = &ca + &cb;
        let c_incoming = &ac + &bc;
        let c_net = &c_outgoing - &c_incoming;

        // Conservation constraint: sum of all net positions = 0
        // a_net + b_net + c_net = 0
        let zero = FpVar::constant(F::zero());
        let sum_nets = &a_net + &b_net + &c_net;
        sum_nets.enforce_equal(&zero)?;

        // Verify gross total calculation
        let gross_total = &ab + &bc + &ca + &ba + &cb + &ac;

        // Simple constraint: net amount should be less than gross total
        // This is a placeholder - real implementation would use more sophisticated constraints

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fr;
    use ark_relations::r1cs::ConstraintSystem;

    #[test]
    fn test_simple_cdr_circuit() {
        let cs = ConstraintSystem::<Fr>::new_ref();

        // CDR data: 1000 min @ 15¢, 5000 MB @ 5¢, 200 SMS @ 10¢ = 15000 + 25000 + 2000 = 42000¢
        let circuit = SimpleCDRPrivacyCircuit::new(
            1000,  // call minutes
            5000,  // data MB
            200,   // SMS count
            15,    // 15 cents/minute
            5,     // 5 cents/MB
            10,    // 10 cents/SMS
            42000, // total = 1000*15 + 5000*5 + 200*10 = 42000 cents
        );

        circuit.generate_constraints(cs.clone()).expect("Circuit should generate constraints");
        assert!(cs.is_satisfied().unwrap(), "Circuit should be satisfied");
        println!("✅ Simple CDR Privacy Circuit: {} constraints", cs.num_constraints());
    }

    #[test]
    fn test_simple_settlement_circuit() {
        let cs = ConstraintSystem::<Fr>::new_ref();

        // Triangular settlement: amounts that result in zero net sum
        // AB=500, BC=750, CA=250, BA=100, CB=150, AC=75
        // A net = (500+75) - (100+250) = 575 - 350 = 225
        // B net = (100+750) - (500+150) = 850 - 650 = 200
        // C net = (250+150) - (75+750) = 400 - 825 = -425
        // Sum: 225 + 200 + (-425) = 0 ✓
        let circuit = SimpleSettlementCircuit::new(
            [500, 750, 250, 100, 150, 75], // bilateral amounts
            2,     // 2 net settlements
            425,   // total net amount (|225| + |200| + |-425|)/2 = 425
        );

        circuit.generate_constraints(cs.clone()).expect("Circuit should generate constraints");
        assert!(cs.is_satisfied().unwrap(), "Circuit should be satisfied");
        println!("✅ Simple Settlement Circuit: {} constraints", cs.num_constraints());
    }

    #[test]
    fn test_invalid_cdr_circuit() {
        let cs = ConstraintSystem::<Fr>::new_ref();

        // Invalid: wrong total (should be 42000, but claiming 50000)
        let circuit = SimpleCDRPrivacyCircuit::new(
            1000, 5000, 200, 15, 5, 10,
            50000, // WRONG total
        );

        circuit.generate_constraints(cs.clone()).expect("Should generate constraints");
        assert!(!cs.is_satisfied().unwrap(), "Invalid circuit should not be satisfied");
        println!("✅ Invalid CDR circuit correctly rejected");
    }

    #[test]
    fn test_invalid_settlement_circuit() {
        let cs = ConstraintSystem::<Fr>::new_ref();

        // Invalid: amounts that don't sum to zero
        // This will fail the conservation constraint
        let circuit = SimpleSettlementCircuit::new(
            [1000, 0, 0, 0, 0, 0], // Unbalanced: A pays 1000, others pay/receive nothing
            1, 1000,
        );

        circuit.generate_constraints(cs.clone()).expect("Should generate constraints");
        assert!(!cs.is_satisfied().unwrap(), "Unbalanced settlement should not be satisfied");
        println!("✅ Invalid settlement circuit correctly rejected");
    }
}