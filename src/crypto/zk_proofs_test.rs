#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fr;
    use ark_ff::UniformRand;
    use rand::thread_rng;

    #[test]
    fn test_groth16_prove_verify_roundtrip() {
        let mut rng = thread_rng();
        let mut zk = ZKProofs::new().unwrap();
        zk.setup().unwrap();

        // Use known values for a, b
        let a = Fr::from(3u64);
        let b = Fr::from(7u64);
        let (proof_bytes, vk_bytes, public_inputs) = zk.prove_toy_sum(a, b).unwrap();
        let verified = ZKProofs::verify_toy_sum(&vk_bytes, &proof_bytes, &public_inputs).unwrap();
        assert!(verified, "Groth16 proof should verify for correct inputs");
    }
}
