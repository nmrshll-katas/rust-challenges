use k256::elliptic_curve::ops::{MulByGenerator, Reduce};
use k256::{elliptic_curve::group::GroupEncoding, Scalar};
use k256::{ProjectivePoint, U256};
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use sha2::{Digest, Sha256};

pub const GENERATOR: ProjectivePoint = ProjectivePoint::GENERATOR;

/// Non-interactive Schnorr ZK DLOG Proof scheme with a Fiat-Shamir transformation
/// Protocol:
/// The prover knows a number(Scalar) x so that Y = x*G, and they want to prove they know it without revealing x
/// The base Schnorr proof protocol works as follows:
/// 1. Commitment: The prover generates a random number r, compute T = rG
/// 2. Challenge: The verifier geenerates a random number c, sends it to the prover
/// 3. Response: The prover computes s = (r + c * x) % q, sends s to the verifier
/// 4. Verification: The verifier checks that s * G == T + (Y * c), sends true or false to the prover
///    Indeed if y=xG then (r + cx)G == rG + cxG == T + cY
///    Basically the prover has "hidden" the details of x by transforming both sides of the equation with an "affine" function (in the curve space): N -> T+cN
///
/// Making the protocol non-interactive:
/// Instead of the verifier having to "send" the challenge c to the prover, the challenge is a deterministic, pseudo-random function of [ public problem variables + public proof ]
/// That deterministic function can be any hash, we choose Sha256
/// That way both the prover and verifier can derive the challenge c independently (without communicating with each other)

/// Uses curve points from secp256k1, in projective coordinates
// RustCrypto::k256 lets us express curve points as either: Affine, Affine(compressed), Projective
// We express point in projective coordinates so that the computation is more efficient than affine (like in the python version)
#[derive(Debug, PartialEq, Eq)]
pub struct DLogProof {
    /// T = rG, where r is a random scalar generated by the prover
    t: ProjectivePoint,
    /// The prover calculates s in step 3 of the protocol, based on the challenge
    /// In the non-interactive proving system, the challenge is a deterministic number, function of the public parameters (problem + proof)
    s: Scalar,
}
// TODO I've kept the same interface as the python code here, but in practice it could be simplified, with the methods accepting only (Self (DLogProof) and Problem as params)
impl DLogProof {
    /// Create a proof that the prover knows a Scalar x so that y = x*G
    ///
    /// The prover knows a number(Scalar) x so that y = x*G, and they want to prove that they know it without revealing x
    /// y is an instance variable (public, curve point, of type ProjectivePoint), x a solution (aka witness variable, known only by the prover, of type Scalar), G a constant (the generator of the curve)
    pub fn prove(
        sid: &str,
        pid: u64,
        x: Scalar,
        y: ProjectivePoint,
        base_point: ProjectivePoint,
    ) -> Self {
        // r is a random Scalar
        let r = Scalar::generate_vartime(&mut thread_rng());
        // so t is a random curve point
        let t = ProjectivePoint::mul_by_generator(&r);
        let c = Self::calc_challenge(sid, pid, &[base_point, y, t]);
        let s = r + c * x;

        Self { t, s }
    }

    /// Verify the proof: check that the prover knows a solution x to y = x*G, without learning x
    pub fn verify(
        &self,
        sid: &str,
        pid: u64,
        y: ProjectivePoint,
        base_point: ProjectivePoint,
    ) -> bool {
        let points_to_hash = [base_point, y, self.t];
        let c = Self::calc_challenge(sid, pid, &points_to_hash);

        let lhs: ProjectivePoint = ProjectivePoint::mul_by_generator(&self.s);
        let rhs = self.t + (y * c);
        lhs == rhs
    }

    /// Compute a hash of the public variables (from problem + proof)
    fn hash_points(sid: &str, pid: u64, points: &[ProjectivePoint]) -> U256 {
        let mut hasher = Sha256::new();
        hasher.update(sid);
        hasher.update(pid.to_le_bytes());
        for point in points {
            hasher.update(point.to_bytes());
        }
        let hash = hasher.finalize();

        // Sha256 hash size is 256 bits, or 32 bytes.
        let hash_u256 = U256::from_be_slice(&hash);
        hash_u256
    }

    /// Compute the (deterministic) challenge c from the public problem variables (instance variables)
    /// In an interactive proving system, the verifier would send the (random) challenge to the prover
    /// Making this challenge deterministic (from a hash) helps turn the proving system into a non-interactive one
    fn calc_challenge(sid: &str, pid: u64, points: &[ProjectivePoint]) -> Scalar {
        let hash = Self::hash_points(sid, pid, points);
        Scalar::reduce(hash)
    }
}

pub struct Problem {
    pub sid: String,
    pub pid: u64,
    pub y: ProjectivePoint,
}
impl Problem {
    pub fn random() -> Self {
        let mut rng = thread_rng();
        let pid = rng.gen::<u64>();
        let sid = (0..7).map(|_| rng.sample(Alphanumeric) as char).collect();
        let x = Scalar::generate_vartime(&mut rng);
        let y = ProjectivePoint::mul_by_generator(&x);
        Self { sid, pid, y }
    }
    pub fn from_solution(solution_x: Scalar) -> Self {
        let mut rng = thread_rng();
        let pid = rng.gen::<u64>();
        let sid = (0..7).map(|_| rng.sample(Alphanumeric) as char).collect();
        let y = ProjectivePoint::mul_by_generator(&solution_x);
        Self { sid, pid, y }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_static_valid_proof_ok() {
        let solution_x = Scalar::generate_vartime(&mut thread_rng());
        let Problem { sid, pid, y } = Problem {
            sid: "sid".to_string(),
            pid: 1,
            y: ProjectivePoint::mul_by_generator(&solution_x),
        };

        let dlog_proof = DLogProof::prove(&sid, pid, solution_x, y, GENERATOR);

        let is_valid = dlog_proof.verify(&sid, pid, y, GENERATOR);
        assert_eq!(is_valid, true);
    }

    #[test]
    fn verify_valid_proof_ok() {
        // TODO (with more time) use deterministic pseudo-randomness based on seed for reproducibility (property-based tests)
        for _ in 1..=10 {
            let solution_x = Scalar::generate_vartime(&mut thread_rng());
            let Problem { sid, pid, y } = Problem::from_solution(solution_x);

            let dlog_proof = DLogProof::prove(&sid, pid, solution_x, y, GENERATOR);

            let is_valid = dlog_proof.verify(&sid, pid, y, GENERATOR);
            assert_eq!(is_valid, true);
        }
    }

    #[test]
    fn verify_invalid_static_proof_false() {
        let solution_x = Scalar::generate_vartime(&mut thread_rng());
        let Problem { sid, pid, y } = Problem {
            sid: "sid".to_string(),
            pid: 1,
            // y is random so doesn't verify y=x*G in the general case
            y: ProjectivePoint::mul_by_generator(&Scalar::generate_vartime(&mut thread_rng())),
        };

        let dlog_proof = DLogProof::prove(&sid, pid, solution_x, y, GENERATOR);
        let is_valid = dlog_proof.verify(&sid, pid, y, GENERATOR);
        assert_eq!(is_valid, false);
    }

    #[test]
    fn verify_invalid_proof_false() {
        // TODO (with more time) use deterministic pseudo-randomness based on seed for reproducibility (property-based tests)
        for _ in 1..=10 {
            // problem and solution are both random so don't verify y=x*G in the general case
            let solution_x = Scalar::generate_vartime(&mut thread_rng());
            let Problem { sid, pid, y } = Problem::random();

            let dlog_proof = DLogProof::prove(&sid, pid, solution_x, y, GENERATOR);
            let is_valid = dlog_proof.verify(&sid, pid, y, GENERATOR);
            assert_eq!(is_valid, false);
        }
    }
}
