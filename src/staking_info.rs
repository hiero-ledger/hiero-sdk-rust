use time::OffsetDateTime;

use crate::proto::services;
use crate::protobuf::ToProtobuf;
use crate::{
    AccountId,
    FromProtobuf,
    Hbar,
};

// todo(sr): is this right?
/// Info related to account/contract staking settings.
#[derive(Debug, Clone)]
pub struct StakingInfo {
    /// If `true`, the contract declines receiving a staking reward. The default value is `false`.
    pub decline_staking_reward: bool,

    /// The staking period during which either the staking settings for this account or contract changed (such as starting
    /// staking or changing staked_node_id) or the most recent reward was earned, whichever is later. If this account or contract
    /// is not currently staked to a node, then this field is not set.
    pub stake_period_start: Option<OffsetDateTime>,

    /// The amount in `Hbar` that will be received in the next reward situation.
    pub pending_reward: Hbar,

    /// The total of balance of all accounts staked to this account or contract.
    pub staked_to_me: Hbar,

    /// The account to which this account or contract is staking.
    pub staked_account_id: Option<AccountId>,

    /// The ID of the node this account or contract is staked to.
    pub staked_node_id: Option<u64>,
}

impl StakingInfo {
    /// Create a new `StakingInfo` from protobuf-encoded `bytes`.
    ///
    /// # Errors
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the bytes fails to produce a valid protobuf.
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the protobuf fails.
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        FromProtobuf::from_bytes(bytes)
    }

    /// Convert `self` to a protobuf-encoded [`Vec<u8>`].
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        ToProtobuf::to_bytes(self)
    }
}

impl FromProtobuf<services::StakingInfo> for StakingInfo {
    fn from_protobuf(pb: services::StakingInfo) -> crate::Result<Self> {
        let (staked_account_id, staked_node_id) = match pb.staked_id {
            Some(services::staking_info::StakedId::StakedAccountId(id)) => {
                (Some(AccountId::from_protobuf(id)?), None)
            }

            Some(services::staking_info::StakedId::StakedNodeId(id)) => (None, Some(id as u64)),

            None => (None, None),
        };

        Ok(Self {
            decline_staking_reward: pb.decline_reward,
            stake_period_start: pb.stake_period_start.map(Into::into),
            pending_reward: Hbar::from_tinybars(pb.pending_reward),
            staked_to_me: Hbar::from_tinybars(pb.staked_to_me),
            staked_account_id,
            staked_node_id,
        })
    }
}

impl ToProtobuf for StakingInfo {
    type Protobuf = services::StakingInfo;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::StakingInfo {
            decline_reward: self.decline_staking_reward,
            stake_period_start: self.stake_period_start.map(Into::into),
            pending_reward: self.pending_reward.to_tinybars(),
            staked_to_me: self.staked_to_me.to_tinybars(),
            staked_id: self
                .staked_node_id
                .map(|it| it as i64)
                .map(services::staking_info::StakedId::StakedNodeId)
                .or_else(|| {
                    self.staked_account_id
                        .to_protobuf()
                        .map(services::staking_info::StakedId::StakedAccountId)
                }),
        }
    }
}
