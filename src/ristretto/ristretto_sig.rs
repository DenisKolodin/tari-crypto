// Copyright 2019 The Tari Project
//
// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
// following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
// disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the
// following disclaimer in the documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote
// products derived from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
// INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
// WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
// USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use crate::{
    ristretto::{RistrettoPublicKey, RistrettoSecretKey},
    signatures::SchnorrSignature,
};

/// # A Schnorr signature implementation on Ristretto
///
/// Find out more about [Schnorr signatures](https://tlu.tarilabs.com/cryptography/digital_signatures/introduction.html).
///
/// `RistrettoSchnorr` utilises the [curve25519-dalek](https://github.com/dalek-cryptography/curve25519-dalek1)
/// implementation of `ristretto255` to provide Schnorr signature functionality.
///
/// In short, a Schnorr sig is made up of the pair _(R, s)_, where _R_ is a public key (of a secret nonce) and _s_ is
/// the signature.
///
/// ## Creating signatures
///
/// You can create a `RisrettoSchnorr` from it's component parts:
///
/// ```edition2018
/// # use tari_crypto::ristretto::*;
/// # use tari_crypto::keys::*;
/// # use tari_crypto::signatures::SchnorrSignature;
/// # use tari_utilities::ByteArray;
/// # use tari_utilities::hex::Hex;
///
/// let public_r =
///     RistrettoPublicKey::from_hex("6a493210f7499cd17fecb510ae0cea23a110e8d5b901f8acadd3095c73a3b919").unwrap();
/// let s = RistrettoSecretKey::from_bytes(b"10000000000000000000000000000000").unwrap();
/// let sig = RistrettoSchnorr::new(public_r, s);
/// ```
///
/// or you can create a signature by signing a message:
///
/// ```rust
/// # use tari_crypto::ristretto::*;
/// # use tari_crypto::keys::*;
/// # use tari_crypto::signatures::SchnorrSignature;
/// # use tari_crypto::common::*;
/// # use digest::Digest;
///
/// fn get_keypair() -> (RistrettoSecretKey, RistrettoPublicKey) {
///     let mut rng = rand::thread_rng();
///     let k = RistrettoSecretKey::random(&mut rng);
///     let pk = RistrettoPublicKey::from_secret_key(&k);
///     (k, pk)
/// }
///
/// #[allow(non_snake_case)]
/// let (k, P) = get_keypair();
/// let (r, R) = get_keypair();
/// let e = Blake256::digest(b"Small Gods");
/// let sig = RistrettoSchnorr::sign(k, r, &e);
/// ```
///
/// # Verifying signatures
///
/// Given a signature, (R,s) and a Challenge, e, you can verify that the signature is valid by calling the `verify`
/// method:
///
/// ```edition2018
/// # use tari_crypto::ristretto::*;
/// # use tari_crypto::keys::*;
/// # use tari_crypto::signatures::SchnorrSignature;
/// # use tari_crypto::common::*;
/// # use tari_utilities::hex::*;
/// # use tari_utilities::ByteArray;
/// # use digest::Digest;
///
/// # #[allow(non_snake_case)]
/// let P = RistrettoPublicKey::from_hex("74896a30c89186b8194e25f8c1382f8d3081c5a182fb8f8a6d34f27fbefbfc70").unwrap();
/// let R = RistrettoPublicKey::from_hex("fa14cb581ce5717248444721242e6b195a482d503a853dea4acb513074d8d803").unwrap();
/// let s = RistrettoSecretKey::from_hex("bd0b253a619310340a4fa2de54cdd212eac7d088ee1dc47e305c3f6cbd020908").unwrap();
/// let sig = RistrettoSchnorr::new(R, s);
/// let e = Blake256::digest(b"Maskerade");
/// assert!(sig.verify_challenge(&P, &e));
/// ```
pub type RistrettoSchnorr = SchnorrSignature<RistrettoPublicKey, RistrettoSecretKey>;

#[cfg(test)]
mod test {
    use digest::Digest;
    use tari_utilities::{hex::from_hex, ByteArray};

    use crate::{
        common::Blake256,
        keys::{PublicKey, SecretKey},
        ristretto::{RistrettoPublicKey, RistrettoSchnorr, RistrettoSecretKey},
    };

    #[test]
    fn default() {
        let sig = RistrettoSchnorr::default();
        assert_eq!(sig.get_signature(), &RistrettoSecretKey::default());
        assert_eq!(sig.get_public_nonce(), &RistrettoPublicKey::default());
    }

    /// Create a signature, and then verify it. Also checks that some invalid signatures fail to verify
    #[test]
    #[allow(non_snake_case)]
    fn sign_and_verify_message() {
        let mut rng = rand::thread_rng();
        let (k, P) = RistrettoPublicKey::random_keypair(&mut rng);
        let (r, R) = RistrettoPublicKey::random_keypair(&mut rng);
        let e = Blake256::new()
            .chain(P.as_bytes())
            .chain(R.as_bytes())
            .chain(b"Small Gods")
            .finalize();
        let e_key = RistrettoSecretKey::from_bytes(&e).unwrap();
        let s = &r + &e_key * &k;
        let sig = RistrettoSchnorr::sign(k, r, &e).unwrap();
        let R_calc = sig.get_public_nonce();
        assert_eq!(R, *R_calc);
        assert_eq!(sig.get_signature(), &s);
        assert!(sig.verify_challenge(&P, &e));
        // Doesn't work for invalid credentials
        assert!(!sig.verify_challenge(&R, &e));
        // Doesn't work for different challenge
        let wrong_challenge = Blake256::digest(b"Guards! Guards!");
        assert!(!sig.verify_challenge(&P, &wrong_challenge));
    }

    /// This test checks that the linearity of Schnorr signatures hold, i.e. that s = s1 + s2 is validated by R1 + R2
    /// and P1 + P2. We do this by hand here rather than using the APIs to guard against regressions
    #[test]
    #[allow(non_snake_case)]
    fn test_signature_addition() {
        let mut rng = rand::thread_rng();
        // Alice and Bob generate some keys and nonces
        let (k1, P1) = RistrettoPublicKey::random_keypair(&mut rng);
        let (r1, R1) = RistrettoPublicKey::random_keypair(&mut rng);
        let (k2, P2) = RistrettoPublicKey::random_keypair(&mut rng);
        let (r2, R2) = RistrettoPublicKey::random_keypair(&mut rng);
        // Each of them creates the Challenge = H(R1 || R2 || P1 || P2 || m)
        let e = Blake256::new()
            .chain(R1.as_bytes())
            .chain(R2.as_bytes())
            .chain(P1.as_bytes())
            .chain(P2.as_bytes())
            .chain(b"Moving Pictures")
            .finalize();
        // Calculate Alice's signature
        let s1 = RistrettoSchnorr::sign(k1, r1, &e).unwrap();
        // Calculate Bob's signature
        let s2 = RistrettoSchnorr::sign(k2, r2, &e).unwrap();
        // Now add the two signatures together
        let s_agg = &s1 + &s2;
        // Check that the multi-sig verifies
        assert!(s_agg.verify_challenge(&(P1 + P2), &e));
    }

    /// Ristretto scalars have a max value 2^255. This test checks that hashed messages above this value can still be
    /// signed as a result of applying modulo arithmetic on the challenge value
    #[test]
    fn challenge_from_invalid_scalar() {
        let mut rng = rand::thread_rng();
        let m = from_hex("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();
        let k = RistrettoSecretKey::random(&mut rng);
        let r = RistrettoSecretKey::random(&mut rng);
        assert!(RistrettoSchnorr::sign(k, r, &m).is_ok());
    }
}
