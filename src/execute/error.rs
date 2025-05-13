// SPDX-License-Identifier: Apache-2.0
use std::error::Error;

use serde::de::StdError;

fn has_transient_io_error<E: StdError>(error: E) -> bool {
    let Some(source) = error.source() else {
        return false;
    };

    if let Some(io_error) = source.downcast_ref::<std::io::Error>() {
        is_io_error_transient(io_error)
    } else {
        false
    }
}

// tonic 0.12
fn is_hyper_error_transient(error: &hyper::Error) -> bool {
    if error.is_canceled() || has_transient_io_error(error) {
        true
    } else if let Some(source) = error.source() {
        if let Some(h2_error) = source.downcast_ref::<h2::Error>() {
            h2_error.is_go_away()
        } else {
            false
        }
    } else {
        false
    }
}

fn is_io_error_transient(error: &std::io::Error) -> bool {
    match error.kind() {
        std::io::ErrorKind::BrokenPipe => true,
        _ => false,
    }
}

pub(super) fn is_tonic_status_transient(status: &tonic::Status) -> bool {
    let source = status
        .source()
        .and_then(|it| it.downcast_ref::<tonic::transport::Error>())
        .and_then(StdError::source);

    let Some(source) = source else {
        return false;
    };

    if let Some(hyper) = source.downcast_ref::<hyper::Error>() {
        is_hyper_error_transient(hyper)
    } else if let Some(hyper) = source.downcast_ref::<hyper::Error>() {
        is_hyper_error_transient(hyper)
    } else {
        false
    }
}

/// Tests some non-detection scenarios.
///
/// Because hyper does not expose constructors for its error variants, there is no
/// reasonable way to construct a test for positive detection of a hyper cancellation.
#[cfg(test)]
mod test_is_tonic_status_transient {
    use tonic::Code;

    use super::is_tonic_status_transient;

    #[test]
    fn ignores_tonic_abort() {
        let input = tonic::Status::new(Code::Aborted, "foo");

        assert!(!is_tonic_status_transient(&input));
    }

    #[test]
    fn ignores_tonic_cancel() {
        let input = tonic::Status::new(Code::Cancelled, "foo");

        assert!(!is_tonic_status_transient(&input));
    }
}
