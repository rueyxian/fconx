///
pub(crate) struct Sha1Hasher {
    //
    hasher: crypto::sha1::Sha1,
}

impl Sha1Hasher {
    const BYTES_LEN: usize = 2048;

    ///
    pub(crate) fn new() -> Sha1Hasher {
        Sha1Hasher {
            hasher: crypto::sha1::Sha1::new(),
        }
    }

    pub(crate) fn create_sha1(&mut self, bytes: &[u8]) -> String {
        use crypto::digest::Digest;
        let sha1 = {
            let idx = usize::min(Sha1Hasher::BYTES_LEN, bytes.len());
            self.hasher.input(&bytes[..idx]);
            self.hasher.result_str()
        };
        self.hasher.reset();
        sha1
    }
}
