# Git Commands for PR: Account/Contract Create/Update with Hooks

## Step 1: Add all hooks module files

```bash
git add src/hooks/
```

## Step 2: Add core transaction files (modified)

```bash
git add src/account/account_create_transaction.rs
git add src/account/account_update_transaction.rs
git add src/contract/contract_create_transaction.rs
git add src/contract/contract_update_transaction.rs
```

## Step 3: Add supporting files (if modified)

```bash
git add src/lib.rs
git add src/transaction/any.rs
git add src/transaction/mod.rs
git add src/transaction/execute.rs
git add src/transaction_record.rs
git add src/schedule/schedulable_transaction_body.rs
git add src/schedule/schedule_create_transaction.rs
git add src/transfer.rs
git add src/transfer_transaction.rs
git add src/token/token_nft_transfer.rs
git add src/token/token_airdrop_transaction.rs
git add src/fee_schedules.rs
git add src/query/payment_transaction.rs
```

## Step 4: Add account/contract test files

```bash
git add tests/e2e/account/account_create_with_hooks.rs
git add tests/e2e/account/account_update_with_hooks.rs
git add tests/e2e/account/mod.rs
git add tests/e2e/hooks/
git add tests/e2e/main.rs
```

## Step 5: Add snapshot files (if modified)

```bash
git add src/snapshots/transaction_record/serialize.txt
git add src/snapshots/transaction_record/serialize2.txt
git add src/token/snapshots/token_airdrop_transaction/serialize.txt
```

## All-in-one command (excludes transfer_with_hooks files)

```bash
# Add hooks module
git add src/hooks/

# Add core transaction files
git add src/account/account_create_transaction.rs \
        src/account/account_update_transaction.rs \
        src/contract/contract_create_transaction.rs \
        src/contract/contract_update_transaction.rs

# Add test files (account/contract only - NO transfer hooks)
git add tests/e2e/account/account_create_with_hooks.rs \
        tests/e2e/account/account_update_with_hooks.rs \
        tests/e2e/account/mod.rs \
        tests/e2e/hooks/ \
        tests/e2e/main.rs

# Add supporting files (only if they were modified)
git add src/lib.rs
# Add other supporting files only if they show up in git status as modified
```

## Quick check: See what will be staged

```bash
git status --short
```

## Verify before committing

```bash
git status
```

## Files to EXCLUDE (transfer hooks - different feature)

```bash
# DO NOT add these - they're for a different PR:
# examples/transfer_with_hooks.rs
# tests/e2e/token/transfer_with_hooks.rs
```
