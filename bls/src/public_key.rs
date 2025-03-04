use blst::min_pk::{AggregatePublicKey as RawAggregatePublicKey, PublicKey as RawPublicKey};
use derive_more::From;

use crate::{Error, PublicKeyBytes};

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug, From)]
pub struct PublicKey(RawPublicKey);

impl From<PublicKey> for PublicKeyBytes {
    #[inline]
    fn from(public_key: PublicKey) -> Self {
        Self(public_key.as_raw().compress())
    }
}

impl TryFrom<PublicKeyBytes> for PublicKey {
    type Error = Error;

    #[inline]
    fn try_from(bytes: PublicKeyBytes) -> Result<Self, Self::Error> {
        let raw = RawPublicKey::uncompress(bytes.as_bytes())?;

        // This is needed to pass `fast_aggregate_verify` tests.
        // See the following for more information:
        // - <https://github.com/supranational/blst/issues/11>
        // - <https://github.com/ethereum/consensus-specs/releases/tag/v1.0.0>
        raw.validate()?;

        Ok(Self(raw))
    }
}

impl PublicKey {
    /// [`eth_aggregate_pubkeys`](https://github.com/ethereum/consensus-specs/blob/86fb82b221474cc89387fa6436806507b3849d88/specs/altair/bls.md#eth_aggregate_pubkeys)
    pub fn aggregate_nonempty(public_keys: impl IntoIterator<Item = Self>) -> Result<Self, Error> {
        public_keys
            .into_iter()
            .reduce(Self::aggregate)
            .ok_or(Error::NoPublicKeysToAggregate)
    }

    #[inline]
    #[must_use]
    pub fn aggregate(mut self, other: Self) -> Self {
        self.aggregate_in_place(other);
        self
    }

    #[inline]
    pub fn aggregate_in_place(&mut self, other: Self) {
        let mut self_aggregate = RawAggregatePublicKey::from_public_key(self.as_raw());
        let other_aggregate = RawAggregatePublicKey::from_public_key(other.as_raw());
        self_aggregate.add_aggregate(&other_aggregate);
        self.0 = self_aggregate.to_public_key();
    }

    pub(crate) const fn as_raw(&self) -> &RawPublicKey {
        &self.0
    }
}
