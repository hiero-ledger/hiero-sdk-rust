use jsonrpsee::types::error::INTERNAL_ERROR_CODE;
use jsonrpsee::types::{
    ErrorObject,
    ErrorObjectOwned,
};
use serde_json::json;

/// Create an internal error response
///
/// This is a helper function to create standard internal errors with a simple message.
pub fn internal_error<E: ToString>(e: E) -> ErrorObjectOwned {
    ErrorObject::owned(INTERNAL_ERROR_CODE, e.to_string(), None::<()>)
}

/// Mock a consensus node error response
///
/// This function creates an error that mimics what the consensus node would return
/// for various invalid states or parameters.
pub fn mock_consensus_error(status: &str, message: &str) -> ErrorObjectOwned {
    let data = json!({
        "status": status,
        "message": message
    });

    ErrorObject::owned(INTERNAL_ERROR_CODE, format!("Consensus node error: {}", status), Some(data))
}
