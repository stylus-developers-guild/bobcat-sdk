use bobcat_maths::U;

use ed25519_dalek::{Signature, VerifyingKey};

use sha2::digest::{
    Digest, FixedOutput, FixedOutputReset, Output, OutputSizeUser, Reset, Update, consts::U64,
};

#[derive(Debug, Clone, Copy)]
pub struct PrecomputedSha512([u8; 64]);

impl OutputSizeUser for PrecomputedSha512 {
    type OutputSize = U64;
}

impl Update for PrecomputedSha512 {
    fn update(&mut self, data: &[u8]) {
        let len = data.len().min(64);
        self.0[..len].copy_from_slice(&data[..len]);
    }
}

impl FixedOutput for PrecomputedSha512 {
    fn finalize_into(self, out: &mut Output<Self>) {
        *out = Output::<Self>::from(self.0);
    }
}

impl Reset for PrecomputedSha512 {
    fn reset(&mut self) {
        self.0 = [0u8; 64];
    }
}

impl FixedOutputReset for PrecomputedSha512 {
    fn finalize_into_reset(&mut self, out: &mut Output<Self>) {
        *out = Output::<Self>::from(self.0);
        Reset::reset(self);
    }
}

impl Digest for PrecomputedSha512 {
    fn new() -> Self {
        PrecomputedSha512([0u8; 64])
    }

    fn new_with_prefix(_data: impl AsRef<[u8]>) -> Self {
        unimplemented!()
    }

    fn update(&mut self, data: impl AsRef<[u8]>) {
        Update::update(self, data.as_ref());
    }

    fn chain_update(self, _data: impl AsRef<[u8]>) -> Self {
        unimplemented!()
    }

    fn finalize(self) -> Output<Self> {
        Output::<Self>::from(self.0)
    }

    fn finalize_into(self, out: &mut Output<Self>) {
        FixedOutput::finalize_into(self, out);
    }

    fn finalize_reset(&mut self) -> Output<Self>
    where
        Self: FixedOutputReset,
    {
        let result = Output::<Self>::from(self.0);
        Reset::reset(self);
        result
    }

    fn finalize_into_reset(&mut self, out: &mut Output<Self>)
    where
        Self: FixedOutputReset,
    {
        FixedOutputReset::finalize_into_reset(self, out);
    }

    fn reset(&mut self)
    where
        Self: Reset,
    {
        Reset::reset(self);
    }

    fn output_size() -> usize {
        64
    }

    fn digest(data: impl AsRef<[u8]>) -> Output<Self> {
        let bytes = data.as_ref();
        let mut result = [0u8; 64];
        let len = bytes.len().min(64);
        result[..len].copy_from_slice(&bytes[..len]);
        Output::<Self>::from(result)
    }
}

pub fn const_edphverify(digest: [u8; 64], pub_key: U, sig: [u8; 64]) -> bool {
    let digest = PrecomputedSha512(digest);
    let key = VerifyingKey::from_bytes(&pub_key.0).unwrap();
    let sig = Signature::from_bytes(&sig);
    key.verify_prehashed_strict(digest, None, &sig).is_ok()
}
