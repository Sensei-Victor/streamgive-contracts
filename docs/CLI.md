# Stellar CLI cheat sheet

Common `stellar contract invoke` commands for the `donation-vault` and
`ngo-registry` on Stellar testnet. These examples submit transactions; the CLI
simulates them first, then sends them unless you pass `--send=no`.

## Set up your shell

Install the [Stellar CLI](https://developers.stellar.org/docs/tools/cli) and
create or import funded testnet identities. Replace the contract IDs with
those for your deployment (the deploy script records them in
[`deployments.json`](../deployments.json)).

```sh
export NETWORK=testnet
export ADMIN=admin-testnet
export DONOR=donor-testnet
export NGO=ngo-testnet
export TREASURY=treasury-testnet
export VAULT_ID="REPLACE_WITH_DONATION_VAULT_CONTRACT_ID"
export REGISTRY_ID="REPLACE_WITH_NGO_REGISTRY_CONTRACT_ID"
export TOKEN_ID="REPLACE_WITH_TOKEN_CONTRACT_ID"

export ADMIN_ADDRESS="$(stellar keys address "$ADMIN")"
export DONOR_ADDRESS="$(stellar keys address "$DONOR")"
export NGO_ADDRESS="$(stellar keys address "$NGO")"
export TREASURY_ADDRESS="$(stellar keys address "$TREASURY")"
```

`--source` names a local Stellar CLI identity that can sign the required
authorization. Address arguments such as `--admin`, `--donor`, and `--ngo`
should be explicit `G...` addresses. Contract arguments such as `--registry`
and `--token` use `C...` contract IDs. Replace the `REPLACE_WITH_...` values
above with the contract IDs from your testnet deployment before running the
commands.

## Initialize

Normally the deploy script initializes both contracts atomically during
deployment with `__constructor`. Use `init` only for a contract deployed
without a constructor and not yet initialized. The address passed as `--admin`
must match the signing identity:

```sh
stellar contract invoke --id "$VAULT_ID" --source "$ADMIN" --network "$NETWORK" \
  -- init --admin "$ADMIN_ADDRESS"

stellar contract invoke --id "$REGISTRY_ID" --source "$ADMIN" --network "$NETWORK" \
  -- init --admin "$ADMIN_ADDRESS"
```

## Approve a registered NGO

The NGO must already be registered. The admin or an authorized registry
approver signs the request; set `--source` to that approver's identity and
`--approver` to its address:

```sh
stellar contract invoke --id "$REGISTRY_ID" --source "$ADMIN" --network "$NETWORK" \
  -- approve_ngo --approver "$ADMIN_ADDRESS" --ngo_owner "$NGO_ADDRESS"
```

## Create a stream

The donor identity signs and funds the stream. `deposit` is in the token's
smallest units; `rate` is smallest units per second. Use a testnet token
contract ID for `TOKEN_ID`.

```sh
stellar contract invoke --id "$VAULT_ID" --source "$DONOR" --network "$NETWORK" \
  -- create_stream --donor "$DONOR_ADDRESS" --ngo "$NGO_ADDRESS" \
     --token "$TOKEN_ID" --deposit 100000000 --rate 1000
```

The call returns the new stream ID. Use that ID to withdraw from the stream.

## Withdraw accrued funds

The NGO identity associated with the stream must sign:

```sh
stellar contract invoke --id "$VAULT_ID" --source "$NGO" --network "$NETWORK" \
  -- withdraw --stream_id 0
```

## Pause or resume the vault

Only the vault admin can pause or unpause it:

```sh
stellar contract invoke --id "$VAULT_ID" --source "$ADMIN" --network "$NETWORK" \
  -- pause

stellar contract invoke --id "$VAULT_ID" --source "$ADMIN" --network "$NETWORK" \
  -- unpause
```

## Change vault configuration

Configuration changes require the vault admin. Set a treasury before setting a
non-zero fee; fees are in basis points (100 bps = 1% and the maximum is
1,000 bps).

```sh
stellar contract invoke --id "$VAULT_ID" --source "$ADMIN" --network "$NETWORK" \
  -- set_treasury --treasury "$TREASURY_ADDRESS"

stellar contract invoke --id "$VAULT_ID" --source "$ADMIN" --network "$NETWORK" \
  -- set_fee_bps --fee_bps 100

stellar contract invoke --id "$VAULT_ID" --source "$ADMIN" --network "$NETWORK" \
  -- set_min_deposit --min_deposit 1000000

stellar contract invoke --id "$VAULT_ID" --source "$ADMIN" --network "$NETWORK" \
  -- set_registry --registry "$REGISTRY_ID"
```

`set_min_deposit` is in the token's smallest units. To check a function's
argument names for your installed CLI version, append `--help` after its name,
for example `-- set_fee_bps --help`. For simulation without submitting, add
`--send=no` before `--`.
