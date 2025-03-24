// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use time::{
    Duration,
    OffsetDateTime,
};

use crate::protobuf::ToProtobuf;
use crate::{
    AccountId,
    ContractId,
    FromProtobuf,
    Key,
    LedgerId,
    StakingInfo,
};

/// Current information on a smart contract instance.
#[derive(Debug, Clone)]
pub struct ContractInfo {
    /// ID of the contract instance, in the format used by transactions.
    pub contract_id: ContractId,

    /// ID of the cryptocurrency account owned by the contract instance,
    /// in the format used in transactions.
    pub account_id: AccountId,

    /// ID of both the contract instance and the cryptocurrency account owned by the contract
    /// instance, in the format used by Solidity.
    pub contract_account_id: String,

    /// The admin key of the contract instance.
    pub admin_key: Option<Key>,

    /// The current time at which this contract instance (and its account) is set to expire.
    pub expiration_time: Option<OffsetDateTime>,

    /// The auto renew period for this contract instance.
    pub auto_renew_period: Option<Duration>,

    /// Number of bytes of storage being used by this instance.
    pub storage: u64,

    /// The memo associated with the contract.
    pub contract_memo: String,

    /// The current balance, in tinybars.
    pub balance: u64,

    /// Whether the contract has been deleted.
    pub is_deleted: bool,

    /// ID of the an account to charge for auto-renewal of this contract.
    pub auto_renew_account_id: Option<AccountId>,

    /// The maximum number of tokens that a contract can be implicitly associated with.
    pub max_automatic_token_associations: u32,

    /// The ledger ID the response was returned from
    pub ledger_id: LedgerId,

    /// Staking metadata for this contract.
    pub staking_info: Option<StakingInfo>,
}

impl ContractInfo {
    /// Create a new `StakingInfo` from protobuf-encoded `bytes`.
    ///
    /// # Errors
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the bytes fails to produce a valid protobuf.
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the protobuf fails.
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        FromProtobuf::<services::contract_get_info_response::ContractInfo>::from_bytes(bytes)
    }

    /// Convert `self` to a protobuf-encoded [`Vec<u8>`].
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        ToProtobuf::to_bytes(self)
    }
}

impl FromProtobuf<services::response::Response> for ContractInfo {
    #[allow(deprecated)]
    fn from_protobuf(pb: services::response::Response) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let response = pb_getv!(pb, ContractGetInfo, services::response::Response);
        let info = pb_getf!(response, contract_info)?;
        Self::from_protobuf(info)
    }
}

impl FromProtobuf<services::contract_get_info_response::ContractInfo> for ContractInfo {
    #[allow(deprecated)]
    fn from_protobuf(pb: services::contract_get_info_response::ContractInfo) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let contract_id = pb_getf!(pb, contract_id)?;
        let account_id = pb_getf!(pb, account_id)?;
        let expiration_time = pb.expiration_time.map(Into::into);
        let auto_renew_period = pb.auto_renew_period.map(Into::into);
        let auto_renew_account_id = Option::from_protobuf(pb.auto_renew_account_id)?;
        let admin_key = Option::from_protobuf(pb.admin_key)?;
        let ledger_id = LedgerId::from_bytes(pb.ledger_id);
        let staking_info = Option::from_protobuf(pb.staking_info)?;

        Ok(Self {
            contract_id: ContractId::from_protobuf(contract_id)?,
            account_id: AccountId::from_protobuf(account_id)?,
            contract_account_id: pb.contract_account_id,
            is_deleted: pb.deleted,
            balance: pb.balance,
            expiration_time,
            auto_renew_period,
            auto_renew_account_id,
            contract_memo: pb.memo,
            max_automatic_token_associations: pb.max_automatic_token_associations as u32,
            admin_key,
            storage: pb.storage as u64,
            ledger_id,
            staking_info,
        })
    }
}

impl ToProtobuf for ContractInfo {
    type Protobuf = services::contract_get_info_response::ContractInfo;

    fn to_protobuf(&self) -> Self::Protobuf {
        #[allow(deprecated)]
        services::contract_get_info_response::ContractInfo {
            contract_id: Some(self.contract_id.to_protobuf()),
            account_id: Some(self.account_id.to_protobuf()),
            contract_account_id: self.contract_account_id.clone(),
            admin_key: self.admin_key.to_protobuf(),
            expiration_time: self.expiration_time.to_protobuf(),
            auto_renew_period: self.auto_renew_period.to_protobuf(),
            storage: self.storage as i64,
            memo: self.contract_memo.clone(),
            balance: self.balance,
            deleted: self.is_deleted,
            ledger_id: self.ledger_id.to_bytes(),
            auto_renew_account_id: self.auto_renew_account_id.to_protobuf(),
            max_automatic_token_associations: self.max_automatic_token_associations as i32,
            staking_info: self.staking_info.to_protobuf(),

            // unimplemented fields
            token_relationships: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use hedera_proto::services;
    use prost::Message;

    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::{
        ContractInfo,
        LedgerId,
    };

    fn make_info() -> services::contract_get_info_response::ContractInfo {
        services::contract_get_info_response::ContractInfo {
            contract_id: Some(services::ContractId {
                shard_num: 0,
                realm_num: 0,
                contract: Some(services::contract_id::Contract::ContractNum(1)),
            }),
            account_id: Some(services::AccountId {
                shard_num: 0,
                realm_num: 0,
                account: Some(services::account_id::Account::AccountNum(2)),
            }),
            contract_account_id: "0.0.3".to_owned(),
            expiration_time: Some(services::Timestamp { seconds: 0, nanos: 4_000 }),
            auto_renew_period: Some(services::Duration { seconds: 24 * 60 * 60 }),
            storage: 6,
            memo: "7".to_owned(),
            balance: 8,
            ledger_id: LedgerId::testnet().to_bytes(),

            ..Default::default()
        }
    }

    #[test]
    fn from_protobuf() {
        expect![[r#"
            ContractInfo {
                contract_id: "0.0.1",
                account_id: "0.0.2",
                contract_account_id: "0.0.3",
                admin_key: None,
                expiration_time: Some(
                    1970-01-01 0:00:00.000004 +00:00:00,
                ),
                auto_renew_period: Some(
                    Duration {
                        seconds: 86400,
                        nanoseconds: 0,
                    },
                ),
                storage: 6,
                contract_memo: "7",
                balance: 8,
                is_deleted: false,
                auto_renew_account_id: None,
                max_automatic_token_associations: 0,
                ledger_id: "testnet",
                staking_info: None,
            }
        "#]]
        .assert_debug_eq(&ContractInfo::from_protobuf(make_info()).unwrap());
    }

    #[test]
    fn to_protobuf() {
        expect![[r#"
            ContractInfo {
                contract_id: Some(
                    ContractId {
                        shard_num: 0,
                        realm_num: 0,
                        contract: Some(
                            ContractNum(
                                1,
                            ),
                        ),
                    },
                ),
                account_id: Some(
                    AccountId {
                        shard_num: 0,
                        realm_num: 0,
                        account: Some(
                            AccountNum(
                                2,
                            ),
                        ),
                    },
                ),
                contract_account_id: "0.0.3",
                admin_key: None,
                expiration_time: Some(
                    Timestamp {
                        seconds: 0,
                        nanos: 4000,
                    },
                ),
                auto_renew_period: Some(
                    Duration {
                        seconds: 86400,
                    },
                ),
                storage: 6,
                memo: "7",
                balance: 8,
                deleted: false,
                token_relationships: [],
                ledger_id: [
                    1,
                ],
                auto_renew_account_id: None,
                max_automatic_token_associations: 0,
                staking_info: None,
            }
        "#]]
        .assert_debug_eq(&ContractInfo::from_protobuf(make_info()).unwrap().to_protobuf())
    }

    #[test]
    fn from_bytes() {
        expect![[r#"
            ContractInfo {
                contract_id: "0.0.1",
                account_id: "0.0.2",
                contract_account_id: "0.0.3",
                admin_key: None,
                expiration_time: Some(
                    1970-01-01 0:00:00.000004 +00:00:00,
                ),
                auto_renew_period: Some(
                    Duration {
                        seconds: 86400,
                        nanoseconds: 0,
                    },
                ),
                storage: 6,
                contract_memo: "7",
                balance: 8,
                is_deleted: false,
                auto_renew_account_id: None,
                max_automatic_token_associations: 0,
                ledger_id: "testnet",
                staking_info: None,
            }
        "#]]
        .assert_debug_eq(&ContractInfo::from_bytes(&make_info().encode_to_vec()).unwrap());
    }
}
