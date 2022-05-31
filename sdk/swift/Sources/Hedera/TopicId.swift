import CHedera

/// The unique identifier for a topic on Hedera.
public final class TopicId: LosslessStringConvertible, Codable {
    /// The shard number (non-negative).
    public let shard: UInt64

    /// The realm number (non-negative).
    public let realm: UInt64

    public let num: UInt64

    public init(num: UInt64, shard: UInt64 = 0, realm: UInt64 = 0) {
        self.num = num
        self.shard = shard
        self.realm = realm
    }

    public convenience init?(_ description: String) {
        var topicId = HederaTopicId()
        let err = hedera_topic_id_from_string(description, &topicId)

        if err != HEDERA_ERROR_OK {
            return nil
        }

        self.init(num: topicId.num, shard: topicId.shard, realm: topicId.realm)
    }

    public convenience init(from decoder: Decoder) throws {
        self.init(try decoder.singleValueContainer().decode(String.self))!
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        try container.encode(String(describing: self))
    }

    public var description: String {
        "\(shard).\(realm).\(num)"
    }
}
