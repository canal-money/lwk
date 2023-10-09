use jade::lock_jade::LockJade;
use software_signer::Signer;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Software(#[from] software_signer::SignError),

    #[error(transparent)]
    Jade(#[from] jade::sign_pset::Error),
}

pub trait Sign {
    /// Try to sign the given pset, mutating it in place.
    /// returns how many signatures were added
    fn sign(&self, pset: &mut elements::pset::PartiallySignedTransaction) -> Result<u32, Error>;
}

impl Sign for LockJade {
    fn sign(&self, pset: &mut elements::pset::PartiallySignedTransaction) -> Result<u32, Error> {
        Ok(self.sign_pset(pset)?)
    }
}

impl<'a> Sign for Signer<'a> {
    fn sign(&self, pset: &mut elements::pset::PartiallySignedTransaction) -> Result<u32, Error> {
        Ok(self.sign_pset(pset)?)
    }
}