// SPDX-License-Identifier: Apache-2.0

use std::env;
use std::fs::{self, create_dir_all, read_dir};
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

            entry.file_type().ok()?.is_file().then(|| entry.path())
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

    // disable emitting for the generated proto files
    cfg.out_dir(&sdk_out_dir).emit_rerun_if_changed(false).compile_protos(
        &["./sdk/transaction_list.proto"],
        &["./sdk/", out_path.to_str().unwrap()],
    )?;

    //  check if the "./sdk" folder has changed
    println!("cargo:rerun-if-changed={}", "./sdk");

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
