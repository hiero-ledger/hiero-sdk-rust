// SPDX-License-Identifier: Apache-2.0

use hedera_proto::services;
use hedera_proto::services::util_service_client::UtilServiceClient;

use crate::entity_id::ValidateChecksums;
use crate::protobuf::{
    FromProtobuf,
    ToProtobuf,
};
use crate::transaction::{
    AnyTransactionData,
    ChunkInfo,
    ToSchedulableTransactionDataProtobuf,
    ToTransactionDataProtobuf,
    TransactionData,
    TransactionExecute,
};
use crate::Transaction;

/// Random Number Generator Transaction.
pub type PrngTransaction = Transaction<PrngTransactionData>;

#[derive(Debug, Clone, Default)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct PrngTransactionData {
    range: Option<u32>,
}

impl PrngTransaction {
    /// Returns the upper-bound for the random number.
    pub fn get_range(&self) -> Option<u32> {
        self.data().range
    }

    /// Sets the upper-bound for the random number.
    ///
    /// If the value is zero, instead of returning a 32-bit number, a 384-bit number will be returned.
    pub fn range(&mut self, range: u32) -> &mut Self {
        self.data_mut().range = Some(range);

        self
    }
}

impl FromProtobuf<services::UtilPrngTransactionBody> for PrngTransactionData {
    fn from_protobuf(pb: services::UtilPrngTransactionBody) -> crate::Result<Self> {
        Ok(Self { range: (pb.range != 0).then_some(pb.range as u32) })
    }
}

impl ToProtobuf for PrngTransactionData {
    type Protobuf = services::UtilPrngTransactionBody;
    fn to_protobuf(&self) -> Self::Protobuf {
        services::UtilPrngTransactionBody { range: self.range.unwrap_or_default() as i32 }
    }
}

impl TransactionData for PrngTransactionData {}

impl From<PrngTransactionData> for AnyTransactionData {
    fn from(value: PrngTransactionData) -> Self {
        Self::Prng(value)
    }
}

impl ValidateChecksums for PrngTransactionData {
    fn validate_checksums(&self, _ledger_id: &crate::ledger_id::RefLedgerId) -> crate::Result<()> {
        Ok(())
    }
}

impl ToSchedulableTransactionDataProtobuf for PrngTransactionData {
    fn to_schedulable_transaction_data_protobuf(
        &self,
    ) -> services::schedulable_transaction_body::Data {
        services::schedulable_transaction_body::Data::UtilPrng(self.to_protobuf())
    }
}

impl ToTransactionDataProtobuf for PrngTransactionData {
    fn to_transaction_data_protobuf(
        &self,
        chunk_info: &ChunkInfo,
    ) -> services::transaction_body::Data {
        let _ = chunk_info.assert_single_transaction();

        services::transaction_body::Data::UtilPrng(self.to_protobuf())
    }
}

impl TransactionExecute for PrngTransactionData {
    fn execute(
        &self,
        channel: tonic::transport::Channel,
        request: services::Transaction,
    ) -> crate::BoxGrpcFuture<'_, services::TransactionResponse> {
        Box::pin(async { UtilServiceClient::new(channel).prng(request).await })
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::transaction::test_helpers::{
        check_body,
        transaction_body,
    };
    use crate::{
        AnyTransaction,
        PrngTransaction,
    };

    fn make_transaction() -> PrngTransaction {
        let mut tx = PrngTransaction::new_for_tests();

        tx.freeze().unwrap();

        tx
    }

    fn make_transaction2() -> PrngTransaction {
        let mut tx = PrngTransaction::new_for_tests();

        tx.range(100).freeze().unwrap();

        tx
    }

    #[test]
    fn serialize() {
        let tx = make_transaction();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect![[r#"
            UtilPrng(
                UtilPrngTransactionBody {
                    range: 0,
                },
            )
        "#]]
        .assert_debug_eq(&tx)
    }

    #[test]
    fn to_from_bytes() {
        let tx = make_transaction();

        let tx2 = AnyTransaction::from_bytes(&tx.to_bytes().unwrap()).unwrap();

        let tx = transaction_body(tx);

        let tx2 = transaction_body(tx2);

        assert_eq!(tx, tx2);
    }

    #[test]
    fn serialize2() {
        let tx = make_transaction2();

        let tx = transaction_body(tx);

        let tx = check_body(tx);

        expect![[r#"
            UtilPrng(
                UtilPrngTransactionBody {
                    range: 100,
                },
            )
        "#]]
        .assert_debug_eq(&tx)
    }

    #[test]
    fn to_from_bytes2() {
        let tx = make_transaction2();

        let tx2 = AnyTransaction::from_bytes(&tx.to_bytes().unwrap()).unwrap();

        let tx = transaction_body(tx);

        let tx2 = transaction_body(tx2);

        assert_eq!(tx, tx2);
    }
}
