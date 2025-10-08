// SPDX-License-Identifier: Apache-2.0

use std::fmt::{
    self,
    Debug,
    Display,
    Formatter,
};
use std::str::FromStr;

use crate::entity_id::{
    Checksum,
    ValidateChecksums,
};
use crate::ledger_id::RefLedgerId;
use crate::proto::services;
#[cfg(not(target_arch = "wasm32"))]
use crate::Client;
use crate::{
    EntityId,
    Error,
    FromProtobuf,
    ToProtobuf,
};

/// The unique identifier for a file on Hiero.
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct FileId {
    /// The shard number.
    pub shard: u64,

    /// The realm number.
    pub realm: u64,

    /// The file number.
    pub num: u64,

    /// A checksum if the file ID was read from a user inputted string which inclueded a checksum
    pub checksum: Option<Checksum>,
}

impl FileId {
    /// Address of the public [node address book](crate::NodeAddressBook) for the current network.
    #[deprecated(note = "use `get_address_book_file_id_for` instead")]
    pub const ADDRESS_BOOK: Self = Self::new(0, 0, 102);

    /// Address of the current fee schedule for the network.
    #[deprecated(note = "use `get_fee_schedule_file_id_for` instead")]
    pub const FEE_SCHEDULE: Self = Self::new(0, 0, 111);

    /// Address of the [current exchange rate](crate::ExchangeRates) of HBAR to USD.
    #[deprecated(note = "use `get_exchange_rates_file_id_for` instead")]
    pub const EXCHANGE_RATES: Self = Self::new(0, 0, 112);

    /// Create a `FileId` with the given `shard.realm.num`.
    pub const fn new(shard: u64, realm: u64, num: u64) -> Self {
        Self { shard, realm, num, checksum: None }
    }

    /// Address of the public [node address book](crate::NodeAddressBook) for the current network.
    pub fn get_address_book_file_id_for(realm: u64, shard: u64) -> Self {
        let address_book_num = 102;
        Self::new(shard, realm, address_book_num)
    }

    /// Address of the current fee schedule for the network.
    pub fn get_fee_schedule_file_id_for(realm: u64, shard: u64) -> Self {
        let fee_schedule_num = 111;
        Self::new(shard, realm, fee_schedule_num)
    }

    /// Address of the [current exchange rate](crate::ExchangeRates) of HBAR to USD.
    pub fn get_exchange_rates_file_id_for(realm: u64, shard: u64) -> Self {
        let exchange_rates_num = 112;
        Self::new(shard, realm, exchange_rates_num)
    }

    /// Create a new `FileId` from protobuf-encoded `bytes`.
    ///
    /// # Errors
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the bytes fails to produce a valid protobuf.
    /// - [`Error::FromProtobuf`](crate::Error::FromProtobuf) if decoding the protobuf fails.
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        FromProtobuf::from_bytes(bytes)
    }

    /// Create a `FileId` from a solidity address.
    ///
    /// # Errors
    /// - [`Error::BasicParse`] if `address` cannot be parsed as a solidity address.
    pub fn from_solidity_address(address: &str) -> crate::Result<Self> {
        let EntityId { shard, realm, num, checksum } = EntityId::from_solidity_address(address)?;

        Ok(Self { shard, realm, num, checksum })
    }

    /// Convert `self` to a protobuf-encoded [`Vec<u8>`].
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        ToProtobuf::to_bytes(self)
    }

    /// Convert `self` into a solidity `address`
    ///
    /// # Errors
    /// - [`Error::BasicParse`] if `self.shard` is larger than `u32::MAX`.
    pub fn to_solidity_address(&self) -> crate::Result<String> {
        EntityId { shard: self.shard, realm: self.realm, num: self.num, checksum: None }
            .to_solidity_address()
    }

    /// Convert `self` to a string with a valid checksum.
    #[cfg(not(target_arch = "wasm32"))]
    #[must_use]
    pub fn to_string_with_checksum(&self, client: &Client) -> String {
        EntityId::to_string_with_checksum(self.to_string(), client)
    }

    /// Validates `self.checksum` (if it exists) for `client`.
    ///
    /// # Errors
    /// - [`Error::BadEntityId`] if there is a checksum, and the checksum is not valid for the client's `ledger_id`.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn validate_checksum(&self, client: &Client) -> Result<(), Error> {
        EntityId::validate_checksum(self.shard, self.realm, self.num, self.checksum, client)
    }
}

impl ValidateChecksums for FileId {
    #[cfg(not(target_arch = "wasm32"))]
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> Result<(), Error> {
        EntityId::validate_checksum_for_ledger_id(
            self.shard,
            self.realm,
            self.num,
            self.checksum,
            ledger_id,
        )
    }

    #[cfg(target_arch = "wasm32")]
    fn validate_checksums(&self, _ledger_id: &RefLedgerId) -> Result<(), Error> {
        // Checksum validation requires networking context, not available in WASM
        Ok(())
    }
}

impl Debug for FileId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl Display for FileId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.shard, self.realm, self.num)
    }
}

impl FromProtobuf<services::FileId> for FileId {
    fn from_protobuf(pb: services::FileId) -> crate::Result<Self> {
        Ok(Self {
            num: pb.file_num as u64,
            shard: pb.shard_num as u64,
            realm: pb.realm_num as u64,
            checksum: None,
        })
    }
}

impl ToProtobuf for FileId {
    type Protobuf = services::FileId;

    fn to_protobuf(&self) -> Self::Protobuf {
        services::FileId {
            file_num: self.num as i64,
            realm_num: self.realm as i64,
            shard_num: self.shard as i64,
        }
    }
}

impl From<u64> for FileId {
    fn from(num: u64) -> Self {
        Self { num, shard: 0, realm: 0, checksum: None }
    }
}

impl FromStr for FileId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        EntityId::from_str(s).map(Self::from)
    }
}

impl From<EntityId> for FileId {
    fn from(value: EntityId) -> Self {
        let EntityId { shard, realm, num, checksum } = value;
        Self { shard, realm, num, checksum }
    }
}

#[cfg(test)]
mod tests {
    use crate::FileId;

    #[test]
    fn should_serialize_from_string() {
        assert_eq!("0.0.5005", "0.0.5005".parse::<FileId>().unwrap().to_string());
    }

    #[test]
    fn from_bytes() {
        assert_eq!(
            "0.0.5005",
            FileId::from_bytes(&FileId::new(0, 0, 5005).to_bytes()).unwrap().to_string()
        );
    }

    #[test]
    fn from_solidity_address() {
        assert_eq!(
            "0.0.5005",
            FileId::from_solidity_address("000000000000000000000000000000000000138D")
                .unwrap()
                .to_string()
        );
    }

    #[test]
    fn to_solidity_address() {
        assert_eq!(
            "000000000000000000000000000000000000138d",
            FileId::new(0, 0, 5005).to_solidity_address().unwrap()
        );
    }
}
