// SPDX-License-Identifier: Apache-2.0

use std::borrow::Cow;
use std::str;

use hedera_proto::services;
use num_bigint::{
    BigInt,
    BigUint,
};

use crate::protobuf::ToProtobuf;
use crate::{
    AccountId,
    ContractId,
    ContractLogInfo,
    ContractNonceInfo,
    FromProtobuf,
};

/// The result returned by a call to a smart contract function.
#[derive(Debug, Clone)]
pub struct ContractFunctionResult {
    /// The smart contract instance whose function was called.
    pub contract_id: ContractId,

    /// The new contract's 20-byte EVM address.
    pub evm_address: Option<ContractId>,

    /// The raw bytes returned by the function.
    pub bytes: Vec<u8>,

    /// Message if there was an error during smart contract execution.
    pub error_message: Option<String>,

    /// Bloom filter for record.
    pub bloom: Vec<u8>,

    /// Units of gas used to execute contract.
    pub gas_used: u64,

    /// The amount of gas available for the call.
    pub gas: u64,

    /// Number of HBAR sent (the function must be payable if this is nonzero).
    pub hbar_amount: u64,

    /// The parameters passed into the contract call.
    pub contract_function_parameters_bytes: Vec<u8>,

    /// The account that is the "sender." If not present it is the accountId from the transactionId.
    pub sender_account_id: Option<AccountId>,

    /// Logs that this call and any called functions produced.
    pub logs: Vec<ContractLogInfo>,

    /// A list of updated contract account nonces containing the new nonce value for each contract account.
    /// This is always empty in a ContractLocalCallQuery response, since no internal creations can happen in a static EVM call.
    pub contract_nonces: Vec<ContractNonceInfo>,

    /// If not null this field specifies what the value of the signer account nonce is post transaction execution.
    /// For transactions that don't update the signer nonce, this field should be null.
    pub signer_nonce: Option<u64>,
}

impl ContractFunctionResult {
    const SLOT_SIZE: usize = 32;

    #[must_use]
    fn get_fixed_bytes<const N: usize>(&self, slot: usize) -> Option<&[u8; N]> {
        self.get_fixed_bytes_at(slot * Self::SLOT_SIZE + (Self::SLOT_SIZE - N))
    }

    // fixme(sr): name is weird, but I can't think of a better one.
    // basically, there's `get_fixed_bytes` which works off of "slots" (multiples of 32 bytes), and this version, which can be from anywhere.
    #[must_use]
    fn get_fixed_bytes_at<const N: usize>(&self, offset: usize) -> Option<&[u8; N]> {
        self.bytes.get(offset..).and_then(|it| it.get(..N)).map(|it| it.try_into().unwrap())
    }

    // fixme(sr): name is weird, but I can't think of a better one.
    #[must_use]
    fn get_u32_at(&self, offset: usize) -> Option<u32> {
        self.get_fixed_bytes_at(28 + offset).map(|it| u32::from_be_bytes(*it))
    }

    #[must_use]
    fn offset_len_pair(&self, offset: usize) -> Option<(usize, usize)> {
        let offset = self.get_u32(offset)? as usize;
        let len = self.get_u32_at(offset)? as usize;
        Some((offset, len))
    }

    /// Get the whole raw function result.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    // note: This would be best named `get_str_lossy` but consistency :/
    /// Get the value at `index` as a solidity `string`.
    ///
    /// Theoretically, all strings here should be utf8, but this function does _lossy_ conversion.
    #[must_use]
    pub fn get_str(&self, index: usize) -> Option<Cow<str>> {
        self.get_bytes(index).map(String::from_utf8_lossy)
    }
    /// Get the value at `index` as a solidity `string[]`.
    ///
    /// Theoretically, all strings here should be utf8, but this function does _lossy_ conversion.
    #[must_use]
    pub fn get_str_array(&self, index: usize) -> Option<Vec<Cow<str>>> {
        let (offset, len) = self.offset_len_pair(index)?;

        let mut v = Vec::with_capacity(len);
        for i in 0..len {
            let str_offset =
                self.get_u32_at(offset + Self::SLOT_SIZE + (i * Self::SLOT_SIZE))? as usize;
            let str_offset = offset + str_offset + Self::SLOT_SIZE;
            let len = self.get_u32_at(str_offset)? as usize;

            let bytes =
                self.bytes.get((str_offset + Self::SLOT_SIZE)..).and_then(|it| it.get(..len))?;

            v.push(String::from_utf8_lossy(bytes));
        }

        Some(v)
    }

    /// Get the value at `index` as solidity `bytes`.
    #[must_use]
    pub fn get_bytes(&self, index: usize) -> Option<&[u8]> {
        let (offset, len) = self.offset_len_pair(index)?;
        self.bytes.get((offset + Self::SLOT_SIZE)..).and_then(|it| it.get(..len))
    }

    /// Get the value at `index` as solidity `bytes32`.
    ///
    /// This is the native word size for the solidity ABI.
    #[must_use]
    pub fn get_bytes32(&self, index: usize) -> Option<&[u8; 32]> {
        self.get_fixed_bytes(index)
    }

    /// Get the value at `index` as a solidity `address` and then hex-encode the result.
    #[must_use]
    pub fn get_address(&self, index: usize) -> Option<String> {
        self.get_fixed_bytes::<20>(index).map(hex::encode)
    }

    /// Get the value at `index` as a solidity `bool`.
    #[must_use]
    pub fn get_bool(&self, index: usize) -> Option<bool> {
        self.get_u8(index).map(|it| it != 0)
    }

    /// Get the value at `index` as a solidity `u8`.
    #[must_use]
    pub fn get_u8(&self, index: usize) -> Option<u8> {
        self.get_fixed_bytes(index).copied().map(u8::from_be_bytes)
    }

    /// Get the value at `index` as a solidity `i8`.
    #[must_use]
    pub fn get_i8(&self, index: usize) -> Option<i8> {
        self.get_fixed_bytes(index).copied().map(i8::from_be_bytes)
    }

    /// Get the value at `index` as a solidity `u32`.
    pub fn get_u32(&self, index: usize) -> Option<u32> {
        self.get_fixed_bytes(index).copied().map(u32::from_be_bytes)
    }

    /// Get the value at `index` as a solidity `i32`.
    #[must_use]
    pub fn get_i32(&self, index: usize) -> Option<i32> {
        self.get_fixed_bytes(index).copied().map(i32::from_be_bytes)
    }

    /// Get the value at `index` as a solidity `u64`.
    #[must_use]
    pub fn get_u64(&self, index: usize) -> Option<u64> {
        self.get_fixed_bytes(index).copied().map(u64::from_be_bytes)
    }

    /// Get the value at `index` as a solidity `i64`.
    #[must_use]
    pub fn get_i64(&self, index: usize) -> Option<i64> {
        self.get_fixed_bytes(index).copied().map(i64::from_be_bytes)
    }

    /// Get the value at `index` as a solidity `u256` (`uint`).
    ///
    /// This is the native unsigned integer size for the solidity ABI.
    #[must_use]
    pub fn get_u256(&self, index: usize) -> Option<BigUint> {
        self.get_bytes32(index).map(|it| BigUint::from_bytes_be(it))
    }

    /// Get the value at `index` as a solidity `i256` (`int`).
    ///
    /// This is the native unsigned integer size for the solidity ABI.
    #[must_use]
    pub fn get_i256(&self, index: usize) -> Option<BigInt> {
        self.get_bytes32(index).map(|it| BigInt::from_signed_bytes_be(it))
    }
}

impl FromProtobuf<services::ContractFunctionResult> for ContractFunctionResult {
    fn from_protobuf(pb: services::ContractFunctionResult) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let contract_id = pb_getf!(pb, contract_id)?;
        let contract_id = ContractId::from_protobuf(contract_id)?;

        let sender_account_id = Option::from_protobuf(pb.sender_id)?;

        let evm_address =
            pb.evm_address.and_then(|address| <[u8; 20]>::try_from(address).ok()).map(|address| {
                ContractId::from_evm_address_bytes(contract_id.shard, contract_id.realm, address)
            });

        let error_message = if pb.error_message.is_empty() { None } else { Some(pb.error_message) };

        // if an exception was thrown, the call result is encoded like the params
        // for a function `Error(string)`
        // https://solidity.readthedocs.io/en/v0.6.2/control-structures.html#revert
        // `map_or` wouldn't actually work here, because `contract_call_result
        #[allow(clippy::map_unwrap_or)]
        let bytes = if error_message.is_some() {
            pb.contract_call_result
                .strip_prefix(&[0x08, 0xc3, 0x79, 0xa0])
                .map(<[u8]>::to_vec)
                .unwrap_or(pb.contract_call_result)
        } else {
            pb.contract_call_result
        };

        let signer_nonce = pb.signer_nonce.map(|it| it as u64);

        Ok(Self {
            contract_id,
            bytes,
            error_message,
            bloom: pb.bloom,
            gas_used: pb.gas_used,
            gas: pb.gas as u64,
            hbar_amount: pb.amount as u64,
            contract_function_parameters_bytes: pb.function_parameters,
            sender_account_id,
            evm_address,
            logs: Vec::from_protobuf(pb.log_info)?,
            contract_nonces: Vec::from_protobuf(pb.contract_nonces)?,
            signer_nonce,
        })
    }
}

impl FromProtobuf<services::response::Response> for ContractFunctionResult {
    fn from_protobuf(pb: services::response::Response) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let pb = pb_getv!(pb, ContractCallLocal, services::response::Response);

        let result = pb_getf!(pb, function_result)?;
        let result = ContractFunctionResult::from_protobuf(result)?;

        Ok(result)
    }
}

impl ToProtobuf for ContractFunctionResult {
    type Protobuf = services::ContractFunctionResult;

    fn to_protobuf(&self) -> Self::Protobuf {
        #[allow(deprecated)]
        services::ContractFunctionResult {
            contract_id: Some(self.contract_id.to_protobuf()),
            contract_call_result: self.bytes.clone(),
            error_message: self.error_message.clone().unwrap_or_default(),
            bloom: self.bloom.clone(),
            gas_used: self.gas,
            log_info: self.logs.to_protobuf(),
            created_contract_i_ds: Vec::new(),
            evm_address: self.evm_address.and_then(|it| it.evm_address.map(|it| it.to_vec())),
            gas: self.gas as i64,
            amount: self.hbar_amount as i64,
            function_parameters: self.contract_function_parameters_bytes.clone(),
            sender_id: self.sender_account_id.to_protobuf(),
            contract_nonces: self.contract_nonces.to_protobuf(),
            signer_nonce: self.signer_nonce.map(|it| it as i64),
        }
    }
}

#[cfg(test)]
mod tests {
    use hedera_proto::services;
    use hex_literal::hex;
    use num_bigint::{
        BigInt,
        BigUint,
    };

    use crate::protobuf::{
        FromProtobuf,
        ToProtobuf,
    };
    use crate::{
        AccountId,
        ContractFunctionResult,
        ContractId,
        ContractNonceInfo,
    };

    const CALL_RESULT: [u8; 320] = hex!(
        "00000000000000000000000000000000000000000000000000000000ffffffff"
        "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
        "00000000000000000000000011223344556677889900aabbccddeeff00112233"
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
        "00000000000000000000000000000000000000000000000000000000000000c0"
        "0000000000000000000000000000000000000000000000000000000000000100"
        "000000000000000000000000000000000000000000000000000000000000000d"
        "48656c6c6f2c20776f726c642100000000000000000000000000000000000000"
        "0000000000000000000000000000000000000000000000000000000000000014"
        "48656c6c6f2c20776f726c642c20616761696e21000000000000000000000000"
    );

    const STRING_ARRAY_RESULT: [u8; 256] = hex!(
        "0000000000000000000000000000000000000000000000000000000000000020"
        "0000000000000000000000000000000000000000000000000000000000000002"
        "0000000000000000000000000000000000000000000000000000000000000040"
        "0000000000000000000000000000000000000000000000000000000000000080"
        "000000000000000000000000000000000000000000000000000000000000000C"
        "72616E646F6D2062797465730000000000000000000000000000000000000000"
        "000000000000000000000000000000000000000000000000000000000000000E"
        "72616E646F6D2062797465732032000000000000000000000000000000000000"
    );

    // previous one, just offset by a bit, to ensure the logic works.
    // notes below, where `slot` is just an offset at a multiple of 32 bytes.
    const STRING_ARRAY_RESULT_2: [u8; 320] = hex!(
        // empty value at slot 0
        "0000000000000000000000000000000000000000000000000000000000000000"
        // reference to slot 3 at slot 1
        // this is interpreted as a string[]
        "0000000000000000000000000000000000000000000000000000000000000060"
        // empty value at slot 2
        "0000000000000000000000000000000000000000000000000000000000000000"
        // length of string (2 items) at slot 3
        "0000000000000000000000000000000000000000000000000000000000000002"
        // relative offset of strings[0] (2 slots) at slot 4
        "0000000000000000000000000000000000000000000000000000000000000040"
        // relative offset of strings[1] (4 slots) at slot 5
        "0000000000000000000000000000000000000000000000000000000000000080"
        // length of strings[0] (12 bytes) at slot 6
        "000000000000000000000000000000000000000000000000000000000000000c"
        // first 12 bytes: value of strings[0], rest is filler, at slot 7
        "72616e646f6d206279746573000000000000000000000000c0ffee0000000000"
        // length of strings[1] (14 bytes) at slot 8
        "000000000000000000000000000000000000000000000000000000000000000e"
        // first 14 bytes: value of strings[0], rest is filler, at slot 7
        "72616E646F6D2062797465732032000000000000decaff000000000000000000"
    );

    #[test]
    fn evm_address() {
        const EVM_ADDRESS: [u8; 20] = hex!("98329e006610472e6b372c080833f6d79ed833cf");
        let result = services::ContractFunctionResult {
            contract_id: Some(ContractId::new(3, 7, 13).to_protobuf()),
            evm_address: Some(EVM_ADDRESS.to_vec()),
            ..Default::default()
        };

        let result = ContractFunctionResult::from_protobuf(result).unwrap();

        assert_eq!(result.contract_id, ContractId::new(3, 7, 13));

        // ensure that we follow *Java* behavior (every SDK has different behavior here)
        assert_eq!(result.evm_address, Some(ContractId::from_evm_address_bytes(3, 7, EVM_ADDRESS)));
    }

    #[test]
    #[allow(deprecated)]
    fn provides_results() {
        let result = services::ContractFunctionResult {
            contract_id: Some(ContractId::from(3).to_protobuf()),
            contract_call_result: CALL_RESULT.to_vec(),
            sender_id: Some(
                AccountId {
                    shard: 31,
                    realm: 41,
                    num: 65,
                    alias: None,
                    evm_address: None,
                    checksum: None,
                }
                .to_protobuf(),
            ),
            contract_nonces: vec![services::ContractNonceInfo {
                contract_id: Some(services::ContractId {
                    shard_num: 1,
                    realm_num: 2,
                    contract: Some(services::contract_id::Contract::ContractNum(3)),
                }),
                nonce: 10,
            }],
            ..Default::default()
        };

        let result = ContractFunctionResult::from_protobuf(result).unwrap();

        assert_eq!(result.get_bool(0).unwrap(), true);
        assert_eq!(result.get_i32(0).unwrap(), -1);
        assert_eq!(result.get_i64(0).unwrap(), u32::MAX as u64 as i64);
        assert_eq!(result.get_i256(0).unwrap(), BigInt::from(u32::MAX));
        assert_eq!(result.get_i256(1).unwrap(), (BigInt::from(1) << 255) - 1);
        assert_eq!(&result.get_address(2).unwrap(), "11223344556677889900aabbccddeeff00112233");
        assert_eq!(result.get_u32(3).unwrap(), u32::MAX);
        assert_eq!(result.get_u64(3).unwrap(), u64::MAX);
        // BigInteger can represent the full range and so should be 2^256 - 1
        assert_eq!(result.get_u256(3).unwrap(), (BigUint::from(1_u8) << 256) - 1_u32);

        assert_eq!(result.get_str(4).unwrap(), "Hello, world!");
        assert_eq!(result.get_str(5).unwrap(), "Hello, world, again!");

        assert_eq!(
            result.sender_account_id,
            Some(AccountId {
                shard: 31,
                realm: 41,
                num: 65,
                alias: None,
                evm_address: None,
                checksum: None,
            })
        );

        assert_eq!(
            result.contract_nonces,
            [ContractNonceInfo {
                contract_id: ContractId {
                    shard: 1,
                    realm: 2,
                    num: 3,
                    checksum: None,
                    evm_address: None
                },
                nonce: 10
            }]
        )
    }

    #[test]
    fn str_array_results() {
        let result = services::ContractFunctionResult {
            contract_id: Some(ContractId::from(3).to_protobuf()),
            contract_call_result: STRING_ARRAY_RESULT.to_vec(),
            ..Default::default()
        };

        let result = ContractFunctionResult::from_protobuf(result).unwrap();

        let strings = result.get_str_array(0).unwrap();
        assert_eq!(strings[0], "random bytes");
        assert_eq!(strings[1], "random bytes 2")
    }

    // previous one, just offset by a bit, to ensure the logic works.
    #[test]
    fn str_array_results2() {
        let result = services::ContractFunctionResult {
            contract_id: Some(ContractId::from(3).to_protobuf()),
            contract_call_result: STRING_ARRAY_RESULT_2.to_vec(),
            ..Default::default()
        };

        let result = ContractFunctionResult::from_protobuf(result).unwrap();

        let strings = result.get_str_array(1).unwrap();
        assert_eq!(strings[0], "random bytes");
        assert_eq!(strings[1], "random bytes 2")
    }
}
