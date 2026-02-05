// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use hiero_sdk::{AccountCreateTransaction, AccountId, Client, Hbar, PrivateKey};

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, env)]
    operator_account_id: AccountId,

    #[clap(long, env)]
    operator_key: PrivateKey,

    #[clap(long, env, default_value = "testnet")]
    hedera_network: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let Args {
        operator_account_id,
        operator_key,
        hedera_network,
    } = Args::parse();

    let network_display = hedera_network.to_uppercase();
    println!("╔════════════════════════════════════════════════════════════╗");
    println!(
        "║      Hedera {} - TLS Transport Security           ║",
        network_display
    );
    println!("╚════════════════════════════════════════════════════════════╝");
    println!("\nOperator Details:");
    println!("   Account ID: {}", operator_account_id);
    println!("   Public Key: {}", operator_key.public_key());
    println!("   Network: {}", hedera_network);

    // Create a client for the specified network
    let client = Client::for_name(&hedera_network)?;
    client.set_operator(operator_account_id, operator_key.clone());

    // First, test without TLS to verify basic connectivity
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║         Step 1: Verify Connectivity (without TLS)         ║");
    println!("╚════════════════════════════════════════════════════════════╝");

    println!("\nNetwork Configuration (Before Request):");
    let network_before = client.network();
    println!("   TLS Enabled: {}", client.transport_security());
    println!("   Network has {} nodes", network_before.len());

    if network_before.is_empty() {
        println!("\nWARNING: Network is EMPTY!");
        println!("   This will cause all requests to fail.");
        println!("   Check if Client::for_name() is working correctly.");
        anyhow::bail!("Network configuration is empty - cannot proceed");
    }

    println!("   Showing first 5 nodes:");
    for (i, (address, account_id)) in network_before.iter().take(5).enumerate() {
        println!("     {}. {} -> {}", i + 1, address, account_id);
    }
    if network_before.len() > 5 {
        println!("     ... and {} more nodes", network_before.len() - 5);
    }

    // Verify addresses look correct
    println!("\nNetwork Validation:");
    let has_correct_port = network_before
        .iter()
        .any(|(addr, _)| addr.contains(":50211"));
    println!("   Contains port 50211 addresses: {}", has_correct_port);

    if !has_correct_port {
        println!("   WARNING: No addresses with port 50211 found!");
        println!("   This may indicate a network configuration issue.");
    }

    println!("\nTesting basic connectivity on port 50211 (plaintext)...");
    println!("   Creating a new account with 1 tinybar initial balance...");
    println!("   Starting request...");
    println!("   (This may take up to 30 seconds if the connection fails)");

    // Generate a key for the new account
    let new_account_key = PrivateKey::generate_ed25519();
    println!("   Generated account key: {}", new_account_key.public_key());

    // Add timeout to first request too
    let plaintext_result = tokio::time::timeout(
        tokio::time::Duration::from_secs(30),
        AccountCreateTransaction::new()
            .set_key_without_alias(new_account_key.public_key())
            .initial_balance(Hbar::from_tinybars(1))
            .execute(&client),
    )
    .await;

    match plaintext_result {
        Ok(Ok(response)) => {
            println!("    Basic connectivity works!");
            println!("   Transaction ID: {}", response.transaction_id);
            println!("   Waiting for receipt...");
            let receipt = response.get_receipt(&client).await?;
            let new_account_id = receipt.account_id.unwrap();
            println!("   Receipt Status: {:?}", receipt.status);
            println!("   Created Account ID: {}", new_account_id);
        }
        Ok(Err(e)) => {
            println!("\n Plaintext Connection Failed!");
            println!("   Error: {:?}", e);
            println!("\n Possible Issues:");
            println!("   1. Network connectivity problem");
            println!("   2. Account {} may not exist", operator_account_id);
            println!("   3. Network configuration issue");
            println!("   4. Firewall blocking connections");
            anyhow::bail!("Plaintext connection failed: {:?}", e);
        }
        Err(_timeout) => {
            println!("\n  Plaintext Connection TIMED OUT (30 seconds)");
            println!("\n  This suggests:");
            println!("   1. Network connectivity issue");
            println!("   2. All nodes are unreachable");
            println!("   3. Firewall blocking port 50211");
            println!("   4. Network configuration problem");
            anyhow::bail!("Plaintext connection timed out - check network connectivity");
        }
    }

    // Now enable TLS and try
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║           Step 2: Testing TLS on Port 50212               ║");
    println!("╚════════════════════════════════════════════════════════════╝");

    println!("\nEnabling TLS...");
    client.set_transport_security(true);
    println!("   TLS Enabled: {}", client.transport_security());
    println!("   Connection: Encrypted (port 50212)");
    println!("   Certificate Verification: Enabled");

    // Log the network configuration
    println!("\nNetwork Configuration (After Enabling TLS):");
    let network = client.network();
    println!("   TLS Enabled: {}", client.transport_security());
    println!("   Network has {} nodes:", network.len());
    println!("   NOTE: Addresses shown are from the network map (port 50211)");
    println!("   The TLS implementation will internally convert these to port 50212");
    println!();

    // Show first 5 nodes as examples
    let sample_nodes: Vec<_> = network.iter().take(5).collect();
    for (i, (address, account_id)) in sample_nodes.iter().enumerate() {
        println!("     {}. {} -> {}", i + 1, address, account_id);
        // Show what the TLS address would be
        let tls_address = address.replace(":50211", ":50212");
        println!("        → TLS: {}", tls_address);
    }
    if network.len() > 5 {
        println!("     ... and {} more nodes", network.len() - 5);
    }

    println!("\nTLS Load Balancing:");
    println!("   The SDK uses ALL available TLS nodes with round-robin load balancing.");
    println!(
        "   Requests are distributed across all {} TLS-enabled nodes.",
        network.len()
    );

    println!("\nCreating account with TLS...");
    println!("   Creating a new account with 1 tinybar initial balance...");
    println!("   This will use load balancing across all TLS nodes:");
    for (i, (addr, account_id)) in sample_nodes.iter().enumerate() {
        let tls_addr = addr.replace(":50211", ":50212");
        println!("     {}. {} ({})", i + 1, tls_addr, account_id);
    }
    if network.len() > 5 {
        println!("     ... and {} more TLS nodes", network.len() - 5);
    }
    println!("   Starting TLS request...");
    println!("   (This may take up to 30 seconds if the connection fails)");

    // Generate a key for the new account
    let new_account_key_tls = PrivateKey::generate_ed25519();
    println!(
        "   Generated account key: {}",
        new_account_key_tls.public_key()
    );

    // Use timeout to prevent indefinite hanging
    let tls_result = tokio::time::timeout(
        tokio::time::Duration::from_secs(30),
        AccountCreateTransaction::new()
            .set_key_without_alias(new_account_key_tls.public_key())
            .initial_balance(Hbar::from_tinybars(1))
            .execute(&client),
    )
    .await;

    match tls_result {
        Ok(Ok(response_tls)) => {
            println!("\nTLS Account Creation Completed Successfully!");
            println!("   Transaction ID: {}", response_tls.transaction_id);
            println!("   Waiting for receipt...");
            let receipt_tls = response_tls.get_receipt(&client).await?;
            let new_account_id_tls = receipt_tls.account_id.unwrap();
            println!("   Receipt Status: {:?}", receipt_tls.status);
            println!("   Created Account ID: {}", new_account_id_tls);

            println!("\n╔════════════════════════════════════════════════════════════╗");
            println!("║                    Success!                                ║");
            println!("╚════════════════════════════════════════════════════════════╝");
            println!("\nTLS transport security works on {}!", hedera_network);
            println!(
                "   {} supports TLS with valid certificates",
                network_display
            );
            println!("   Encrypted connection on port 50212");
            println!("   Certificate verification passed");
            println!("\nCost: ~1 HBAR per account creation (1 tinybar initial balance + fees)");
        }
        Ok(Err(e)) => {
            println!("\nTLS Connection Failed!");
            println!("   Error: {:?}", e);

            println!("\n╔════════════════════════════════════════════════════════════╗");
            println!("║                    Analysis                                ║");
            println!("╚════════════════════════════════════════════════════════════╝");
            println!("\nTLS Issue Detected:");
            println!("   • Plaintext (port 50211): Works");
            println!("   • TLS (port 50212): Failed");

            println!("\nRoot Cause Analysis:");
            println!("   Possible issues with TLS connection:");
            println!();
            println!("   1. Port Conversion:");
            println!("      • Network map uses port 50211 addresses");
            println!("      • TLS internally replaces :50211 → :50212");
            println!("      • This happens in NodeConnection::channel_with_tls_conversion()");
            println!();
            println!("   2. Load Balancing:");
            println!("      • SDK uses round-robin across ALL TLS nodes");
            println!("      • Certificates are retrieved from all nodes");
            println!("      • Requests are distributed for redundancy");
            println!();
            println!("   3. Certificate Verification:");
            println!("      • Uses SslVerifyMode::PEER (strict verification)");
            println!("      • Requires valid, signed certificates");
            println!("      • Self-signed certificates will fail");
            println!("      • Certificates are dynamically retrieved from each node");
            println!();
            println!("   4. Possible Issues:");
            println!("      • TLS nodes may not have TLS enabled on port 50212");
            println!("      • Port 50212 may be firewalled");
            println!("      • Certificate verification may be failing");
            println!("      • Network may block outbound port 50212");

            println!("\nRecommendations:");
            println!("   1. Use plaintext connections (default) for now");
            println!("   2. TLS may be for private/enterprise deployments only");
            println!("   3. Check if testnet/mainnet nodes support TLS on port 50212");
        }
        Err(_timeout) => {
            println!("\nTLS Connection TIMED OUT (30 seconds)");

            println!("\n╔════════════════════════════════════════════════════════════╗");
            println!("║                  Timeout Analysis                          ║");
            println!("╚════════════════════════════════════════════════════════════╝");
            println!("\nConnection Timeout Detected:");
            println!("   • Plaintext (port 50211): Works");
            println!("   • TLS (port 50212): Timeout (no response)");

            println!("\nWhat This Means:");
            println!("   The timeout strongly suggests:");
            println!();
            println!("   1. Port 50212 is NOT open/listening:");
            println!(
                "      • {} consensus nodes don't have TLS enabled",
                network_display
            );
            println!("      • Port 50212 is filtered/firewalled");
            println!("      • The service is not running on that port");
            println!();
            println!("   2. Mirror Node vs Consensus Node:");
            println!("      • Mirror nodes ≠ Consensus nodes");
            println!("      • This example tests CONSENSUS nodes (for transactions)");
            println!("      • Mirror nodes use different ports/protocols (REST API)");
            println!();
            println!("   3. Expected Behavior:");
            println!("      • If port was open but cert failed: immediate error");
            println!("      • If port is closed/filtered: timeout (what we see)");
            println!("      • This confirms port 50212 is NOT available");

            println!("\nConclusion:");
            println!(
                "   Hedera {} CONSENSUS nodes do NOT support TLS on port 50212.",
                network_display
            );
            println!("   The TLS feature may be:");
            println!("   • Only for local/private networks");
            println!("   • For future use when networks enable it");
            println!("   • Incomplete implementation");

            println!("\nRecommendation:");
            println!(
                "   Continue using plaintext connections (port 50211) for {}.",
                hedera_network
            );
            println!("   This is the standard and supported configuration.");
        }
    }

    Ok(())
}
