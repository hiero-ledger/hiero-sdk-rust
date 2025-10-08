use crate::{
    AccountId,
    ValidateChecksums,
};

// no rename all, because each field is renamed.
// can't do boxing because `Copy`.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub(crate) enum StakedId {
    AccountId(AccountId),
    NodeId(u64),
}

impl StakedId {
    pub(crate) fn to_account_id(self) -> Option<AccountId> {
        match self {
            StakedId::AccountId(it) => Some(it),
            StakedId::NodeId(_) => None,
        }
    }

    pub(crate) fn to_node_id(self) -> Option<u64> {
        match self {
            StakedId::NodeId(it) => Some(it),
            StakedId::AccountId(_) => None,
        }
    }
}

impl ValidateChecksums for StakedId {
    fn validate_checksums(
        &self,
        ledger_id: &crate::ledger_id::RefLedgerId,
    ) -> Result<(), crate::Error> {
        self.to_account_id().validate_checksums(ledger_id)
    }
}

impl From<AccountId> for StakedId {
    fn from(v: AccountId) -> Self {
        Self::AccountId(v)
    }
}

impl From<u64> for StakedId {
    fn from(v: u64) -> Self {
        Self::NodeId(v)
    }
}

mod proto {
    use super::StakedId;
    use crate::proto::services;
    use crate::FromProtobuf;

    macro_rules! impl_from_pb {
        ($ty:ty) => {
            impl FromProtobuf<$ty> for StakedId {
                fn from_protobuf(value: $ty) -> crate::Result<Self> {
                    type PbStakedId = $ty;
                    match value {
                        PbStakedId::StakedAccountId(value) => {
                            Ok(Self::AccountId(FromProtobuf::from_protobuf(value)?))
                        }
                        PbStakedId::StakedNodeId(value) => Ok(Self::NodeId(value as u64)),
                    }
                }
            }
        };
    }

    impl_from_pb!(services::contract_create_transaction_body::StakedId);
    impl_from_pb!(services::contract_update_transaction_body::StakedId);
    impl_from_pb!(services::crypto_create_transaction_body::StakedId);
    impl_from_pb!(services::crypto_update_transaction_body::StakedId);
}
