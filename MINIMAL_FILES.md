# Minimal Files Needed for Account/Contract Create/Update Hooks PR

## ✅ REQUIRED Files (Core functionality)

### 1. Core Transaction Files (Modified)

```bash
git add src/account/account_create_transaction.rs
git add src/account/account_update_transaction.rs
git add src/contract/contract_create_transaction.rs
git add src/contract/contract_update_transaction.rs
```

### 2. Hooks Module Files (New - all of them)

```bash
git add src/hooks/
```

### 3. Test Files (New)

```bash
git add tests/e2e/account/account_create_with_hooks.rs
git add tests/e2e/account/account_update_with_hooks.rs
git add tests/e2e/account/mod.rs
git add tests/e2e/hooks/
git add tests/e2e/main.rs
```

### 4. Library Exports (Modified - if hooks are exported)

```bash
git add src/lib.rs
```

## ❓ MAYBE NEEDED (Check if they actually changed)

### Supporting Files - Only add if they have hooks-related changes:

```bash
# Check first: git diff src/transaction/any.rs
# Only add if it has AccountCreate/AccountUpdate/ContractCreate/ContractUpdate hooks changes
git add src/transaction/any.rs

# Check first: git diff src/transaction/mod.rs
git add src/transaction/mod.rs

# Check first: git diff src/transaction_record.rs
git add src/transaction_record.rs
```

## ❌ NOT NEEDED (Side effects / unrelated)

These files might have changed but are NOT required for account/contract hooks:

-   `src/schedule/` files - only if schedule transactions use hooks
-   `src/transfer_transaction.rs` - transfer hooks are different feature
-   `src/token/token_nft_transfer.rs` - token hooks are different feature
-   `src/token/token_airdrop_transaction.rs` - token hooks are different feature
-   `src/fee_schedules.rs` - likely unrelated
-   `src/query/payment_transaction.rs` - likely unrelated
-   Snapshot files - auto-generated from tests, might be fine to include but not required

## 🎯 Minimal Command (Only what's absolutely needed)

```bash
# Core transaction files
git add src/account/account_create_transaction.rs \
        src/account/account_update_transaction.rs \
        src/contract/contract_create_transaction.rs \
        src/contract/contract_update_transaction.rs

# Hooks module
git add src/hooks/

# Tests
git add tests/e2e/account/account_create_with_hooks.rs \
        tests/e2e/account/account_update_with_hooks.rs \
        tests/e2e/account/mod.rs \
        tests/e2e/hooks/ \
        tests/e2e/main.rs

# Library exports
git add src/lib.rs

# Check if these changed for hooks (add only if they did):
git diff src/transaction/any.rs | grep -i hook
git diff src/transaction/mod.rs | grep -i hook
git diff src/transaction_record.rs | grep -i hook
```
