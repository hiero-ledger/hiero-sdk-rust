// SPDX-License-Identifier: Apache-2.0

/// Get an optional field from a protobuf object, returning an error if the field does not exist.
macro_rules! pb_getf {
    ($expr:expr, $field:ident) => {{
        $expr.$field.ok_or_else(|| {
            $crate::Error::from_protobuf(concat!("unexpected missing `", stringify!($field), "`"))
        })
    }};
}

/// Get a specific variant from a `oneof` field of a protobuf object, returning an error
/// if the variant is not as expected.
macro_rules! pb_getv {
    ($expr:expr, $variant:ident, $ty:ty) => {{
        use $ty::*;

        match $expr {
            $variant(it) => it,

            _ => {
                return Err($crate::Error::from_protobuf(format!(
                    concat!("unexpected {:?} received, expecting `", stringify!($variant), "`"),
                    $expr
                )));
            }
        }
    }};
}
