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

    /// Data for an EIP-7702 ethereum transaction (type 4).
    Eip7702(Eip7702EthereumData),
}

impl EthereumData {
    pub(super) fn call_data_mut(&mut self) -> &mut Vec<u8> {
        match self {
            EthereumData::Legacy(it) => &mut it.call_data,
            EthereumData::Eip1559(it) => &mut it.call_data,
            EthereumData::Eip7702(it) => &mut it.call_data,
        }
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        match bytes.split_first() {
            // note: eating the 2 here involves a bit of extra work.
            Some((2, bytes)) => Eip1559EthereumData::decode_rlp(&Rlp::new(bytes))
                .map(Self::Eip1559)
                .map_err(Error::basic_parse),

            Some((4, bytes)) => Eip7702EthereumData::decode_rlp(&Rlp::new(bytes))
                .map(Self::Eip7702)
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
            EthereumData::Eip7702(it) => it.to_bytes(),
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
    /// them to include your transaction in a block. Not used in Hiero.
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

/// An EIP-7702 authorization entry, representing a delegation from an EOA to a contract address.
#[derive(Clone)]
#[non_exhaustive]
pub struct Authorization {
    /// The chain ID for which this authorization is valid.
    pub chain_id: Vec<u8>,

    /// The contract address to delegate to.
    pub address: Vec<u8>,

    /// The nonce of the authorizing account.
    pub nonce: Vec<u8>,

    /// The Y parity of the signature.
    pub y_parity: Vec<u8>,

    /// The R value of the signature.
    pub r: Vec<u8>,

    /// The S value of the signature.
    pub s: Vec<u8>,
}

impl fmt::Debug for Authorization {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { chain_id, address, nonce, y_parity, r, s } = self;
        f.debug_struct("Authorization")
            .field("chain_id", &hex::encode(chain_id))
            .field("address", &hex::encode(address))
            .field("nonce", &hex::encode(nonce))
            .field("y_parity", &hex::encode(y_parity))
            .field("r", &hex::encode(r))
            .field("s", &hex::encode(s))
            .finish()
    }
}

impl Authorization {
    fn decode_rlp(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        if rlp.item_count()? != 6 {
            return Err(rlp::DecoderError::RlpIncorrectListLen);
        }

        Ok(Self {
            chain_id: rlp.val_at(0)?,
            address: rlp.val_at(1)?,
            nonce: rlp.val_at(2)?,
            y_parity: rlp.val_at(3)?,
            r: rlp.val_at(4)?,
            s: rlp.val_at(5)?,
        })
    }

    fn encode_rlp(&self, rlp: &mut rlp::RlpStream) {
        rlp.begin_list(6);
        rlp.append(&self.chain_id)
            .append(&self.address)
            .append(&self.nonce)
            .append(&self.y_parity)
            .append(&self.r)
            .append(&self.s);
    }
}

/// Data for an EIP-7702 ethereum transaction (type 4).
///
/// EIP-7702 introduces EOA code delegation, allowing externally owned accounts to
/// temporarily delegate their code execution to a smart contract address.
#[derive(Clone)]
#[non_exhaustive]
pub struct Eip7702EthereumData {
    /// ID of the chain.
    pub chain_id: Vec<u8>,

    /// Transaction's nonce.
    pub nonce: Vec<u8>,

    /// An 'optional' additional fee in Ethereum that is paid directly to miners in order to incentivize
    /// them to include your transaction in a block. Not used in Hiero.
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

    /// The list of EIP-7702 authorizations.
    pub authorization_list: Vec<Authorization>,

    /// Recovery parameter used to ease the signature verification.
    pub recovery_id: Vec<u8>,

    /// The R value of the signature.
    pub r: Vec<u8>,

    /// The S value of the signature.
    pub s: Vec<u8>,
}

impl fmt::Debug for Eip7702EthereumData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
            authorization_list,
            recovery_id,
            r,
            s,
        } = self;

        struct HexList<'a, T: AsRef<[u8]>>(&'a [T]);

        impl<T: AsRef<[u8]>> fmt::Debug for HexList<'_, T> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.0.iter().map(hex::encode)).finish()
            }
        }

        f.debug_struct("Eip7702EthereumData")
            .field("chain_id", &hex::encode(chain_id))
            .field("nonce", &hex::encode(nonce))
            .field("max_priority_gas", &hex::encode(max_priority_gas))
            .field("max_gas", &hex::encode(max_gas))
            .field("gas_limit", &hex::encode(gas_limit))
            .field("to", &hex::encode(to))
            .field("value", &hex::encode(value))
            .field("call_data", &hex::encode(call_data))
            .field("access_list", &HexList(access_list))
            .field("authorization_list", authorization_list)
            .field("recovery_id", &hex::encode(recovery_id))
            .field("r", &hex::encode(r))
            .field("s", &hex::encode(s))
            .finish()
    }
}

impl Eip7702EthereumData {
    fn decode_rlp(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        if rlp.item_count()? != 13 {
            return Err(rlp::DecoderError::RlpIncorrectListLen);
        }

        let auth_list_rlp = rlp.at(9)?;
        let auth_count = auth_list_rlp.item_count()?;
        let mut authorization_list = Vec::with_capacity(auth_count);
        for i in 0..auth_count {
            authorization_list.push(Authorization::decode_rlp(&auth_list_rlp.at(i)?)?);
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
            authorization_list,
            recovery_id: rlp.val_at(10)?,
            r: rlp.val_at(11)?,
            s: rlp.val_at(12)?,
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

        if first != 4 {
            return Err(Error::basic_parse(rlp::DecoderError::Custom("Invalid kind")));
        }

        Self::decode_rlp(&Rlp::new(bytes)).map_err(Error::basic_parse)
    }

    /// Convert this data to rlp encoded bytes.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = BytesMut::new();
        buffer.put_u8(0x04);
        let mut rlp = rlp::RlpStream::new_list_with_buffer(buffer, 13);

        rlp.append(&self.chain_id)
            .append(&self.nonce)
            .append(&self.max_priority_gas)
            .append(&self.max_gas)
            .append(&self.gas_limit)
            .append(&self.to)
            .append(&self.value)
            .append(&self.call_data)
            .append_list::<Vec<_>, _>(self.access_list.as_slice());

        // Encode authorization list as a list of lists
        rlp.begin_list(self.authorization_list.len());
        for auth in &self.authorization_list {
            auth.encode_rlp(&mut rlp);
        }

        rlp.append(&self.recovery_id).append(&self.r).append(&self.s);

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

    #[test]
    fn eip7702_to_from_bytes() {
        use crate::ethereum::ethereum_data::{
            Authorization,
            Eip7702EthereumData,
        };

        let data = Eip7702EthereumData {
            chain_id: hex!("012a").to_vec(),
            nonce: hex!("02").to_vec(),
            max_priority_gas: hex!("2f").to_vec(),
            max_gas: hex!("2f").to_vec(),
            gas_limit: hex!("018000").to_vec(),
            to: hex!("7e3a9eaf9bcc39e2ffa38eb30bf7a93feacbc181").to_vec(),
            value: hex!("0de0b6b3a7640000").to_vec(),
            call_data: hex!("123456").to_vec(),
            access_list: vec![],
            authorization_list: vec![Authorization {
                chain_id: hex!("012a").to_vec(),
                address: hex!("1234567890abcdef1234567890abcdef12345678").to_vec(),
                nonce: hex!("01").to_vec(),
                y_parity: hex!("01").to_vec(),
                r: hex!("df48f2efd10421811de2bfb125ab75b2d3c44139c4642837fb1fccce911fd479")
                    .to_vec(),
                s: hex!("1aaf7ae92bee896651dfc9d99ae422a296bf5d9f1ca49b2d96d82b79eb112d66")
                    .to_vec(),
            }],
            recovery_id: hex!("01").to_vec(),
            r: hex!("df48f2efd10421811de2bfb125ab75b2d3c44139c4642837fb1fccce911fd479")
                .to_vec(),
            s: hex!("1aaf7ae92bee896651dfc9d99ae422a296bf5d9f1ca49b2d96d82b79eb112d66")
                .to_vec(),
        };

        let bytes = data.to_bytes();
        let parsed = EthereumData::from_bytes(&bytes).unwrap();

        // Round-trip: re-encode and compare
        assert_eq!(hex::encode(&bytes), hex::encode(parsed.to_bytes()));

        // Verify it's an Eip7702 variant
        assert!(matches!(parsed, EthereumData::Eip7702(_)));

        // Verify fields
        match parsed {
            EthereumData::Eip7702(data) => {
                assert_eq!(hex::encode(&data.chain_id), "012a");
                assert_eq!(hex::encode(&data.nonce), "02");
                assert_eq!(hex::encode(&data.max_priority_gas), "2f");
                assert_eq!(hex::encode(&data.max_gas), "2f");
                assert_eq!(hex::encode(&data.gas_limit), "018000");
                assert_eq!(
                    hex::encode(&data.to),
                    "7e3a9eaf9bcc39e2ffa38eb30bf7a93feacbc181"
                );
                assert_eq!(hex::encode(&data.value), "0de0b6b3a7640000");
                assert_eq!(hex::encode(&data.call_data), "123456");
                assert!(data.access_list.is_empty());
                assert_eq!(data.authorization_list.len(), 1);

                let auth = &data.authorization_list[0];
                assert_eq!(hex::encode(&auth.chain_id), "012a");
                assert_eq!(
                    hex::encode(&auth.address),
                    "1234567890abcdef1234567890abcdef12345678"
                );
                assert_eq!(hex::encode(&auth.nonce), "01");
                assert_eq!(hex::encode(&auth.y_parity), "01");
            }
            _ => panic!("expected Eip7702"),
        }
    }

    #[test]
    fn eip7702_empty_authorization_list() {
        use crate::ethereum::ethereum_data::Eip7702EthereumData;

        let data = Eip7702EthereumData {
            chain_id: hex!("012a").to_vec(),
            nonce: hex!("01").to_vec(),
            max_priority_gas: hex!("2f").to_vec(),
            max_gas: hex!("2f").to_vec(),
            gas_limit: hex!("018000").to_vec(),
            to: hex!("7e3a9eaf9bcc39e2ffa38eb30bf7a93feacbc181").to_vec(),
            value: vec![],
            call_data: vec![],
            access_list: vec![],
            authorization_list: vec![],
            recovery_id: hex!("01").to_vec(),
            r: hex!("df48f2efd10421811de2bfb125ab75b2d3c44139c4642837fb1fccce911fd479")
                .to_vec(),
            s: hex!("1aaf7ae92bee896651dfc9d99ae422a296bf5d9f1ca49b2d96d82b79eb112d66")
                .to_vec(),
        };

        let bytes = data.to_bytes();
        let parsed = EthereumData::from_bytes(&bytes).unwrap();

        assert!(matches!(parsed, EthereumData::Eip7702(_)));

        match parsed {
            EthereumData::Eip7702(data) => {
                assert!(data.authorization_list.is_empty());
            }
            _ => panic!("expected Eip7702"),
        }
    }

    #[test]
    fn eip7702_multiple_authorizations() {
        use crate::ethereum::ethereum_data::{
            Authorization,
            Eip7702EthereumData,
        };

        let data = Eip7702EthereumData {
            chain_id: hex!("012a").to_vec(),
            nonce: hex!("01").to_vec(),
            max_priority_gas: hex!("2f").to_vec(),
            max_gas: hex!("2f").to_vec(),
            gas_limit: hex!("018000").to_vec(),
            to: hex!("7e3a9eaf9bcc39e2ffa38eb30bf7a93feacbc181").to_vec(),
            value: vec![],
            call_data: vec![],
            access_list: vec![],
            authorization_list: vec![
                Authorization {
                    chain_id: hex!("012a").to_vec(),
                    address: hex!("1234567890abcdef1234567890abcdef12345678").to_vec(),
                    nonce: hex!("01").to_vec(),
                    y_parity: hex!("01").to_vec(),
                    r: hex!(
                        "df48f2efd10421811de2bfb125ab75b2d3c44139c4642837fb1fccce911fd479"
                    )
                    .to_vec(),
                    s: hex!(
                        "1aaf7ae92bee896651dfc9d99ae422a296bf5d9f1ca49b2d96d82b79eb112d66"
                    )
                    .to_vec(),
                },
                Authorization {
                    chain_id: hex!("012a").to_vec(),
                    address: hex!("abcdef1234567890abcdef1234567890abcdef12").to_vec(),
                    nonce: hex!("02").to_vec(),
                    y_parity: hex!("00").to_vec(),
                    r: hex!(
                        "f9fbff985d374be4a55f296915002eec11ac96f1ce2df183adf992baa9390b2f"
                    )
                    .to_vec(),
                    s: hex!(
                        "0c1e867cc960d9c74ec2e6a662b7908ec4c8cc9f3091e886bcefbeb2290fb792"
                    )
                    .to_vec(),
                },
            ],
            recovery_id: hex!("01").to_vec(),
            r: hex!("df48f2efd10421811de2bfb125ab75b2d3c44139c4642837fb1fccce911fd479")
                .to_vec(),
            s: hex!("1aaf7ae92bee896651dfc9d99ae422a296bf5d9f1ca49b2d96d82b79eb112d66")
                .to_vec(),
        };

        let bytes = data.to_bytes();
        let parsed = EthereumData::from_bytes(&bytes).unwrap();

        assert!(matches!(parsed, EthereumData::Eip7702(_)));

        match parsed {
            EthereumData::Eip7702(data) => {
                assert_eq!(data.authorization_list.len(), 2);
                assert_eq!(
                    hex::encode(&data.authorization_list[0].address),
                    "1234567890abcdef1234567890abcdef12345678"
                );
                assert_eq!(
                    hex::encode(&data.authorization_list[1].address),
                    "abcdef1234567890abcdef1234567890abcdef12"
                );
            }
            _ => panic!("expected Eip7702"),
        }
    }

    #[test]
    fn eip7702_from_bytes_standalone() {
        use crate::ethereum::ethereum_data::{
            Authorization,
            Eip7702EthereumData,
        };

        let data = Eip7702EthereumData {
            chain_id: hex!("012a").to_vec(),
            nonce: hex!("01").to_vec(),
            max_priority_gas: hex!("2f").to_vec(),
            max_gas: hex!("2f").to_vec(),
            gas_limit: hex!("018000").to_vec(),
            to: hex!("7e3a9eaf9bcc39e2ffa38eb30bf7a93feacbc181").to_vec(),
            value: vec![],
            call_data: vec![],
            access_list: vec![],
            authorization_list: vec![Authorization {
                chain_id: hex!("012a").to_vec(),
                address: hex!("1234567890abcdef1234567890abcdef12345678").to_vec(),
                nonce: hex!("01").to_vec(),
                y_parity: hex!("01").to_vec(),
                r: hex!("df48f2efd10421811de2bfb125ab75b2d3c44139c4642837fb1fccce911fd479")
                    .to_vec(),
                s: hex!("1aaf7ae92bee896651dfc9d99ae422a296bf5d9f1ca49b2d96d82b79eb112d66")
                    .to_vec(),
            }],
            recovery_id: hex!("01").to_vec(),
            r: hex!("df48f2efd10421811de2bfb125ab75b2d3c44139c4642837fb1fccce911fd479")
                .to_vec(),
            s: hex!("1aaf7ae92bee896651dfc9d99ae422a296bf5d9f1ca49b2d96d82b79eb112d66")
                .to_vec(),
        };

        let bytes = data.to_bytes();

        // Test standalone from_bytes
        let parsed = Eip7702EthereumData::from_bytes(&bytes).unwrap();
        assert_eq!(hex::encode(&parsed.chain_id), "012a");
        assert_eq!(parsed.authorization_list.len(), 1);
    }
}
