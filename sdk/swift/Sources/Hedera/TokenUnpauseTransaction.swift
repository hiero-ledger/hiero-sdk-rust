/// Unpauses a previously paused token.
public class TokenUnpauseTransaction: Transaction {
    /// Create a new `TokenUnpauseTransaction`.
    public init(
            tokenId: TokenId? = nil
    ) {
        self.tokenId = tokenId
    }

    /// The token to be paused.
    public var tokenId: TokenId?

    /// Sets the token to be paused.
    @discardableResult
    public func tokenId(_ tokenId: TokenId?) -> Self {
        self.tokenId = tokenId

        return self
    }

    private enum CodingKeys: String, CodingKey {
        case tokenId
    }

    public override func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)

        try container.encode(tokenId, forKey: .tokenId)

        try super.encode(to: encoder)
    }
}
