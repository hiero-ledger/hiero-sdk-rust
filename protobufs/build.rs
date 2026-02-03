// SPDX-License-Identifier: Apache-2.0

use std::env;
use std::fs::{
    self,
    create_dir_all,
    read_dir,
};
use std::path::Path;

use anyhow::Ok;
use regex::RegexBuilder;

const DERIVE_EQ_HASH: &str = "#[derive(Eq, Hash)]";
const SERVICES_FOLDER: &str = "./services/hapi/hedera-protobuf-java-api/src/main/proto/services";

fn main() -> anyhow::Result<()> {
    // services is the "base" module for the hedera protobufs
    // in the beginning, there was only services and it was named "protos"

    let services_path = Path::new(SERVICES_FOLDER);

    // The contents of this folder will be copied and modified before it is
    // used for code generation. Later we will suppress generation of cargo
    // directives on the copy, so set a directive on the source.
    println!("cargo:rerun-if-changed={}", SERVICES_FOLDER);

    if !services_path.is_dir() {
        anyhow::bail!("Folder {SERVICES_FOLDER} does not exist; do you need to `git submodule update --init`?");
    }

    let out_dir = env::var("OUT_DIR")?;
    let out_path = Path::new(&out_dir);
    let services_tmp_path = out_path.join("services");

    // ensure we start fresh
    let _ = fs::remove_dir_all(&services_tmp_path);

    create_dir_all(&services_tmp_path)?;

    // copy over services into our tmp path so we can edit
    fs_extra::copy_items(
        &[services_path],
        &out_path,
        &fs_extra::dir::CopyOptions::new().overwrite(true).copy_inside(false),
    )?;
    fs::rename(out_path.join("services"), &services_tmp_path)?;

    let services: Vec<_> = read_dir(&services_tmp_path)?
        .chain(read_dir(&services_tmp_path.join("auxiliary").join("tss"))?)
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();

            // Skip hook-related proto files
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with("hook_") || file_name.starts_with("lambda_") {
                    return None;
                }
            }

            entry.file_type().ok()?.is_file().then(|| path)
        })
        .collect();

    // iterate through each file
    let re_package = RegexBuilder::new(r"^package (.*);$").multi_line(true).build()?;
    for service in &services {
        let contents = fs::read_to_string(service)?;

        // ensure that every `package _` entry is `package proto;`
        let contents = re_package.replace(&contents, "package proto;");

        let contents = contents.replace("com.hedera.hapi.node.addressbook.", "");
        let contents = contents.replace("com.hedera.hapi.services.auxiliary.history.", "");
        let contents = contents.replace("com.hedera.hapi.services.auxiliary.tss.", "");
        let contents = contents.replace("com.hedera.hapi.platform.event.", "");
        let contents = contents.replace("com.hedera.hapi.node.hooks.", "");
        let contents = contents.replace("com.hedera.hapi.node.hooks.legacy.", "");

        let contents = remove_unused_types(&contents);

        fs::write(service, &*contents)?;
    }

    let mut cfg = tonic_build::configure()
        // We have already emitted a cargo directive to trigger a rerun on the source folder
        // that the copy this builds is based on. If the directives are not suppressed, the
        // crate will rebuild on every compile due to the modified time stamps post-dating
        // the start time of the compile action.
        .emit_rerun_if_changed(false);

    // most of the protobufs in "basic types" should be Eq + Hash + Copy
    // any protobufs that would typically be used as parameter, that meet the requirements of those
    // traits
    cfg = cfg
        .type_attribute("proto.ShardID", DERIVE_EQ_HASH)
        .type_attribute("proto.RealmID", DERIVE_EQ_HASH)
        .type_attribute("proto.AccountID", DERIVE_EQ_HASH)
        .type_attribute("proto.AccountID.account", DERIVE_EQ_HASH)
        .type_attribute("proto.FileID", DERIVE_EQ_HASH)
        .type_attribute("proto.ContractID", DERIVE_EQ_HASH)
        .type_attribute("proto.ContractID.contract", DERIVE_EQ_HASH)
        .type_attribute("proto.TransactionID", DERIVE_EQ_HASH)
        .type_attribute("proto.Timestamp", DERIVE_EQ_HASH)
        .type_attribute("proto.NftTransfer", DERIVE_EQ_HASH)
        .type_attribute("proto.Fraction", DERIVE_EQ_HASH)
        .type_attribute("proto.TopicID", DERIVE_EQ_HASH)
        .type_attribute("proto.TokenID", DERIVE_EQ_HASH)
        .type_attribute("proto.ScheduleID", DERIVE_EQ_HASH)
        .type_attribute("proto.FeeComponents", DERIVE_EQ_HASH)
        .type_attribute("proto.Key", DERIVE_EQ_HASH)
        .type_attribute("proto.KeyList", DERIVE_EQ_HASH)
        .type_attribute("proto.ThresholdKey", DERIVE_EQ_HASH)
        .type_attribute("proto.Key.key", DERIVE_EQ_HASH)
        .type_attribute("proto.SignaturePair", DERIVE_EQ_HASH)
        .type_attribute("proto.SignaturePair.signature", DERIVE_EQ_HASH)
        .type_attribute("proto.FeeData", DERIVE_EQ_HASH)
        .type_attribute("proto.TokenBalance", DERIVE_EQ_HASH)
        .type_attribute("proto.TokenAssociation", DERIVE_EQ_HASH)
        .type_attribute("proto.CryptoAllowance", DERIVE_EQ_HASH)
        .type_attribute("proto.TokenAllowance", DERIVE_EQ_HASH)
        .type_attribute("proto.GrantedCryptoAllowance", DERIVE_EQ_HASH)
        .type_attribute("proto.GrantedTokenAllowance", DERIVE_EQ_HASH)
        .type_attribute("proto.Duration", DERIVE_EQ_HASH);

    // the ResponseCodeEnum should be marked as #[non_exhaustive] so
    // adding variants does not trigger a breaking change
    cfg = cfg.type_attribute("proto.ResponseCodeEnum", "#[non_exhaustive]");

    // the ResponseCodeEnum is not documented in the proto source
    cfg = cfg.type_attribute(
        "proto.ResponseCodeEnum",
        r#"#[doc = "
  Returned in `TransactionReceipt`, `Error::PreCheckStatus`, and `Error::ReceiptStatus`.
  
  The success variant is `Success` which is what a `TransactionReceipt` will contain for a
  successful transaction.
      "]"#,
    );

    cfg.compile_protos(&services, &[out_path.to_str().unwrap()])?;

    // NOTE: prost generates rust doc comments and fails to remove the leading * line
    remove_useless_comments(&Path::new(&env::var("OUT_DIR")?).join("proto.rs"))?;

    // mirror
    // NOTE: must be compiled in a separate folder otherwise it will overwrite the previous build
    let mirror_out_dir = Path::new(&env::var("OUT_DIR")?).join("mirror");
    create_dir_all(&mirror_out_dir)?;

    tonic_build::configure()
        .build_server(false)
        .extern_path(".proto.Timestamp", "crate::services::Timestamp")
        .extern_path(".proto.TopicID", "crate::services::TopicId")
        .extern_path(".proto.FileID", "crate::services::FileId")
        .extern_path(".proto.NodeAddress", "crate::services::NodeAddress")
        .extern_path(".proto.Transaction", "crate::services::Transaction")
        .extern_path(
            ".proto.ConsensusMessageChunkInfo",
            "crate::services::ConsensusMessageChunkInfo",
        )
        .out_dir(&mirror_out_dir)
        .compile_protos(
            &["./mirror/consensus_service.proto", "./mirror/mirror_network_service.proto"],
            &["./mirror/", out_path.to_str().unwrap()],
        )?;

    remove_useless_comments(&mirror_out_dir.join("proto.rs"))?;

    // sdk
    // NOTE: must be compiled in a separate folder otherwise it will overwrite the previous build
    let sdk_out_dir = Path::new(&env::var("OUT_DIR")?).join("sdk");
    create_dir_all(&sdk_out_dir)?;

    // note:
    // almost everything in services must be specified here.
    let cfg = tonic_build::configure();
    let cfg = builder::extern_basic_types(cfg)
        .services_same("AssessedCustomFee")
        .services_same("ConsensusCreateTopicTransactionBody")
        .services_same("ConsensusDeleteTopicTransactionBody")
        .services_same("ConsensusMessageChunkInfo")
        .services_same("ConsensusSubmitMessageTransactionBody")
        .services_same("ConsensusUpdateTopicTransactionBody")
        .services_same("ContractCallTransactionBody")
        .services_same("ContractCreateTransactionBody")
        .services_same("ContractDeleteTransactionBody")
        .services_same("ContractUpdateTransactionBody")
        .services_same("CryptoAddLiveHashTransactionBody")
        .services_same("CryptoApproveAllowanceTransactionBody")
        .services_same("CryptoCreateTransactionBody")
        .services_same("CryptoDeleteTransactionBody")
        .services_same("CryptoDeleteAllowanceTransactionBody")
        .services_same("CryptoTransferTransactionBody")
        .services_same("CryptoUpdateTransactionBody")
        .services_same("CryptoDeleteLiveHashTransactionBody")
        .services_same("CustomFee")
        .services_same("Duration")
        .services_same("EthereumTransactionBody")
        .services_same("FileAppendTransactionBody")
        .services_same("FileCreateTransactionBody")
        .services_same("FileDeleteTransactionBody")
        .services_same("FileUpdateTransactionBody")
        .services_same("FixedFee")
        .services_same("FractionalFee")
        .services_same("FreezeTransactionBody")
        .services_same("FreezeType")
        .services_same("LiveHash")
        .services_same("NftRemoveAllowance")
        .services_same("NodeStake")
        .services_same("NodeStakeUpdateTransactionBody")
        .services_same("RoyaltyFee")
        .services_same("SchedulableTransactionBody")
        .services_same("ScheduleCreateTransactionBody")
        .services_same("ScheduleDeleteTransactionBody")
        .services_same("ScheduleSignTransactionBody")
        .services_same("SystemDeleteTransactionBody")
        .services_same("SystemUndeleteTransactionBody")
        .services_same("TokenAssociateTransactionBody")
        .services_same("TokenBurnTransactionBody")
        .services_same("TokenCreateTransactionBody")
        .services_same("TokenDeleteTransactionBody")
        .services_same("TokenDissociateTransactionBody")
        .services_same("TokenFeeScheduleUpdateTransactionBody")
        .services_same("TokenFreezeAccountTransactionBody")
        .services_same("TokenGrantKycTransactionBody")
        .services_same("TokenMintTransactionBody")
        .services_same("TokenPauseTransactionBody")
        .services_same("TokenRevokeKycTransactionBody")
        .services_same("TokenUnfreezeAccountTransactionBody")
        .services_same("TokenUnpauseTransactionBody")
        .services_same("TokenUpdateTransactionBody")
        .services_same("TokenUpdateNftsTransactionBody")
        .services_same("TokenWipeAccountTransactionBody")
        .services_same("TssMessageTransactionBody")
        .services_same("TssVoteTransactionBody")
        .services_same("TssShareSignatureTransactionBody")
        .services_same("TssEncryptionKeyTransactionBody")
        .services_same("Transaction")
        .services_same("TransactionBody")
        .services_same("UncheckedSubmitBody")
        .services_same("UtilPrngTransactionBody")
        .services_same("VirtualAddress");

    cfg.out_dir(&sdk_out_dir).compile_protos(
        &["./sdk/transaction_list.proto"],
        &["./sdk/", out_path.to_str().unwrap()],
    )?;

    // see note wrt services.
    remove_useless_comments(&sdk_out_dir.join("proto.rs"))?;

    Ok(())
}

fn remove_useless_comments(path: &Path) -> anyhow::Result<()> {
    let mut contents = fs::read_to_string(path)?;

    contents = contents.replace("///*\n", "");
    contents = contents.replace("/// *\n", "");
    contents = contents.replace("/// UNDOCUMENTED", "");

    // Remove code examples in comments
    let re = regex::Regex::new(r"/// ```[\s\S]*?/// ```\n").unwrap();
    contents = re.replace_all(&contents, "").to_string();

    fs::write(path, contents)?;

    Ok(())
}

// Helper function to comment out entire oneof blocks by name
fn comment_out_oneof(contents: &str, oneof_name: &str) -> String {
    let lines: Vec<&str> = contents.lines().collect();
    let mut result = Vec::new();
    let mut i = 0;
    let oneof_pattern = format!("oneof {} {{", oneof_name);

    while i < lines.len() {
        let line = lines[i].trim();
        if line.starts_with(&oneof_pattern) || line == &format!("oneof {}", oneof_name) {
            // Found the start of the oneof, comment it out and track brace depth
            let original_line = lines[i];
            let indent = original_line.len() - original_line.trim_start().len();
            let indent_str = &original_line[..indent];

            let mut depth = 0;
            let mut found_opening_brace = line.contains('{');

            result.push(format!("{}// {}", indent_str, lines[i].trim_start()));
            if found_opening_brace {
                depth = 1;
            }
            i += 1;

            // Comment out all lines until we close the oneof block
            while i < lines.len() && (depth > 0 || !found_opening_brace) {
                let current_line = lines[i];
                let current_indent = current_line.len() - current_line.trim_start().len();
                let current_indent_str = &current_line[..current_indent.min(current_line.len())];

                result.push(format!("{}// {}", current_indent_str, current_line.trim_start()));

                if !found_opening_brace && current_line.trim_start().starts_with('{') {
                    found_opening_brace = true;
                    depth = 1;
                }

                if found_opening_brace {
                    for ch in current_line.chars() {
                        if ch == '{' {
                            depth += 1;
                        } else if ch == '}' {
                            depth -= 1;
                        }
                    }
                }

                i += 1;
                if depth == 0 && found_opening_brace {
                    break;
                }
            }
        } else {
            result.push(lines[i].to_string());
            i += 1;
        }
    }

    result.join("\n")
}

// Helper function to comment out entire message blocks
fn comment_out_message(contents: &str, message_name: &str) -> String {
    let lines: Vec<&str> = contents.lines().collect();
    let mut result = Vec::new();
    let mut i = 0;
    let message_start = format!("message {} {{", message_name);

    while i < lines.len() {
        let line = lines[i].trim();
        if line.starts_with(&message_start) || line == &format!("message {}", message_name) {
            // Found the start of the message, comment it out and track brace depth
            let mut depth = 0;
            let mut found_opening_brace = line.contains('{');

            result.push(format!("// {}", lines[i]));
            if found_opening_brace {
                depth = 1;
            }
            i += 1;

            // Comment out all lines until we close the message block
            while i < lines.len() && (depth > 0 || !found_opening_brace) {
                let current_line = lines[i];
                result.push(format!("// {}", current_line));

                if !found_opening_brace && current_line.trim_start().starts_with('{') {
                    found_opening_brace = true;
                    depth = 1;
                }

                if found_opening_brace {
                    for ch in current_line.chars() {
                        if ch == '{' {
                            depth += 1;
                        } else if ch == '}' {
                            depth -= 1;
                        }
                    }
                }

                i += 1;
                if depth == 0 && found_opening_brace {
                    break;
                }
            }
        } else {
            result.push(lines[i].to_string());
            i += 1;
        }
    }

    result.join("\n")
}

// Temporary function to remove unused types in transaction.proto
fn remove_unused_types(contents: &str) -> String {
    let contents = contents.replace(
        "import \"platform/event/state_signature_transaction.proto\";",
        "// import \"platform/event/state_signature_transaction.proto\";",
    );

    let contents = contents.replace(
        "import \"services/auxiliary/history/history_proof_vote.proto\";",
        "// import \"services/auxiliary/history/history_proof_vote.proto\";",
    );
    let contents = contents.replace(
        "import \"services/auxiliary/history/history_proof_signature.proto\";",
        "// import \"services/auxiliary/history/history_proof_signature.proto\";",
    );
    let contents = contents.replace(
        "import \"services/auxiliary/history/history_proof_key_publication.proto\";",
        "// import \"services/auxiliary/history/history_proof_key_publication.proto\";",
    );

    let contents = contents.replace(
        "import \"services/auxiliary/hints/hints_key_publication.proto\";",
        "// import \"services/auxiliary/hints/hints_key_publication.proto\";",
    );

    let contents = contents.replace(
        "import \"services/auxiliary/hints/hints_preprocessing_vote.proto\";",
        "// import \"services/auxiliary/hints/hints_preprocessing_vote.proto\";",
    );

    let contents = contents.replace(
        "import \"services/auxiliary/hints/hints_partial_signature.proto\";",
        "// import \"services/auxiliary/hints/hints_partial_signature.proto\";",
    );

    let contents = contents.replace(
        "import \"services/auxiliary/hints/crs_publication.proto\";",
        "// import \"services/auxiliary/hints/crs_publication.proto\";",
    );

    let contents = contents.replace("StateSignatureTransaction", "// StateSignatureTransaction");

    let contents =
        contents.replace("HistoryProofSignatureTransaction", "// HistoryProofSignatureTransaction");

    let contents = contents.replace(
        "HistoryProofKeyPublicationTransaction",
        "// HistoryProofKeyPublicationTransaction",
    );

    let contents =
        contents.replace("HistoryProofVoteTransaction", "// HistoryProofVoteTransaction");

    let contents = contents.replace(
        "com.hedera.hapi.services.auxiliary.hints.HintsPreprocessingVoteTransactionBody",
        "// com.hedera.hapi.services.auxiliary.hints.HintsPreprocessingVoteTransactionBody",
    );

    let contents = contents.replace(
        "com.hedera.hapi.services.auxiliary.hints.HintsKeyPublicationTransactionBody",
        "// com.hedera.hapi.services.auxiliary.hints.HintsKeyPublicationTransactionBody",
    );

    let contents = contents.replace(
        "com.hedera.hapi.services.auxiliary.hints.HintsPartialSignatureTransactionBody",
        "// com.hedera.hapi.services.auxiliary.hints.HintsPartialSignatureTransactionBody",
    );

    let contents = contents.replace(
        "com.hedera.hapi.services.auxiliary.hints.CrsPublicationTransactionBody",
        "// com.hedera.hapi.services.auxiliary.hints.CrsPublicationTransactionBody",
    );

    // Comment out hook-related imports
    let contents = contents.replace(
        "import \"services/hook_types.proto\";",
        "// import \"services/hook_types.proto\";",
    );

    let contents = contents.replace(
        "import \"services/hook_dispatch.proto\";",
        "// import \"services/hook_dispatch.proto\";",
    );

    let contents = contents.replace(
        "import \"services/lambda_sstore.proto\";",
        "// import \"services/lambda_sstore.proto\";",
    );

    // Comment out hook transaction bodies in transaction.proto
    // Note: After package replacement, these become just the class name
    let contents = contents.replace(
        "HookDispatchTransactionBody hook_dispatch",
        "// HookDispatchTransactionBody hook_dispatch",
    );

    let contents = contents.replace(
        "LambdaSStoreTransactionBody lambda_sstore",
        "// LambdaSStoreTransactionBody lambda_sstore",
    );

    // Comment out hook creation details in various transaction bodies
    // Note: After package replacement, this becomes just "HookCreationDetails"
    let contents = contents.replace(
        "repeated HookCreationDetails hook_creation_details",
        "// repeated HookCreationDetails hook_creation_details",
    );

    // Comment out hook_ids_to_delete field
    let contents = contents
        .replace("repeated int64 hook_ids_to_delete", "// repeated int64 hook_ids_to_delete");

    // Comment out HookId references
    let contents = contents.replace("proto.HookId", "// proto.HookId");
    let contents = contents.replace("HookId hook_id", "// HookId hook_id");

    // Comment out entire Hook message blocks
    let contents = comment_out_message(&contents, "HookId");
    let contents = comment_out_message(&contents, "HookEntityId");
    let contents = comment_out_message(&contents, "HookCall");
    let contents = comment_out_message(&contents, "EvmHookCall");

    // Comment out all hook-related oneofs that contain HookCall fields
    let contents = comment_out_oneof(&contents, "hook_call");
    let contents = comment_out_oneof(&contents, "sender_allowance_hook_call");
    let contents = comment_out_oneof(&contents, "receiver_allowance_hook_call");

    // Comment out hook-related enum values in HederaFunctionality
    let contents = contents.replace("LambdaSStore = 109;", "// LambdaSStore = 109;");
    let contents = contents.replace("HookDispatch = 110;", "// HookDispatch = 110;");

    contents
}

trait BuilderExtensions {
    fn services_path<T: AsRef<str>, U: AsRef<str>>(self, proto_name: T, rust_name: U) -> Self
    where
        Self: Sized;

    fn services_same<T: AsRef<str>>(self, name: T) -> Self
    where
        Self: Sized,
    {
        self.services_path(&name, &name)
    }
}

impl BuilderExtensions for tonic_build::Builder {
    fn services_path<T: AsRef<str>, U: AsRef<str>>(self, proto_name: T, rust_name: U) -> Self {
        let proto_name = proto_name.as_ref();
        let rust_name = rust_name.as_ref();

        self.extern_path(format!(".proto.{proto_name}"), format!("crate::services::{rust_name}"))
    }
}

mod builder {
    use crate::BuilderExtensions;

    pub(super) fn extern_basic_types(builder: tonic_build::Builder) -> tonic_build::Builder {
        builder
            .services_same("Fraction")
            .services_same("Timestamp")
            .services_path("AccountID", "AccountId")
            .services_path("TokenID", "TokenId")
            .services_same("AccountAmount")
            .services_same("CurrentAndNextFeeSchedule")
            .services_same("FeeComponents")
            .services_same("FeeData")
            .services_same("FeeSchedule")
            .services_same("Key")
            .services_path("FileID", "FileId")
            .services_same("KeyList")
            .services_same("NftTransfer")
            .services_same("NodeAddress")
            .services_same("NodeAddressBook")
            .services_path("RealmID", "RealmId")
            .services_path("ScheduleID", "ScheduleId")
            .services_path("SemanticVersion", "SemanticVersion")
            .services_path("ServiceEndpoint", "ServiceEndpoint")
            .services_same("ServicesConfigurationList")
            .services_path("Setting", "Setting")
            .services_path("ShardID", "ShardId")
            .services_path("Signature", "Signature")
            .services_path("SignatureList", "SignatureList")
            .services_path("SignatureMap", "SignatureMap")
            .services_path("SignaturePair", "SignaturePair")
            .services_path("ThresholdKey", "ThresholdKey")
            .services_path("ThresholdSignature", "ThresholdSignature")
            .services_path("TimestampSeconds", "TimestampSeconds")
            .services_path("TokenBalance", "TokenBalance")
            .services_path("TokenBalances", "TokenBalances")
            .services_path("TokenRelationship", "TokenRelationship")
            .services_path("TokenTransferList", "TokenTransferList")
            .services_path("TopicID", "TopicId")
            .services_path("TransactionFeeSchedule", "TransactionFeeSchedule")
            .services_path("TransactionID", "TransactionId")
            .services_path("TransferList", "TransferList")
            .services_path("HederaFunctionality", "HederaFunctionality")
            .services_path("SubType", "SubType")
            .services_path("TokenFreezeStatus", "TokenFreezeStatus")
            .services_path("TokenKycStatus", "TokenKycStatus")
            .services_path("TokenSupplyType", "TokenSupplyType")
            .services_path("TokenType", "TokenType")
            .services_path("GrantedCryptoAllowance", "GrantedCryptoAllowance")
            .services_path("GrantedTokenAllowance", "GrantedTokenAllowance")
            .services_path("CryptoAllowance", "CryptoAllowance")
            .services_path("TokenAllowance", "TokenAllowance")
            .services_path("GrantedNftAllowance", "GrantedNftAllowance")
            .services_path("NftAllowance", "NftAllowance")
            .services_path("TokenPauseStatus", "TokenPauseStatus")
            .services_path("TokenAssociation", "TokenAssociation")
            .services_path("ContractID", "ContractId")
            .services_path("StakingInfo", "StakingInfo")
    }
}
