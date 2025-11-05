# Files Needed for PR: Account/Contract Create/Update with Hooks

## Core Transaction Files (Modified)

-   `src/account/account_create_transaction.rs` - Added hooks support
-   `src/account/account_update_transaction.rs` - Added hooks support
-   `src/contract/contract_create_transaction.rs` - Added hooks support
-   `src/contract/contract_update_transaction.rs` - Added hooks support

## Hooks Module Files

All files in `src/hooks/` directory:

-   `src/hooks/evm_hook_spec.rs`
-   `src/hooks/hook_creation_details.rs`
-   `src/hooks/hook_extension_point.rs`
-   `src/hooks/hook_id.rs`
-   `src/hooks/lambda_evm_hook.rs`
-   `src/hooks/lambda_storage_update.rs`
-   `src/hooks/lambda_storage_slot.rs`
-   `src/hooks/lambda_s_store_transaction.rs`
-   `src/hooks/fungible_hook_call.rs`
-   `src/hooks/fungible_hook_type.rs`
-   `src/hooks/nft_hook_call.rs`
-   `src/hooks/nft_hook_type.rs`
-   `src/hooks/hook_call.rs`
-   `src/hooks/hook_entity_id.rs`

## Supporting Files (Modified)

-   `src/lib.rs` - Exports hooks types
-   `src/transaction/any.rs` - May include hooks support
-   `src/schedule/schedulable_transaction_body.rs` - May include hooks support
-   `src/transaction/mod.rs` - Transaction handling
-   `src/transaction_record.rs` - Transaction record support

## Test Files

-   `tests/e2e/account/mod.rs` - Test module registration
-   `tests/e2e/account/account_create_with_hooks.rs` - Account create hooks test
-   `tests/e2e/account/account_update_with_hooks.rs` - Account update hooks test
-   `tests/e2e/token/transfer_with_hooks.rs` - Transfer with hooks test
-   `tests/e2e/hooks/mod.rs` - Hooks test module
-   `tests/e2e/main.rs` - Test registration

## Example Files

-   `examples/transfer_with_hooks.rs` - Example usage

## Protobuf Files (if modified)

-   `protobufs/build.rs` - Protobuf build configuration
-   `protobufs/services` - Protobuf service definitions (submodule - be careful)

## Quick Command to List All Modified Files

```bash
git status --short | awk '{print $2}' | grep -E "(account|contract|hook|test|example)" | sort
```
