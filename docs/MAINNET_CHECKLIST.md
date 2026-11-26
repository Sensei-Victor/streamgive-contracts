# Mainnet launch checklist

Use this checklist for every first mainnet deployment and for any redeploy
decision. The mainnet script is intentionally one-shot: it refuses to overwrite
an existing `mainnet` entry and initializes the contracts atomically with the
configured admin. Complete all required items and record the evidence before
running `scripts/deploy-mainnet.sh --confirm`.

## Sign-off

| Sign-off | Owner | Status |
| --- | --- | --- |
| Security review complete | Security lead | [ ] |
| Engineering readiness complete | Engineering lead | [ ] |
| Operations and incident readiness complete | Operations lead | [ ] |
| Final go / no-go decision | Release manager | [ ] |

Record the final decision, date, and approver here:

- Decision:
- Date:
- Release manager:

## Pre-launch checklist

| Done | Owner | Item | Required evidence |
| --- | --- | --- | --- |
| [ ] | Engineering lead | Run the full test suite on the exact release commit: `cargo test --workspace`. | Passing CI link and commit SHA. |
| [ ] | Engineering lead | Review the release diff and confirm no unrelated or unreviewed contract changes are included. | Approved pull request and release commit SHA. |
| [ ] | Security lead | Record the audit status for the release. An internal threat-model review is not an external audit; name the auditor, scope, report date, and remediation status if an audit exists. | Audit/report URL or an explicit written `not externally audited` decision. |
| [ ] | Security lead | Verify every audit finding and pre-launch security review item is resolved or explicitly accepted. | Findings list with owners and accepted residual risk. |
| [ ] | Release manager | Confirm the deploy commit is tagged or otherwise reproducible and the Wasm build is reproducible. | Release tag/commit and build environment details. |
| [ ] | Engineering lead | Confirm the mainnet network passphrase will be used and no conflicting `STELLAR_NETWORK_PASSPHRASE` override is set. | Deployment environment review. |
| [ ] | Operations lead | Confirm `stellar`, `node`, `rustup`, and the `wasm32v1-none` target are installed on the approved deploy workstation. | Prerequisite check or clean build log. |
| [ ] | Operations lead | Confirm the source/deployer identity is funded for real mainnet fees and that its signing method is approved. | Funded identity and signer procedure. |
| [ ] | Release manager | Confirm `deployments.json` has no existing `mainnet` entry and that redeployment is not being attempted accidentally. | Before-deploy JSON review. |

## Audit and security decision

| Done | Owner | Item | Required evidence |
| --- | --- | --- | --- |
| [ ] | Security lead | Record the launch audit status: externally audited, internally reviewed only, or not reviewed. | Signed audit decision. |
| [ ] | Security lead | Review [THREAT_MODEL.md](./THREAT_MODEL.md), especially T-21 (missing registry), C-1 (admin authority), fee risks, and pause behavior. | Review sign-off and any accepted-risk tickets. |
| [ ] | Security lead | Confirm the release handles known residual risks from the threat model. | Exception or mitigation record for each accepted risk. |
| [ ] | Engineering lead | Verify the Wasm hashes for the release against reviewed artifacts before deployment. | NGO-registry and donation-vault SHA-256 hashes. |

## Admin key custody

The deployed `__constructor` stores the same `STELLAR_ADMIN_ADDRESS` on both
contracts. Loss or compromise of this key is launch-critical; key custody must
be decided and documented before deployment.

| Done | Owner | Item | Required evidence |
| --- | --- | --- | --- |
| [ ] | Operations lead | Name the admin G-address and verify it belongs to the approved custody solution. | Verified G-address and custody record. |
| [ ] | Operations lead | Document who can initiate an admin action, who must approve it, and how multi-person approval is enforced. | Custody/MFA/multisig runbook. |
| [ ] | Operations lead | Store recovery material according to the organization's key-loss and compromise policy. | Secured recovery plan and test date. |
| [ ] | Security lead | Confirm the deployer identity is not used as the admin unless explicitly approved. | Address separation review. |
| [ ] | Operations lead | Test the `propose_admin` / `accept_admin` / `cancel_admin_proposal` procedure on testnet or in an approved signing exercise. | Test transaction IDs and runbook validation. |
| [ ] | Operations lead | Decide whether the vault admin may ever renounce admin and whether this is permanently irreversible. | Written renounce decision. |
| [ ] | Operations lead | Assign custody for both vault and registry upgrade, pause, fee, treasury, registry, and approver operations. | Privileged-operation matrix. |

## NGO registry and verification

| Done | Owner | Item | Required evidence |
| --- | --- | --- | --- |
| [ ] | NGO operations owner | Decide whether `STELLAR_REGISTRY_ADDRESS` will be `auto`, an existing registry address, or intentionally unset. | Registry configuration decision. |
| [ ] | Security lead | If no registry is configured, explicitly approve the `--allow-no-registry` exception because this allows streams to unverified addresses. | Signed exception with start/end owner and rollback plan. |
| [ ] | NGO operations owner | Confirm the registry's approver list and NGO verification process are ready before/at deployment. | Approver names, review process, and test cases. |
| [ ] | Operations lead | Add an alert for `registry()` returning no address or for configuration changing unexpectedly. | Alert rule and test notification. |

## Fee and treasury settings

The launch deploy supports `STELLAR_FEE_BPS`,
`STELLAR_TREASURY_ADDRESS`, and `STELLAR_MIN_DEPOSIT`. A non-zero fee requires
a treasury address. Fee behavior is documented in [FEES.md](./FEES.md).

| Done | Owner | Item | Required evidence |
| --- | --- | --- | --- |
| [ ] | Finance/Treasury owner | Decide the launch `STELLAR_FEE_BPS` value (0-1,000 basis points, maximum 10%). | Approved fee value and rationale. |
| [ ] | Finance/Treasury owner | Decide the launch `STELLAR_TREASURY_ADDRESS` G-address or C-address and verify custody. | Verified address and custody record. |
| [ ] | Finance/Treasury owner | Confirm whether a fee of zero is intentional; changing it later requires an admin action. | Configuration approval. |
| [ ] | Operations lead | Decide and record `STELLAR_MIN_DEPOSIT` in stroops. | Approved value, including units. |
| [ ] | Engineering lead | Confirm fee, treasury, and minimum-deposit values to use in the deploy environment. | Reviewed deploy command/environment, with secrets handled securely. |
| [ ] | Monitoring owner | Add fee/treasury configuration and `total_fees_collected` monitoring. | Dashboard and alert links. |

## Pause and incident plan

Only `donation-vault` has a pause switch. Pausing does not stop accrual or make
funds disappear; it rejects new streams and withdrawals while allowing donor
cancel and emergency rescue paths. The registry has no pause switch.

| Done | Owner | Item | Required evidence |
| --- | --- | --- | --- |
| [ ] | Incident commander | Document who may declare a pause and who may approve unpause. | Incident authority matrix. |
| [ ] | Incident commander | Define pause triggers: suspected exploit, accounting mismatch, misconfigured registry or treasury, or unhealthy upstream dependencies. | Written trigger thresholds. |
| [ ] | Operations lead | Test `pause`, view-only behavior, `cancel_stream`, and `unpause` on testnet with the launch admin. | Testnet transaction IDs and operator sign-off. |
| [ ] | Incident commander | Document that pausing halts vault withdrawals/create/top-up/modify actions but not accrued time. | Incident runbook. |
| [ ] | Operations lead | Define the `rescue_stream` approval, scope, and audit requirements before it is ever used. | Rescue runbook and audit logging method. |
| [ ] | Incident commander | Define escalation, communications, and user/refund guidance for a pause incident. | Incident communications template. |

## Monitoring and alerting

| Done | Owner | Item | Required evidence |
| --- | --- | --- | --- |
| [ ] | Monitoring owner | Monitor contract availability and Horizon/RPC/Soroban transaction health. | Dashboard and alert routing. |
| [ ] | Monitoring owner | Alert on pause/unpause, admin transfer, upgrade, fee, treasury, registry, and minimum-deposit changes. | Alert rules with contract event coverage. |
| [ ] | Monitoring owner | Monitor stream creation/cancellation, withdrawals, fees, and vault balance versus committed stream balances. | Dashboard and reconciliation job. |
| [ ] | Monitoring owner | Monitor NGO registration, verification, revocation, and approver changes. | Registry dashboard and alert rules. |
| [ ] | Monitoring owner | Monitor archive/TTL risk and run scheduled `extend_stream` / `touch_ngo` operations for critical entries. | Keeper schedule and failure alerts. |
| [ ] | Operations lead | Define paging severity, on-call ownership, and response targets for admin changes, paused state, and accounting mismatch. | On-call rota and escalation policy. |
| [ ] | Operations lead | Establish log and alert retention, plus protected access to signing and incident records. | Retention and access-control evidence. |

## Deployment

| Done | Owner | Item | Required evidence |
| --- | --- | --- | --- |
| [ ] | Release manager | Perform a dry run or testnet deployment using the same script and configuration shape. | Testnet deployment IDs and resulting configuration. |
| [ ] | Release manager | Prepare the deploy command with approved environment values and `--confirm`. | Reviewed command with secrets removed. |
| [ ] | Release manager | Have a rollback/partial-deploy coordinator watch the deploy and capture contract IDs if a later step fails. | Named responder and communication channel. |
| [ ] | Operations lead | Run the script from the repository root and preserve complete build and invocation logs. | Deploy log. |
| [ ] | Engineering lead | If deployment fails part-way, record all emitted contract IDs and do not run the script again until recovery is approved. | Incident/partial-deploy record. |

## Post-deployment verification

| Done | Owner | Item | Required evidence |
| --- | --- | --- | --- |
| [ ] | Engineering lead | Confirm `deployments.json` contains both contract IDs and both reviewed Wasm hashes under `mainnet`. | Updated JSON diff. |
| [ ] | Engineering lead | Verify the vault's `admin()` equals `STELLAR_ADMIN_ADDRESS`. | Transaction/read output. |
| [ ] | Engineering lead | Verify the registry and vault contract IDs, network passphrase, and Wasm hashes on a public explorer/RPC. | Verified on-chain IDs and hashes. |
| [ ] | Engineering lead | Verify `registry()`, `treasury()`, `fee_bps()`, and `min_deposit()` match the approved launch values. | Read-back output. |
| [ ] | Operations lead | If configured, verify the vault registry points to the approved registry contract/address. | Read-back and configuration approval. |
| [ ] | Engineering lead | Confirm the contracts are initialized and that public smoke tests do not require mainnet funds unexpectedly. | Smoke-test transaction IDs. |
| [ ] | Monitoring owner | Verify monitoring receives live events and test alerts for pause/admin/configuration changes. | Test alert receipt. |
| [ ] | Operations lead | Publish contract IDs and Wasm hashes to integrators only after verification succeeds. | Announcement or release record. |
| [ ] | Release manager | Obtain the final post-deploy go-live sign-off. | Dated sign-off from release manager and operations lead. |
