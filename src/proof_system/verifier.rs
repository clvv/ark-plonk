// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//
// Copyright (c) ZK-INFRA. All rights reserved.

//! Verifier-side of the PLONK Proving System

use crate::{constraint_system::StandardComposer, circuit::BlindingRandomness};
use crate::error::Error;
use crate::proof_system::widget::VerifierKey as PlonkVerifierKey;
use crate::proof_system::Proof;
use crate::transcript::TranscriptWrapper;
use ark_ec::{PairingEngine, TEModelParameters};
use ark_poly_commit::kzg10::{Powers, VerifierKey};

/// Abstraction structure designed verify [`Proof`]s.
pub struct Verifier<E, P>
where
    E: PairingEngine,
    P: TEModelParameters<BaseField = E::Fr>,
{
    /// VerificationKey which is used to verify a specific PLONK circuit
    pub verifier_key: Option<PlonkVerifierKey<E, P>>,

    /// Circuit Description
    pub(crate) cs: StandardComposer<E, P>,

    /// Store the messages exchanged during the preprocessing stage.
    ///
    /// This is copied each time, we make a proof, so that we can use the same
    /// verifier to verify multiple proofs from the same circuit. If this is
    /// not copied, then the verification procedure will modify the transcript,
    /// making it unusable for future proofs.
    pub preprocessed_transcript: TranscriptWrapper<E>,
}

impl<E, P> Verifier<E, P>
where
    E: PairingEngine,
    P: TEModelParameters<BaseField = E::Fr>,
{
    /// Creates a new `Verifier` instance.
    pub fn new(label: &'static [u8]) -> Self {
        Self {
            verifier_key: None,
            cs: StandardComposer::new(),
            preprocessed_transcript: TranscriptWrapper::new(label),
        }
    }

    /// Creates a new `Verifier` instance with some expected size.
    pub fn with_expected_size(label: &'static [u8], size: usize) -> Self {
        Self {
            verifier_key: None,
            cs: StandardComposer::with_expected_size(size),
            preprocessed_transcript: TranscriptWrapper::new(label),
        }
    }

    /// Returns the number of gates in the circuit.
    pub fn circuit_size(&self) -> usize {
        self.cs.circuit_size()
    }

    /// Returns a mutable copy of the underlying composer.
    pub fn mut_cs(&mut self) -> &mut StandardComposer<E, P> {
        &mut self.cs
    }

    /// Preprocess a circuit to obtain a [`VerifierKey`] and a circuit
    /// descriptor so that the `Verifier` instance can verify [`Proof`]s
    /// for this circuit descriptor instance.
    pub fn preprocess(&mut self, commit_key: &Powers<E>) -> Result<(), Error> {
        let br = BlindingRandomness::default();
        self.preprocess_wbr(commit_key, &br)
    }

    /// Preprocess, with blinding randomness `br`, a circuit to obtain a [`VerifierKey`] and a
    /// circuit descriptor so that the `Verifier` instance can verify [`Proof`]s for this circuit
    /// descriptor instance.
    pub fn preprocess_wbr(&mut self, commit_key: &Powers<E>, br: &BlindingRandomness<E::Fr>) -> Result<(), Error> {
        let vk = self.cs.preprocess_verifier(
            commit_key,
            &mut self.preprocessed_transcript,
            br
        )?;

        self.verifier_key = Some(vk);
        Ok(())
    }

    /// Keys the [`Transcript`] with additional seed information
    /// Wrapper around [`Transcript::append_message`].
    ///
    /// [`Transcript`]: merlin::Transcript
    /// [`Transcript::append_message`]: merlin::Transcript::append_message
    pub fn key_transcript(&mut self, label: &'static [u8], message: &[u8]) {
        self.preprocessed_transcript
            .transcript
            .append_message(label, message);
    }

    /// Verifies a [`Proof`] using `pc_verifier_key` and `public_inputs`.
    pub fn verify(
        &self,
        proof: &Proof<E, P>,
        pc_verifier_key: &VerifierKey<E>,
        public_inputs: &[E::Fr],
    ) -> Result<(), Error> {
        proof.verify(
            self.verifier_key.as_ref().unwrap(),
            &mut self.preprocessed_transcript.clone(),
            pc_verifier_key,
            public_inputs,
        )
    }
}

impl<E, P> Default for Verifier<E, P>
where
    E: PairingEngine,
    P: TEModelParameters<BaseField = E::Fr>,
{
    #[inline]
    fn default() -> Verifier<E, P> {
        Verifier::new(b"plonk")
    }
}
