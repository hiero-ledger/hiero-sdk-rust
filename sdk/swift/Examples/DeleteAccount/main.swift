import Hedera

@main
public enum Program {
    public static func main() async throws {
        let client = Client.forTestnet()

        client.setPayerAccountId(34_952_813)
        client.addDefaultSigner(
            PrivateKey("302c020100300506032b65700420adceb87b3667f6909ab77d4016055590fe0328346f8430c4d6e4871fa2fec409")!)

        let response = try await AccountDeleteTransaction()
            .transferAccountId("0.0.6189")
            .deleteAccountId("0.0.34952813")
            .execute(client)

        _ = try await response.getReceipt(client)
    }
}
