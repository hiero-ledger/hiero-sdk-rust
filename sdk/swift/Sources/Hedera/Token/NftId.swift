/*
 * ‌
 * Hedera Swift SDK
 * ​
 * Copyright (C) 2022 - 2023 Hedera Hashgraph, LLC
 * ​
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * ‍
 */

import CHedera
import Foundation

/// The unique identifier for a non-fungible token (NFT) instance on Hedera.
public struct NftId: Codable, LosslessStringConvertible, ExpressibleByStringLiteral, Equatable, ValidateChecksums {
    /// The (non-fungible) token of which this NFT is an instance.
    public let tokenId: TokenId

    /// The unique identifier for this instance.
    public let serial: UInt64

    /// Create a new `NftId` from the passed `tokenId` and `serial`.
    public init(tokenId: TokenId, serial: UInt64) {
        self.tokenId = tokenId
        self.serial = serial
    }

    private init<S: StringProtocol>(parsing description: S) throws {
        guard let (tokenId, serial) = description.splitOnce(on: "/") ?? description.splitOnce(on: "@") else {
            throw HError(
                kind: .basicParse,
                description: "unexpected NftId format - expected [tokenId]/[serialSumber] or [tokenId]@[serialNumber]")
        }

        self.tokenId = try .fromString(tokenId)
        self.serial = try UInt64(parsing: serial)
    }

    public static func fromString(_ description: String) throws -> Self {
        try self.init(parsing: description)
    }

    public init?(_ description: String) {
        try? self.init(parsing: description)
    }

    public init(stringLiteral value: StringLiteralType) {
        // swiftlint:disable:next force_try
        try! self.init(parsing: value)
    }

    public init(from decoder: Decoder) throws {
        self.init(try decoder.singleValueContainer().decode(String.self))!
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        try container.encode(String(describing: self))
    }

    public static func fromBytes(_ bytes: Data) throws -> Self {
        try bytes.withUnsafeTypedBytes { pointer in
            var shard: UInt64 = 0
            var realm: UInt64 = 0
            var num: UInt64 = 0
            var serial: UInt64 = 0

            try HError.throwing(
                error: hedera_nft_id_from_bytes(pointer.baseAddress, pointer.count, &shard, &realm, &num, &serial))

            return Self(tokenId: TokenId(shard: shard, realm: realm, num: num), serial: serial)
        }
    }

    public func toBytes() -> Data {
        var buf: UnsafeMutablePointer<UInt8>?
        let size = hedera_nft_id_to_bytes(tokenId.shard, tokenId.realm, tokenId.num, serial, &buf)

        return Data(bytesNoCopy: buf!, count: size, deallocator: .unsafeCHederaBytesFree)
    }

    public var description: String {
        "\(tokenId)/\(serial)"
    }

    internal func validateChecksums(on ledgerId: LedgerId) throws {
        try tokenId.validateChecksums(on: ledgerId)
    }
}
