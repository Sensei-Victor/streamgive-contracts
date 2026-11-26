# A donor's guide to StreamGive streams

This guide is for people giving money, not for developers. It explains what a
StreamGive stream does, what each wallet signature moves, what it costs, and
what can go wrong — without assuming you have read the contract code.

The short version: a stream moves a fixed amount of a token from your wallet
into a public Stellar contract, then releases it to one NGO a little at a
time. You can pause it, slow it down, point it at another NGO, or cancel it.
When you cancel, the part that has not been released yet comes back to you.
The part that has already been released belongs to the NGO.

## The pieces, in plain words

- **Stream** — one donation, paid in advance, released over time at a fixed
  pace. It is not a bank mandate and it does not renew itself.
- **Vault** — the public contract on Stellar that holds the donation and
  releases it. It is not a company account; the rules are enforced by code.
- **NGO** — the organisation receiving the money. Each stream pays exactly one
  NGO.
- **Token** — what you are giving, such as XLM or a stablecoin on Stellar.
- **Smallest unit** — tokens are counted in whole numbers. Most Stellar tokens
  have 7 decimal places, so one whole token is 10,000,000 smallest units. For
  XLM, the smallest unit is the stroop.
- **Released or accrued** — money that has built up for the NGO since the last
  time the stream was checked. It belongs to the NGO even before the NGO
  collects it.

## Two numbers decide everything: deposit and pace

When you create a stream you choose a **deposit** (how much goes in) and a
**rate** (how fast it comes out). The rate is an amount of the token's
smallest unit **per second**, not per day or per month. The stream lasts until
the deposit runs out:

```text
duration in seconds = deposit ÷ rate
```

The pace runs every second, including nights and weekends. There is no monthly
schedule to miss and no payment date to remember.

For a token with 7 decimal places, these are roughly right:

| You want to give | Set the rate to |
| --- | --- |
| 1 XLM per day | 115 stroops per second |
| 10 XLM per day | 1,157 stroops per second |
| 30 XLM per month (30-day month) | 115 stroops per second |
| 100 XLM per month (30-day month) | 385 stroops per second |

Those are rounded down, so the NGO receives slightly less than the exact
target. For example, 115 stroops per second for 30 days releases about
29.81 XLM. Check your token's decimal places first: using the wrong scale by a
factor of ten million is the single easiest mistake to make.

## What each signature does

Every action below is a separate Stellar transaction, so your wallet pays a
small network fee each time.

| Action | What moves |
| --- | --- |
| Create a stream | The whole deposit leaves your wallet and enters the vault. |
| Top up | The amount you add leaves your wallet and joins the same stream. |
| Pause | What has already built up is paid to the NGO; the rest stops building up. |
| Resume | No money moves. The clock starts again from that moment. |
| Change the pace | What has built up at the old pace is paid first; the new pace applies only from then on. |
| Move to another NGO | What has built up goes to the old NGO; future build-up goes to the new one. |
| Cancel | What has built up goes to the NGO; everything left comes back to you. |

## Before you create a stream

1. **Use a wallet you control** that can sign Stellar transactions.
2. **Hold the token you want to give**, including any trustline (the permission
   your wallet needs to hold that asset).
3. **Get the NGO's address from an official source.** Addresses are long and a
   mistyped or substituted one cannot be undone.
4. **Decide how much you can give in advance.** The whole deposit leaves your
   wallet at creation; it is not spread across your future paydays.
5. **Decide the pace, and sanity-check the end date.** Divide the deposit by
   the rate and look at the result in days or months.

## Creating a stream

Creating a stream asks for five things:

- the donor (you),
- the NGO,
- the token,
- the deposit,
- the rate.

Your wallet shows the deposit leaving your account. Once it confirms, the
stream exists and starts building up immediately unless you chose a later
start date.

Two optional extras exist:

- **A later start date.** You can schedule a stream to begin up to one year in
  the future, for example for a pledge that should start next month. The
  deposit still leaves your wallet now, but nothing builds up until that date.
  Cancelling before the start date returns the whole deposit.
- **A memo.** A short public label, at most 32 bytes (roughly 32 plain-text
  characters), such as a campaign or invoice reference. It is stored on-chain,
  so never put anything private in it.

## What happens while the stream runs

The NGO's share builds up every second. The NGO collects it in its own
transactions; you do not need to send anything, and you cannot collect it for
them.

- Your deposit is no longer in your wallet, so do not count it as available
  balance.
- Each collection reduces what is left in the stream.
- The stream stops when the balance reaches zero. The contract calls that
  **drained**. It does not renew; top up if you want it to continue.
- You can check what is left, what has been paid, and when it will run out at
  any time in the StreamGive app or a block explorer.

## Topping up

A top-up adds more of the same token to an existing stream, keeping its NGO
and pace. Your wallet moves only the amount you add.

- The amount must meet the same minimum as a new deposit, if a minimum is set.
- Topping up a paused stream adds the money but does **not** restart it.
  Resume it when you want it to continue.
- Topping up a drained stream brings it back to life.
- A cancelled stream cannot be topped up. Create a new stream instead.

## Pausing and resuming

Pausing is for "stop for now, keep the rest for later".

When you pause, the stream first pays out whatever has built up so far, then
stops the clock. The remaining balance stays in the vault. While paused,
nothing new builds up, so the NGO is not owed for the paused time.

Resuming starts the clock again from that moment. There is no catch-up: the
paused period is not added later.

You can cancel a paused stream whenever you want. Because nothing builds up
while paused, cancelling then returns the entire remaining balance to you.

## Changing the pace or the NGO

**Changing the pace** pays out what has built up at the old rate first, then
applies the new rate only going forward. Lowering the rate is always allowed.
Raising it may be refused if the NGO is no longer on StreamGive's verified
list. Raising the rate can also be refused if it would make the stream empty
faster than StreamGive's minimum allowed time.

**Moving the stream to another NGO** pays what has built up to the current NGO
first, then sends all future build-up to the new one. The new NGO must be
verified if StreamGive has a verification list turned on. You cannot move a
stream to yourself.

## Cancelling a stream

Cancelling ends the stream for good:

1. Whatever has built up is paid to the NGO, with the usual fee treatment.
2. Everything not yet released is returned to your wallet.
3. The stream is marked cancelled and its remaining balance and rate become
   zero.

A few things follow from that:

- You cannot get back the part that has already built up, even if the NGO has
  not collected it yet. Cancelling does not reverse past giving.
- If the stream was paused, there is no new build-up, so the whole remaining
  balance comes back.
- Cancelling cannot be undone. A cancelled stream cannot be restarted, topped
  up, or paced again; create a new one if you want to resume giving.
- The record is kept for history, so a cancelled stream may still appear in
  lists and explorers.
- Cancelling works even while StreamGive has paused new donations and
  withdrawals. That is deliberate: a pause should never trap your unspent
  deposit.

## Fees

There are two different fees, and they are easy to confuse.

**StreamGive's protocol fee.** This is a percentage of each payout to the NGO,
capped at 10%. It is taken out of the NGO's payout, not added to your deposit.
Your original deposit and your cancellation refund are not charged it.

For example, at a 5% fee, a payout of 1,000 units sends 950 to the NGO and 50
to StreamGive's treasury. The fee is rounded down to the smallest unit, so very
small payouts may pay nothing. If StreamGive has not configured a fee, the
payout goes to the NGO in full.

**The Stellar network fee.** Every transaction costs a small amount of XLM to
send, set by the Stellar network rather than by StreamGive, and it can rise
when the network is busy. You pay it when you create, top up, pause, resume,
change the pace, move, or cancel a stream. The NGO pays it when collecting.

## Limits and rules at a glance

StreamGive can configure limits. These are the kinds of rules that may apply:

| Rule | What it means for you |
| --- | --- |
| Minimum deposit | A stream and a top-up must be at least the configured minimum. |
| Rate ceiling | A pace above the configured maximum is refused. |
| Minimum duration | A stream that would empty faster than the configured minimum is refused, to catch a mistyped pace. |
| Start date | No more than one year in the future. |
| Memo | At most 32 bytes, and public forever. |
| Streams per donor | A cap on how many streams you may create. The count is of every stream you have **ever created**, so cancelling an old one does not free a slot. Contact StreamGive if you need the limit raised. |

## What can go wrong

**You picked the wrong NGO.** Future build-up can be redirected to the right
NGO with a stream move, but what has already built up is paid to the NGO you
originally chose. There is no clawback from the NGO. Check the address before
signing, especially for a large deposit.

**You used the wrong units.** Entering whole tokens where the smallest unit is
expected, or the reverse, is off by a factor of ten million. A stream created
at the wrong pace can be paused or cancelled, but anything that has already
built up still goes to the NGO.

**The pace is too high.** A large rate drains the deposit quickly, and you
cannot take back what has already been released. Lower the rate to slow it
down; the change only applies going forward.

**The pace is too low.** The money stays committed for a long time. Raise the
rate if allowed, or cancel and get the unspent part back.

**The NGO loses its verified status.** Opening a stream, topping up, raising
the rate, or moving a stream to an NGO that is no longer verified may be
refused. Lowering the rate, pausing, and cancelling still work, so you can
always slow or stop the flow.

**StreamGive pauses the vault.** New streams, top-ups, pace changes, moves,
pauses, resumes, and NGO collections are blocked while the vault is paused.
Cancelling still works, which is the quickest way to stop future giving and
recover the unspent balance. In a serious incident, StreamGive's admin also has
an emergency path that returns the full remaining balance to donors without
first paying out what has built up. That path is admin-only and only available
while the vault is paused.

**The token itself has restrictions.** Some Stellar assets can be frozen or
revoked by their issuer. If your account, the NGO's account, or the vault is
frozen for that token, a refund or payout can fail until the issuer lifts the
restriction. The contract cannot override the token's own rules.

**The token takes a cut when moved.** A few tokens charge a transfer fee.
StreamGive credits your stream with the amount you entered, not what actually
arrived, so with such a token a stream can run short near the end.

**The stream sits untouched for a long time.** If there is no activity for
about 90 days, the network can archive the stream's record to save space. Your
money is not lost, but the record may need to be restored, which involves a
small network action, before the stream can be used again. Collecting, topping
up, pausing, changing the pace, or cancelling all count as activity.

**You expected a full refund on cancel.** Only the unreleased part comes back.
Money that has built up is the NGO's, whether or not it has collected yet.

**The stream is already cancelled or empty.** A cancelled stream cannot be
topped up, paced, moved, paused, or cancelled again. A drained stream can be
topped up to continue. Otherwise, create a new stream.

**You lost access to your wallet.** There is no account recovery in the
contract. Nobody at StreamGive can sign for you or move your funds without
your authorization.

**StreamGive changes its settings or the contract.** The admin — StreamGive's
operator role — can change limits and fees, pause the vault, and upgrade the
contract code. Those powers are public and limited by the contract, but they
are real. For the full list of what the admin can and cannot do, see
[THREAT_MODEL.md](./THREAT_MODEL.md).

## Checking on a stream

A StreamGive app or block explorer can show, for each stream:

- whether it is active, paused, cancelled, or finished,
- how much of your deposit is left,
- how much has been paid to the NGO so far,
- how much has built up but not yet been collected,
- the pace and the estimated finish time,
- the amount you would get back if you cancelled right now.

## Words used in this guide

| Word | Plain meaning |
| --- | --- |
| Accrued | Money that has built up for the NGO since the last collection. |
| Cancelled | Closed by the donor. Nothing more builds up, and the rest was refunded. |
| Drained | Finished because the deposit ran out, not because it was cancelled. It can be topped up. |
| Paused | Stopped for now. The rest stays in the vault and nothing new builds up. |
| Smallest unit | The token's indivisible unit; for XLM, the stroop. |
| Vault | The public Stellar contract that holds the donation and enforces the rules. |

## If you are reading the contract directly

The guide above uses plain words. If you are checking the contract or a
transaction, these are the entry points behind each action:

| Entry point | What it does |
| --- | --- |
| `create_stream` | Creates a stream that starts building up now. |
| `create_stream_with_start` | Creates a stream with a later start date. |
| `create_stream_with_memo` | Creates a stream with a public label. |
| `top_up` | Adds more of the token to a stream. |
| `pause_stream` | Pays out what has built up, then stops the clock. Returns the amount paid. |
| `resume_stream` | Restarts a paused stream from now. |
| `modify_rate` | Pays out at the old pace, then changes the pace going forward. |
| `transfer_stream` | Pays the old NGO, then redirects future build-up to a new NGO. |
| `cancel_stream` | Pays the NGO what has built up, refunds the rest to you, and closes the stream. Returns the refund. |
| `get_stream`, `streams_by_donor` | Read a stream, or list ones belonging to a donor. |
| `pending_accrual`, `pending_refund`, `depletion_time` | Views of built-up money, the refund a cancel would return now, and the estimated finish time. |

If you are unsure what a transaction will do, stop before signing and ask
StreamGive to confirm the stream id and the amounts. No support person ever
needs your secret key or recovery phrase.
