// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use crate::{
    AccountId,
    PublicKey,
    TransactionId,
};

pub struct SignatureMap(
    pub HashMap<AccountId, HashMap<TransactionId, HashMap<PublicKey, Vec<u8>>>>,
);

impl SignatureMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert_signature(
        &mut self,
        node_id: AccountId,
        transaction_id: TransactionId,
        public_key: PublicKey,
        signature: Vec<u8>,
    ) {
        self.0
            .entry(node_id)
            .or_insert_with(HashMap::new)
            .entry(transaction_id)
            .or_insert_with(HashMap::new)
            .insert(public_key, signature);
    }

    pub fn remove(
        &mut self,
        account_id: &AccountId,
        transaction_id: &TransactionId,
        public_key: &PublicKey,
    ) -> Option<Vec<u8>> {
        self.0
            .get_mut(account_id)
            .and_then(|tx_map| tx_map.get_mut(transaction_id))
            .and_then(|sig_map| sig_map.remove(public_key))
    }

    pub fn remove_transaction(
        &mut self,
        account_id: &AccountId,
        transaction_id: &TransactionId,
    ) -> Option<HashMap<PublicKey, Vec<u8>>> {
        self.0.get_mut(account_id).and_then(|tx_map| tx_map.remove(transaction_id))
    }
}
