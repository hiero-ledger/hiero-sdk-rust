// SPDX-License-Identifier: Apache-2.0

use hedera_proto::mirror;

use crate::fee_estimate::{
    fee_estimate::FeeEstimate,
    fee_estimate_mode::FeeEstimateMode,
    fee_estimate_response::FeeEstimateResponse,
    fee_extra::FeeExtra,
    network_fee::NetworkFee,
};
use crate::protobuf::FromProtobuf;

// Re-export for use in fee_estimate_query
pub(super) use crate::protobuf::FromProtobuf as _;

impl FromProtobuf<mirror::FeeEstimateResponse> for FeeEstimateResponse {
    fn from_protobuf(pb: mirror::FeeEstimateResponse) -> crate::Result<Self> {
        Ok(Self {
            mode: FeeEstimateMode::from_value(pb.mode as u32)
                .ok_or_else(|| crate::Error::from_protobuf("Invalid EstimateMode value"))?,
            network: NetworkFee::from_protobuf(
                pb.network.ok_or_else(|| crate::Error::from_protobuf("Missing network fee"))?,
            )?,
            node: FeeEstimate::from_protobuf(
                pb.node.ok_or_else(|| crate::Error::from_protobuf("Missing node fee"))?,
            )?,
            notes: pb.notes,
            service: FeeEstimate::from_protobuf(
                pb.service.ok_or_else(|| crate::Error::from_protobuf("Missing service fee"))?,
            )?,
            total: pb.total,
        })
    }
}

impl FromProtobuf<mirror::NetworkFee> for NetworkFee {
    fn from_protobuf(pb: mirror::NetworkFee) -> crate::Result<Self> {
        Ok(Self { multiplier: pb.multiplier as u32, subtotal: pb.subtotal })
    }
}

impl FromProtobuf<mirror::FeeEstimate> for FeeEstimate {
    fn from_protobuf(pb: mirror::FeeEstimate) -> crate::Result<Self> {
        Ok(Self {
            base: pb.base,
            extras: pb
                .extras
                .into_iter()
                .map(FeeExtra::from_protobuf)
                .collect::<crate::Result<Vec<_>>>()?,
        })
    }
}

impl FromProtobuf<mirror::FeeExtra> for FeeExtra {
    fn from_protobuf(pb: mirror::FeeExtra) -> crate::Result<Self> {
        Ok(Self {
            charged: pb.charged as u32,
            count: pb.count as u32,
            fee_per_unit: pb.fee_per_unit,
            included: pb.included as u32,
            name: pb.name,
            subtotal: pb.subtotal,
        })
    }
}

