use crate::Result;
use ark_bn254::{Bn254, Fr};
use ark_groth16::{Groth16, Proof, ProvingKey, VerifyingKey, prepare_verifying_key};
use ark_groth16::r1cs_to_qap::LibsnarkReduction;
use ark_snark::SNARK;
use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystemRef, SynthesisError, Variable,
};
use ark_relations::lc;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::rand::thread_rng;

#[derive(Clone, Default)]
pub struct ToyCircuit {
    // Public input: claimed sum
    pub a_plus_b: Option<Fr>,
    // Private inputs
    pub a: Option<Fr>,
    pub b: Option<Fr>,
}

impl ConstraintSynthesizer<Fr> for ToyCircuit {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> core::result::Result<(), SynthesisError> {
        // Allocate private inputs a, b as witnesses
        let a_val = self.a.ok_or(SynthesisError::AssignmentMissing)?;
        let b_val = self.b.ok_or(SynthesisError::AssignmentMissing)?;
        let sum_val = self.a_plus_b.ok_or(SynthesisError::AssignmentMissing)?;

        // Allocate variables in the CS
        let a_var = cs.new_witness_variable(|| Ok(a_val))?;
        let b_var = cs.new_witness_variable(|| Ok(b_val))?;
        let sum_var = cs.new_input_variable(|| Ok(sum_val))?; // public input

        // Enforce: a + b = sum
        // That is: a + b - sum = 0
        cs.enforce_constraint(
            lc!() + a_var + b_var,
            lc!() + Variable::One,
            lc!() + sum_var,
        )?;
        Ok(())
    }
}

pub struct ZKProofs {
    pk: Option<ProvingKey<Bn254>>,
    vk: Option<VerifyingKey<Bn254>>,
}

impl ZKProofs {
    pub fn new() -> Result<Self> { Ok(Self { pk: None, vk: None }) }

    pub fn setup(&mut self) -> Result<()> {
        let mut rng = thread_rng();
        let empty = ToyCircuit::default();
    let (pk, vk) = Groth16::<Bn254, LibsnarkReduction>::circuit_specific_setup(empty, &mut rng)?;
        self.pk = Some(pk);
        self.vk = Some(vk);
        Ok(())
    }

    pub fn prove_toy_sum(&self, a: Fr, b: Fr) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    let pk = self.pk.as_ref().ok_or_else(|| anyhow::anyhow!("pk not initialized"))?;
    let vk = self.vk.as_ref().ok_or_else(|| anyhow::anyhow!("vk not initialized"))?;

        let a_plus_b = a + b;
        let circuit = ToyCircuit { a_plus_b: Some(a_plus_b), a: Some(a), b: Some(b) };
        let mut rng = thread_rng();
    let proof: Proof<Bn254> = Groth16::<Bn254, LibsnarkReduction>::prove(pk, circuit, &mut rng)?;

        // Serialize proof, vk, public inputs
        let mut proof_bytes = Vec::new();
        proof.serialize_compressed(&mut proof_bytes)?;

        let mut vk_bytes = Vec::new();
        vk.serialize_compressed(&mut vk_bytes)?;

        let mut public_inputs = Vec::new();
        a_plus_b.serialize_compressed(&mut public_inputs)?;

        Ok((proof_bytes, vk_bytes, public_inputs))
    }

    pub fn verify_toy_sum(vk_bytes: &[u8], proof_bytes: &[u8], public_inputs: &[u8]) -> Result<bool> {
        let vk = VerifyingKey::<Bn254>::deserialize_compressed(vk_bytes)?;
        let pvk = prepare_verifying_key(&vk);
        let proof = Proof::<Bn254>::deserialize_compressed(proof_bytes)?;
        let a_plus_b = Fr::deserialize_compressed(public_inputs)?;
    Ok(Groth16::<Bn254, LibsnarkReduction>::verify_proof(&pvk, &proof, &[a_plus_b])?)
    }
}
