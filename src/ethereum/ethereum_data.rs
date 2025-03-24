// SPDX-License-Identifier: Apache-2.0

use core::fmt;

use bytes::{
    BufMut,
    BytesMut,
};
use rlp::Rlp;

use crate::Error;

/// Data for an [`EthereumTransaction`](crate::EthereumTransaction).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum EthereumData {
    /// Data for a legacy ethereum transaction.
    Legacy(LegacyEthereumData),

    /// Data for an Eip 1559 ethereum transaction.
    Eip1559(Eip1559EthereumData),
}

impl EthereumData {
    pub(super) fn call_data_mut(&mut self) -> &mut Vec<u8> {
        match self {
            EthereumData::Legacy(it) => &mut it.call_data,
            EthereumData::Eip1559(it) => &mut it.call_data,
        }
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        match bytes.split_first() {
            // note: eating the 2 here involves a bit of extra work.
            Some((2, bytes)) => Eip1559EthereumData::decode_rlp(&Rlp::new(bytes))
                .map(Self::Eip1559)
                .map_err(Error::basic_parse),

            Some(_) => Ok(Self::Legacy(LegacyEthereumData::from_bytes(bytes)?)),
            None => Err(Error::basic_parse("Empty ethereum transaction data")),
        }
    }

    /// convert this data to rlp encoded bytes.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            EthereumData::Legacy(it) => it.to_bytes(),
            EthereumData::Eip1559(it) => it.to_bytes(),
        }
    }
}

/// Data for a legacy ethereum transaction.
#[derive(Clone)]
#[non_exhaustive]
pub struct LegacyEthereumData {
    /// Transaction's nonce.
    pub nonce: Vec<u8>,

    /// Price for 1 gas.
    pub gas_price: Vec<u8>,

    /// The amount of gas available for the transaction.
    pub gas_limit: Vec<u8>,

    /// The receiver of the transaction.
    pub to: Vec<u8>,

    /// The transaction value.
    pub value: Vec<u8>,

    /// The V value of the signature.
    pub v: Vec<u8>,

    /// The raw call data.
    pub call_data: Vec<u8>,

    /// The R value of the signature.
    pub r: Vec<u8>,

    /// The S value of the signature.
    pub s: Vec<u8>,
}

// manual impl of debug for the hex encoding of everything :/
impl fmt::Debug for LegacyEthereumData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { nonce, gas_price, gas_limit, to, value, v, call_data, r, s } = self;
        f.debug_struct("LegacyEthereumData")
            .field("nonce", &hex::encode(nonce))
            .field("gas_price", &hex::encode(gas_price))
            .field("gas_limit", &hex::encode(gas_limit))
            .field("to", &hex::encode(to))
            .field("value", &hex::encode(value))
            .field("v", &hex::encode(v))
            .field("call_data", &hex::encode(call_data))
            .field("r", &hex::encode(r))
            .field("s", &hex::encode(s))
            .finish()
    }
}

impl LegacyEthereumData {
    fn decode_rlp(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        if rlp.item_count()? != 9 {
            return Err(rlp::DecoderError::RlpIncorrectListLen);
        }

        Ok(Self {
            nonce: rlp.val_at(0)?,
            gas_price: rlp.val_at(1)?,
            gas_limit: rlp.val_at(2)?,
            to: rlp.val_at(3)?,
            value: rlp.val_at(4)?,
            call_data: rlp.val_at(5)?,
            v: rlp.val_at(6)?,
            r: rlp.val_at(7)?,
            s: rlp.val_at(8)?,
        })
    }

    /// Deserialize this data from rlp encoded bytes.
    ///
    /// # Errors
    /// - [`Error::BasicParse`] if decoding the bytes fails.
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        // todo: test this.
        Self::decode_rlp(&Rlp::new(bytes)).map_err(Error::basic_parse)
    }

    /// Convert this data to rlp encoded bytes.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        // todo: test this.
        let mut rlp = rlp::RlpStream::new_list(9);

        rlp.append(&self.nonce)
            .append(&self.gas_price)
            .append(&self.gas_limit)
            .append(&self.to)
            .append(&self.value)
            .append(&self.call_data)
            .append(&self.v)
            .append(&self.r)
            .append(&self.s);

        rlp.out().to_vec()
    }
}

/// Data for an Eip 1559 ethereum transaction.
#[derive(Clone)]
#[non_exhaustive]
pub struct Eip1559EthereumData {
    /// ID of the chain.
    pub chain_id: Vec<u8>,

    /// Transaction's nonce.
    pub nonce: Vec<u8>,

    /// An 'optional' additional fee in Ethereum that is paid directly to miners in order to incentivize
    /// them to include your transaction in a block. Not used in Hedera.
    pub max_priority_gas: Vec<u8>,

    /// The maximum amount, in tinybars, that the payer of the hedera transaction
    /// is willing to pay to complete the transaction.
    pub max_gas: Vec<u8>,

    /// The amount of gas available for the transaction.
    pub gas_limit: Vec<u8>,

    /// The receiver of the transaction.
    pub to: Vec<u8>,

    /// The transaction value.
    pub value: Vec<u8>,

    /// The raw call data.
    pub call_data: Vec<u8>,

    /// Specifies an array of addresses and storage keys that the transaction plans to access.
    pub access_list: Vec<Vec<u8>>,

    /// Recovery parameter used to ease the signature verification.
    pub recovery_id: Vec<u8>,

    /// The R value of the signature.
    pub r: Vec<u8>,

    /// The S value of the signature.
    pub s: Vec<u8>,
}

// manual impl of debug for the hex encoding of everything.
impl fmt::Debug for Eip1559EthereumData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct HexList<'a, T: AsRef<[u8]>>(&'a [T]);

        impl<'a, T: AsRef<[u8]>> fmt::Debug for HexList<'a, T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.0.iter().map(hex::encode)).finish()
            }
        }

        let Self {
            chain_id,
            nonce,
            max_priority_gas,
            max_gas,
            gas_limit,
            to,
            value,
            call_data,
            access_list,
            recovery_id,
            r,
            s,
        } = self;

        f.debug_struct("Eip1559EthereumData")
            .field("chain_id", &hex::encode(chain_id))
            .field("nonce", &hex::encode(nonce))
            .field("max_priority_gas", &hex::encode(max_priority_gas))
            .field("max_gas", &hex::encode(max_gas))
            .field("gas_limit", &hex::encode(gas_limit))
            .field("to", &hex::encode(to))
            .field("value", &hex::encode(value))
            .field("call_data", &hex::encode(call_data))
            .field("access_list", &HexList(access_list))
            .field("recovery_id", &hex::encode(recovery_id))
            .field("r", &hex::encode(r))
            .field("s", &hex::encode(s))
            .finish()
    }
}

impl Eip1559EthereumData {
    fn decode_rlp(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        if rlp.item_count()? != 12 {
            return Err(rlp::DecoderError::RlpIncorrectListLen);
        }

        Ok(Self {
            chain_id: rlp.val_at(0)?,
            nonce: rlp.val_at(1)?,
            max_priority_gas: rlp.val_at(2)?,
            max_gas: rlp.val_at(3)?,
            gas_limit: rlp.val_at(4)?,
            to: rlp.val_at(5)?,
            value: rlp.val_at(6)?,
            call_data: rlp.val_at(7)?,
            access_list: rlp.list_at(8)?,
            recovery_id: rlp.val_at(9)?,
            r: rlp.val_at(10)?,
            s: rlp.val_at(11)?,
        })
    }

    /// Deserialize this data from rlp encoded bytes.
    ///
    /// # Errors
    /// - [`Error::BasicParse`] if decoding the bytes fails.
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        let (&first, bytes) = bytes
            .split_first()
            .ok_or_else(|| Error::basic_parse("Empty ethereum transaction data"))?;

        if first != 2 {
            return Err(Error::basic_parse(rlp::DecoderError::Custom("Invalid kind")));
        }

        Self::decode_rlp(&Rlp::new(bytes)).map_err(Error::basic_parse)
    }

    /// Convert this data to rlp encoded bytes.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = BytesMut::new();
        buffer.put_u8(0x02);
        let mut rlp = rlp::RlpStream::new_list_with_buffer(buffer, 12);

        rlp.append(&self.chain_id)
            .append(&self.nonce)
            .append(&self.max_priority_gas)
            .append(&self.max_gas)
            .append(&self.gas_limit)
            .append(&self.to)
            .append(&self.value)
            .append(&self.call_data)
            .append_list::<Vec<_>, _>(self.access_list.as_slice())
            .append(&self.recovery_id)
            .append(&self.r)
            .append(&self.s);

        rlp.out().to_vec()
    }
}

#[cfg(test)]
mod test {
    use expect_test::expect;
    use hex_literal::hex;

    use crate::ethereum::EthereumData;
    // https://github.com/hashgraph/hedera-services/blob/1e01d9c6b8923639b41359c55413640b589c4ec7/hapi-utils/src/test/java/com/hedera/services/ethereum/EthTxDataTest.java#L49
    const RAW_TX_TYPE_0: &[u8]  =
        &hex!("f864012f83018000947e3a9eaf9bcc39e2ffa38eb30bf7a93feacbc18180827653820277a0f9fbff985d374be4a55f296915002eec11ac96f1ce2df183adf992baa9390b2fa00c1e867cc960d9c74ec2e6a662b7908ec4c8cc9f3091e886bcefbeb2290fb792");

    const RAW_TX_TYPE_2: &[u8] =
        &hex!("02f87082012a022f2f83018000947e3a9eaf9bcc39e2ffa38eb30bf7a93feacbc181880de0b6b3a764000083123456c001a0df48f2efd10421811de2bfb125ab75b2d3c44139c4642837fb1fccce911fd479a01aaf7ae92bee896651dfc9d99ae422a296bf5d9f1ca49b2d96d82b79eb112d66");

    #[test]
    fn legacy_to_from_bytes() {
        let data = EthereumData::from_bytes(RAW_TX_TYPE_0).unwrap();

        assert_eq!(hex::encode(RAW_TX_TYPE_0), hex::encode(data.to_bytes()));

        expect![[r#"
            Legacy(
                LegacyEthereumData {
                    nonce: "01",
                    gas_price: "2f",
                    gas_limit: "018000",
                    to: "7e3a9eaf9bcc39e2ffa38eb30bf7a93feacbc181",
                    value: "",
                    v: "0277",
                    call_data: "7653",
                    r: "f9fbff985d374be4a55f296915002eec11ac96f1ce2df183adf992baa9390b2f",
                    s: "0c1e867cc960d9c74ec2e6a662b7908ec4c8cc9f3091e886bcefbeb2290fb792",
                },
            )
        "#]]
        .assert_debug_eq(&data);

        // We don't currently support a way to get the ethereum hash, but we could
        // assert_eq!(hex!("9ffbd69c44cf643ed8d1e756b505e545e3b5dd3a6b5ef9da1d8eca6679706594"), data.ethereum_hash);
    }

    #[test]
    fn eip1559_to_from_bytes() {
        let data = EthereumData::from_bytes(RAW_TX_TYPE_2).unwrap();
        assert_eq!(hex::encode(RAW_TX_TYPE_2), hex::encode(data.to_bytes()));

        expect![[r#"
            Eip1559(
                Eip1559EthereumData {
                    chain_id: "012a",
                    nonce: "02",
                    max_priority_gas: "2f",
                    max_gas: "2f",
                    gas_limit: "018000",
                    to: "7e3a9eaf9bcc39e2ffa38eb30bf7a93feacbc181",
                    value: "0de0b6b3a7640000",
                    call_data: "123456",
                    access_list: [],
                    recovery_id: "01",
                    r: "df48f2efd10421811de2bfb125ab75b2d3c44139c4642837fb1fccce911fd479",
                    s: "1aaf7ae92bee896651dfc9d99ae422a296bf5d9f1ca49b2d96d82b79eb112d66",
                },
            )
        "#]]
        .assert_debug_eq(&data);
    }
}
