# Donation-vault fees

Fees are charged only when a payout is settled and a treasury is configured.
The effective rate is selected at withdrawal time: a token-specific
`TokenFeeBps(token)` override wins over the global `FeeBps`; otherwise the
global rate is used. Changing a fee does not rewrite existing streams and does
not charge deposits.

The fee is rounded down (toward zero):

```text
fee = floor(gross_payout * effective_fee_bps / 10,000)
net = gross_payout - fee
```

For example, with a global fee of 250 bps (2.5%), a gross payout of 1,003
stroops produces `floor(1,003 * 250 / 10,000) = 25` stroops of fee and 978
stroops for the NGO. The 25 stroops are sent to `treasury`.

## Per-token overrides

`set_token_fee_bps(USDC, 100)` makes USDC use 1% even when the global fee is
250 bps. A 1,003-USDC-stroop payout therefore charges
`floor(1,003 * 100 / 10,000) = 10` and pays 993. Clearing the override with
`clear_token_fee_bps(USDC)` makes later withdrawals use the global rate again.
The override is read when the payout is made, not when the stream is created.

## `withdraw_batch` aggregation

`withdraw_batch` settles each selected stream and aggregates amounts by token.
For each token it applies that token's effective rate to the sum of the gross
accruals, then rounds down once. For example, two USDC streams accrue 501 and
502, with a 1% override: gross is 1,003, fee is
`floor(1,003 * 100 / 10,000) = 10`, and the NGO receives 993. The returned
amounts are net amounts in the input stream order; a batch must contain streams
for one NGO, but may contain multiple tokens.

## Read-only fee views

`pending_payout(stream_id)` returns `(net, fee)` for the stream's currently
accrued gross amount without changing state. It uses the current treasury and
fee configuration, so it also reflects changes made after stream creation.
`max_fee_bps()` returns 1,000 (10%), the maximum accepted by `set_fee_bps` and
`set_token_fee_bps`. If no treasury is configured, the computed fee is zero and
the full gross amount is payable regardless of the configured rate.

`pending_payout` and `depletion_time` use integer arithmetic. Accrual is capped
at the remaining balance; depletion time uses ceiling division, so 100 units at
30 units/second depletes in `ceil(100 / 30) = 4` seconds, not 3.
