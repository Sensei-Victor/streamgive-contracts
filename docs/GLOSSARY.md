# Glossary of contract terms

These definitions describe the `donation-vault` stream lifecycle.

## Accrual

The amount earned by an NGO since the stream's last checkpoint. It is
calculated lazily as `min(rate × elapsed, balance)`, where `elapsed` starts at
the later of `last_update` and the stream's `start`. A paused stream accrues
nothing. Accrual is not transferred until a call settles the stream.

**Example:** At 10 token units per second, after 20 seconds a stream has
accrued 200 units. If only 150 units remain in its balance, accrual is capped
at 150.

## Drained

A stream status meaning its balance reached zero through normal accrual and
settlement, without being cancelled. The status is recorded when a contract
call settles the stream; it does not change just because time passes. Unlike
`Cancelled`, a `Drained` stream can become `Active` again if the donor tops it
up.

**Example:** A stream with 100 units remaining at 10 units per second is
withdrawn after at least 10 seconds. Settlement pays the final 100 units and
marks the stream `Drained`.

## Grace ledgers

An admin-configured number of extra ledger entries for which a cancelled
stream record is retained, in addition to the contract's standard cancelled
stream retention period. This is an indexing window, not extra time for the
stream to accrue. It applies when a stream is cancelled or rescued.

**Example:** Setting `cancel_grace_ledgers` to 100 retains a cancelled
stream's record for 100 additional ledgers beyond the standard retention
period, giving an indexer more time to observe its final state.

## Rescue

An admin-only emergency operation available only while the vault is paused.
It refunds the stream's entire remaining balance to its donor, marks the
stream `Cancelled`, and does not first settle uncheckpointed accrual to the
NGO. Any payments already settled remain paid.

**Example:** If a stream has a remaining balance of 1,000 units and 200 units
have accrued since the last checkpoint, `rescue_stream` refunds all 1,000
remaining units to the donor; it does not pay those 200 units to the NGO.

## Settle

To checkpoint a stream's accrued amount: reduce its balance, update its
cumulative gross withdrawn amount and `last_update`, then transfer the net
payout to the NGO and, when configured, the protocol fee to the treasury.
Settlement is an internal operation used by `withdraw` and certain donor
operations; there is no public `settle` entry point.

**Example:** If 200 units have accrued since the previous checkpoint, a
`withdraw` settles those 200 units, advances `last_update` to the current
ledger time, and leaves future accrual to be calculated from that checkpoint.
