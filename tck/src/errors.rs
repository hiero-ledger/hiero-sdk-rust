use hedera::Error;
use jsonrpsee::types::error::INTERNAL_ERROR_CODE;
use jsonrpsee::types::{
    ErrorObject,
    ErrorObjectOwned,
};
use serde_json::json;

pub(crate) const HEDERA_ERROR: i32 = -32001;

pub fn from_hedera_error(error: Error) -> ErrorObjectOwned {
    match error {
        Error::QueryPreCheckStatus { status, .. }
        | Error::ReceiptStatus { status, .. }
        | Error::TransactionPreCheckStatus { status, .. } => ErrorObject::owned(
            HEDERA_ERROR,
            "Hiero error".to_string(),
            Some(json!({
                "status": status.as_str_name().to_string(),
                "message": error.to_string(),
            })),
        ),
        _ => ErrorObject::owned(INTERNAL_ERROR_CODE, error.to_string(), None::<()>),
    }
}
