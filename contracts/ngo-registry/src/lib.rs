// SPDX-License-Identifier: Apache-2.0
#![no_std]
use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype, Address, Bytes, BytesN,
    Env, String, Vec,
};

#[contractevent(topics = ["register"], data_format = "single-value")]
pub struct RegisterEvent {
    #[topic]
    pub owner: Address,
    pub name: String,
}

#[contractevent(topics = ["renamed"], data_format = "single-value")]
pub struct RenamedEvent {
    #[topic]
    pub owner: Address,
    pub name: String,
}

#[contractevent(topics = ["approved"], data_format = "single-value")]
pub struct ApprovedEvent {
    #[topic]
    pub owner: Address,
    pub data: (),
}

#[contractevent(topics = ["revoked"], data_format = "single-value")]
pub struct RevokedEvent {
    #[topic]
    pub owner: Address,
    pub data: (),
}

#[contractevent(topics = ["unregist"], data_format = "single-value")]
pub struct UnregisteredEvent {
    #[topic]
    pub owner: Address,
    pub data: (),
}

#[contractevent(topics = ["removed"], data_format = "single-value")]
pub struct RemovedEvent {
    #[topic]
    pub owner: Address,
    pub data: (),
}

#[contractevent(topics = ["rejected"], data_format = "single-value")]
pub struct RejectedEvent {
    #[topic]
    pub owner: Address,
    pub reason_code: u32,
}

#[contractevent(topics = ["propadmin"], data_format = "single-value")]
pub struct ProposedAdminEvent {
    pub admin: Address,
}

#[contractevent(topics = ["acptadmin"], data_format = "single-value")]
pub struct AcceptedAdminEvent {
    pub admin: Address,
}

#[contractevent(topics = ["canceladm"], data_format = "single-value")]
pub struct CancelledAdminEvent {
    pub data: (),
}

#[contractevent(topics = ["init"], data_format = "single-value")]
pub struct InitializedEvent {
    pub admin: Address,
}

#[contractevent(topics = ["upgrade"], data_format = "single-value")]
pub struct UpgradedEvent {
    pub new_wasm_hash: BytesN<32>,
}

#[contractevent(topics = ["apadd"], data_format = "single-value")]
pub struct ApproverAddedEvent {
    #[topic]
    pub approver: Address,
    pub data: (),
}

#[contractevent(topics = ["aprem"], data_format = "single-value")]
pub struct ApproverRemovedEvent {
    #[topic]
    pub approver: Address,
    pub data: (),
}

#[contractevent(topics = ["metadata"], data_format = "single-value")]
pub struct MetadataUriSetEvent {
    #[topic]
    pub owner: Address,
    pub metadata_uri: Option<String>,
}

#[contractevent(topics = ["renew"], data_format = "single-value")]
pub struct RenewedEvent {
    #[topic]
    pub owner: Address,
    /// The new verification expiry, as a ledger timestamp.
    pub verified_until: u64,
}

#[contractevent(topics = ["verperiod"], data_format = "single-value")]
pub struct VerifyPeriodSetEvent {
    pub verify_period: u64,
}

#[contracttype]
// Debug and PartialEq let tests assert_eq! on a try_* callâ€™s full
// Result<Result<Ngo, _>, _> rather than unwrapping it by hand first.
#[derive(Clone, Debug, PartialEq)]
pub struct Ngo {
    pub owner: Address,
    pub name: String,
    pub verified: bool,
    /// Optional owner-supplied reference to off-chain metadata (a website,
    /// description, or logo), so an entry can be linked to what the NGO
    /// publishes about itself. `None` until the owner calls
    /// `set_metadata_uri`. Bounded by `MAX_METADATA_URI_LEN`. See issue #347.
    pub metadata_uri: Option<String>,
    /// Optional ledger timestamp at which this verification lapses. Set by
    /// `approve_ngo` / `batch_approve` from the configured verification period
    /// (see `set_verify_period`), and moved forward by `renew_ngo`. `None` means
    /// the verification never expires. Once the current ledger timestamp
    /// reaches it the entry is treated as unverified by `is_verified` and
    /// `get_ngo`, until an admin renews it. See issue #348.
    pub verified_until: Option<u64>,
}

/// A single page of the NGO owners index, returned by `list_ngos_page`.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct NgoOwnerPage {
    /// The owner addresses in this page, in registry order.
    pub owners: Vec<Address>,
    /// The `start` to pass on the next call: one past the end of this page,
    /// clamped to the current owner count. Iteration is complete once it
    /// reaches `total_ngos()` (or `ngo_count()`).
    pub next: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    /// The address authorized to initialize and administer the registry.
    Admin,
    /// The administrator nominated to take over, pending acceptance.
    PendingAdmin,
    /// Unused and reserved. Preserved to maintain enum discriminant stability
    /// for existing deployed contracts. Callers should read `NgoCount` via `ngo_count()`.
    TotalNgos,
    /// The registry record keyed by an NGO owner's address.
    Ngo(Address),
    /// The total number of NGO records stored in the registry.
    NgoCount,
    /// Current number of registered NGO owners in the index.
    OwnerIndexCount,
    /// Registered NGO owner at a specific index in the owners index.
    OwnerByIndex(u32),
    /// Index of a registered NGO owner in the owners index.
    IndexByOwner(Address),
    /// Addresses (other than the admin) permitted to approve and revoke NGOs.
    Approvers,
    /// Current number of verified NGO records, maintained by the review and
    /// removal entry points and read by `verified_count`.
    VerifiedCount,
    /// The verification period in seconds applied to new approvals and to
    /// `renew_ngo`. `0` (the default) means approvals never expire. See
    /// `set_verify_period`.
    VerifyPeriod,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    AlreadyRegistered = 3,
    NotRegistered = 4,
    /// The NGO has already been approved: it can no longer be unregistered,
    /// renamed, rejected, or approved again. Revoke it first.
    AlreadyVerified = 5,
    /// `name` is longer than `MAX_NGO_NAME_LEN` (200 bytes).
    NameTooLong = 6,
    /// `revoke_ngo` was called on an NGO that isn't currently verified.
    /// The NGO has not been approved, so it cannot be revoked.
    NotVerified = 7,
    /// `accept_admin` or `cancel_admin_proposal` was called without a prior
    /// (or already-completed) `propose_admin`.
    NoPendingAdmin = 8,
    /// The proposed administrator is not a valid replacement.
    InvalidAdmin = 9,
    ArithmeticOverflow = 10,
    /// The supplied name is empty, contains only Unicode whitespace or invisible
    /// format characters, contains control characters, or is invalid UTF-8.
    InvalidName = 11,
    /// `approve_ngo`, `batch_approve`, or `revoke_ngo` was called by an address
    /// that is neither the admin nor a listed approver.
    NotApprover = 12,
    /// `add_approver` was called for an address already in the approver list.
    AlreadyApprover = 13,
    /// The supplied metadata URI is longer than `MAX_METADATA_URI_LEN`
    /// (512 bytes).
    MetadataUriTooLong = 14,
    /// `renew_ngo` was called while no verification period is configured, so
    /// there is no expiry to extend. Set one with `set_verify_period`.
    NoVerifyPeriod = 15,
    // Error numbers are append-only: never renumber, reuse, or remove a
    // variant. New errors take the next unused number. See
    // docs/UPGRADE_POLICY.md.
}

/// Upper bound on `Ngo.name`, in bytes. Persistent storage cost scales with
/// what's stored, so without a cap a registration could inflate its own
/// entry's storage footprint indefinitely. Comfortably fits a real
/// organization name while keeping a single entry's storage bounded.
/// Note: this is strictly a byte limit (200 bytes), not a character limit.
pub const MAX_NGO_NAME_LEN: u32 = 200;

/// Upper bound on `Ngo.metadata_uri`, in bytes. As with `MAX_NGO_NAME_LEN`,
/// this keeps a single entry's storage footprint bounded; 512 bytes fits a
/// typical URL (including a logo or IPFS reference) with room to spare.
pub const MAX_METADATA_URI_LEN: u32 = 512;

/// Maximum number of NGO owners returned by `list_ngos` / `list_ngos_page` in
/// a single page. Caps read footprint, memory, and gas consumption to keep
/// pagination bounded.
pub const MAX_LIST_LIMIT: u32 = 50;

/// Approximate ledgers per day at a 5-second close time. Used to express
/// storage TTLs (which the network counts in ledgers, not wall time) in
/// human terms.
const DAY_IN_LEDGERS: u32 = 17_280;

const INSTANCE_BUMP_AMOUNT: u32 = 30 * DAY_IN_LEDGERS;
const INSTANCE_LIFETIME_THRESHOLD: u32 = INSTANCE_BUMP_AMOUNT - DAY_IN_LEDGERS;

const NGO_BUMP_AMOUNT: u32 = 90 * DAY_IN_LEDGERS;
const NGO_LIFETIME_THRESHOLD: u32 = NGO_BUMP_AMOUNT - DAY_IN_LEDGERS;

fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

fn extend_ngo_ttl(env: &Env, owner: &Address) {
    env.storage().persistent().extend_ttl(
        &DataKey::Ngo(owner.clone()),
        NGO_LIFETIME_THRESHOLD,
        NGO_BUMP_AMOUNT,
    );
}

fn extend_owner_index_ttl(env: &Env, index: u32, owner: &Address) {
    env.storage().persistent().extend_ttl(
        &DataKey::OwnerByIndex(index),
        NGO_LIFETIME_THRESHOLD,
        NGO_BUMP_AMOUNT,
    );
    env.storage().persistent().extend_ttl(
        &DataKey::IndexByOwner(owner.clone()),
        NGO_LIFETIME_THRESHOLD,
        NGO_BUMP_AMOUNT,
    );
}

fn get_owner_count(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::OwnerIndexCount)
        .unwrap_or(0)
}

fn set_owner_count(env: &Env, count: u32) {
    env.storage()
        .instance()
        .set(&DataKey::OwnerIndexCount, &count);
}

fn get_verified_count(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::VerifiedCount)
        .unwrap_or(0)
}

fn set_verified_count(env: &Env, count: u64) {
    env.storage()
        .instance()
        .set(&DataKey::VerifiedCount, &count);
}

/// Reads the configured verification period in seconds. `0` (the default)
/// means approvals never expire. See `set_verify_period`.
fn get_verify_period(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::VerifyPeriod)
        .unwrap_or(0)
}

/// Returns whether a verification expiry has lapsed at the current ledger
/// time. `None` never expires. An entry stays valid while
/// `now < verified_until`, so it is treated as unverified once `now` reaches
/// the stored timestamp. See issue #348.
fn has_expired(env: &Env, verified_until: Option<u64>) -> bool {
    match verified_until {
        Some(until) => env.ledger().timestamp() >= until,
        None => false,
    }
}

/// Returns whether a code point is an invisible or format character (category Cf).
///
/// Deliberately limited to zero-width spaces, joiners, word joiners, soft hyphens,
/// and byte order marks. These are treated as invisible (whitespace-like) for the
/// emptiness check: a name consisting solely of these characters and/or Unicode whitespace
/// is rejected. When present alongside visible characters, they are permitted (e.g.
/// for complex script ligatures).
/// Note: bidirectional override characters (such as U+202E) are not handled here and
/// should be tracked as a follow-up spoofing consideration.
fn is_invisible_format(c: char) -> bool {
    matches!(
        c,
        '\u{200B}' // Zero Width Space
        | '\u{200C}' // Zero Width Non-Joiner
        | '\u{200D}' // Zero Width Joiner
        | '\u{2060}' // Word Joiner
        | '\u{FEFF}' // Zero Width No-Break Space (BOM)
        | '\u{00AD}' // Soft Hyphen
        | '\u{200E}' // Left-to-Right Mark
        | '\u{200F}' // Right-to-Left Mark
    )
}

/// Validates a raw byte slice as an NGO name.
///
/// Rejects names longer than `MAX_NGO_NAME_LEN` (200 bytes) with `Error::NameTooLong`.
/// Rejects names that are empty, invalid UTF-8, contain any control character
/// (Unicode category Cc), or consist solely of Unicode whitespace and/or invisible
/// format characters with `Error::InvalidName`.
pub(crate) fn validate_name_bytes(bytes: &[u8]) -> Result<(), Error> {
    if bytes.len() > MAX_NGO_NAME_LEN as usize {
        return Err(Error::NameTooLong);
    }
    if bytes.is_empty() {
        return Err(Error::InvalidName);
    }
    let s = core::str::from_utf8(bytes).map_err(|_| Error::InvalidName)?;
    let mut has_visible = false;
    for c in s.chars() {
        if c.is_control() {
            return Err(Error::InvalidName);
        }
        if c.is_whitespace() || is_invisible_format(c) {
            continue;
        }
        has_visible = true;
    }
    if !has_visible {
        return Err(Error::InvalidName);
    }
    Ok(())
}

/// Validates an NGO name in Soroban SDK `String` format.
///
/// Copies into a stack buffer bounded by `MAX_NGO_NAME_LEN` (200 bytes)
/// without heap allocations. Replaces `is_whitespace_only`.
fn validate_name(name: &String) -> Result<(), Error> {
    if name.len() > MAX_NGO_NAME_LEN {
        return Err(Error::NameTooLong);
    }
    if name.is_empty() {
        return Err(Error::InvalidName);
    }
    let len = name.len() as usize;
    let mut buf = [0u8; MAX_NGO_NAME_LEN as usize];
    let bytes: Bytes = name.into();
    bytes.copy_into_slice(&mut buf[..len]);
    validate_name_bytes(&buf[..len])
}

/// Validates an optional NGO metadata URI.
///
/// `None` is always allowed. `Some` must be at most `MAX_METADATA_URI_LEN`
/// (512 bytes) or the call fails with `Error::MetadataUriTooLong`. The URI is
/// opaque to the contract â€” an `https://`, `ipfs://`, or any other scheme is
/// accepted â€” so only the length is enforced. See issue #347.
fn validate_metadata_uri(uri: &Option<String>) -> Result<(), Error> {
    if let Some(u) = uri {
        if u.len() > MAX_METADATA_URI_LEN {
            return Err(Error::MetadataUriTooLong);
        }
    }
    Ok(())
}

/// Reads the configured admin and requires their auth, failing with
/// `Error::NotInitialized` if `init` hasn't been called yet. Shared by
/// every admin-gated entry point so the same three steps aren't repeated
/// at each call site.
fn require_admin(env: &Env) -> Result<Address, Error> {
    let admin: Address = env
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(Error::NotInitialized)?;
    admin.require_auth();
    Ok(admin)
}

/// Reads the stored approver list, or an empty list if none was set.
fn read_approvers(env: &Env) -> Vec<Address> {
    env.storage()
        .instance()
        .get(&DataKey::Approvers)
        .unwrap_or_else(|| Vec::new(env))
}

/// Returns whether `candidate` is in the approver list. The admin is not
/// stored here, so this reflects explicit approvers only.
fn is_approver_address(env: &Env, candidate: &Address) -> bool {
    read_approvers(env).contains(candidate)
}

/// Authorizes a review action (approve/revoke). The acting address must be
/// either the admin or a listed approver, and must have authorized the call.
/// Because a contract can't observe its own caller, the acting address is
/// passed in by the caller and checked here: this is what lets review work be
/// delegated without handing out the admin key. Fails with
/// `Error::NotApprover` for anyone else.
fn require_approver(env: &Env, candidate: &Address) -> Result<(), Error> {
    let admin: Address = env
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(Error::NotInitialized)?;
    if *candidate != admin && !is_approver_address(env, candidate) {
        return Err(Error::NotApprover);
    }
    candidate.require_auth();
    Ok(())
}

/// Marks one registered, unverified NGO as verified and publishes its
/// `approved` event. Shared by `approve_ngo` and `batch_approve` so a single
/// approval behaves the same whichever entry point drives it: an address with
/// no entry fails with `Error::NotRegistered`, and one that is already
/// verified with `Error::AlreadyVerified`.
fn approve_registered_ngo(env: &Env, ngo_owner: Address) -> Result<(), Error> {
    let key = DataKey::Ngo(ngo_owner.clone());
    let mut ngo: Ngo = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(Error::NotRegistered)?;
    if ngo.verified {
        return Err(Error::AlreadyVerified);
    }
    ngo.verified = true;
    // Stamp the verification expiry from the configured period (issue #348).
    // A zero period leaves `verified_until` as `None`, i.e. no expiry, which
    // is the default and matches the pre-expiry behaviour.
    let period = get_verify_period(env);
    ngo.verified_until = if period == 0 {
        None
    } else {
        Some(
            env.ledger()
                .timestamp()
                .checked_add(period)
                .ok_or(Error::ArithmeticOverflow)?,
        )
    };
    env.storage().persistent().set(&key, &ngo);

    let verified = get_verified_count(env);
    let new_verified = verified.checked_add(1).ok_or(Error::ArithmeticOverflow)?;
    set_verified_count(env, new_verified);

    extend_instance_ttl(env);
    extend_ngo_ttl(env, &ngo_owner);

    ApprovedEvent {
        owner: ngo_owner,
        data: (),
    }
    .publish(env);

    Ok(())
}

/// Deletes an NGO's registry entry and its position in the owners index,
/// shrinking the live counts. Shared by `unregister` (owner-initiated),
/// `remove_ngo` (admin-initiated) and `reject_ngo` (reviewer-initiated); the
/// caller is responsible for authorizing the call and for the
/// verified/unverified check. Does not touch the instance TTL â€” the caller
/// decides whether to bump it. Decrements `NgoCount`, and `VerifiedCount` when
/// the entry being removed was verified, using checked subtraction and
/// returning `Error::ArithmeticOverflow` on underflow.
fn delete_ngo_entry(env: &Env, owner: &Address) -> Result<(), Error> {
    let count: u64 = env
        .storage()
        .instance()
        .get(&DataKey::NgoCount)
        .unwrap_or(0);
    let new_ngo_count = count.checked_sub(1).ok_or(Error::ArithmeticOverflow)?;

    // Keep `verified_count` in step with the entry being removed.
    if let Some(ngo) = env
        .storage()
        .persistent()
        .get::<_, Ngo>(&DataKey::Ngo(owner.clone()))
    {
        if ngo.verified {
            let verified = get_verified_count(env);
            let new_verified = verified.checked_sub(1).ok_or(Error::ArithmeticOverflow)?;
            set_verified_count(env, new_verified);
        }
    }

    env.storage()
        .persistent()
        .remove(&DataKey::Ngo(owner.clone()));

    env.storage()
        .instance()
        .set(&DataKey::NgoCount, &new_ngo_count);

    // Remove from index using swap-and-pop to maintain a dense [0..count) index
    if let Some(index) = env
        .storage()
        .persistent()
        .get::<_, u32>(&DataKey::IndexByOwner(owner.clone()))
    {
        let count = get_owner_count(env);
        if count > 0 {
            let last_index = count.checked_sub(1).ok_or(Error::ArithmeticOverflow)?;
            if index != last_index {
                if let Some(last_owner) = env
                    .storage()
                    .persistent()
                    .get::<_, Address>(&DataKey::OwnerByIndex(last_index))
                {
                    env.storage()
                        .persistent()
                        .set(&DataKey::OwnerByIndex(index), &last_owner);
                    env.storage()
                        .persistent()
                        .set(&DataKey::IndexByOwner(last_owner.clone()), &index);
                    extend_owner_index_ttl(env, index, &last_owner);
                }
            }
            env.storage()
                .persistent()
                .remove(&DataKey::OwnerByIndex(last_index));
            env.storage()
                .persistent()
                .remove(&DataKey::IndexByOwner(owner.clone()));
            set_owner_count(env, last_index);
        }
    }

    Ok(())
}

#[contract]
pub struct NgoRegistry;

#[contractimpl]
impl NgoRegistry {
    /// Initializes the registry with an administrator atomically at deployment time.
    pub fn __constructor(env: Env, admin: Address) {
        env.storage().instance().set(&DataKey::Admin, &admin);
        extend_instance_ttl(&env);
        InitializedEvent { admin }.publish(&env);
    }

    /// Sets the registry admin. Can only be called once. Neutralized in favor of
    /// `__constructor` for atomic deployment; requires admin auth.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryArgs, NgoRegistryClient};
    /// let env = Env::default();
    /// env.mock_all_auths();
    ///
    /// let admin = Address::generate(&env);
    /// let contract_id = env.register(NgoRegistry, NgoRegistryArgs::__constructor(&admin));
    /// let client = NgoRegistryClient::new(&env, &contract_id);
    /// ```
    pub fn init(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        extend_instance_ttl(&env);
        InitializedEvent { admin }.publish(&env);
        Ok(())
    }

    /// Reads back the registry admin set by `init`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let contract_id = env.register(NgoRegistry, ());
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// # let admin = Address::generate(&env);
    /// # client.init(&admin);
    /// assert_eq!(client.admin(), admin);
    /// ```
    pub fn admin(env: Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)
    }

    /// Reads back the address proposed by `propose_admin`, if any hasn't
    /// yet been accepted or cancelled.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let contract_id = env.register(NgoRegistry, ());
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// # let admin = Address::generate(&env);
    /// # client.init(&admin);
    /// assert_eq!(client.pending_admin(), None);
    ///
    /// let new_admin = Address::generate(&env);
    /// client.propose_admin(&new_admin);
    /// assert_eq!(client.pending_admin(), Some(new_admin));
    /// ```
    pub fn pending_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::PendingAdmin)
    }

    /// Starts a two-step admin transfer by recording `new_admin` as pending.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let contract_id = env.register(NgoRegistry, ());
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// # let admin = Address::generate(&env);
    /// # client.init(&admin);
    /// let new_admin = Address::generate(&env);
    /// client.propose_admin(&new_admin);
    /// // The old admin is still in charge until accept_admin is called.
    /// assert_eq!(client.admin(), admin);
    /// ```
    pub fn propose_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        let current_admin = require_admin(&env)?;
        if new_admin == current_admin {
            return Err(Error::InvalidAdmin);
        }

        env.storage()
            .instance()
            .set(&DataKey::PendingAdmin, &new_admin);
        extend_instance_ttl(&env);

        ProposedAdminEvent { admin: new_admin }.publish(&env);

        Ok(())
    }

    /// Completes a two-step admin transfer.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let contract_id = env.register(NgoRegistry, ());
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// # let admin = Address::generate(&env);
    /// # client.init(&admin);
    /// let new_admin = Address::generate(&env);
    /// client.propose_admin(&new_admin);
    /// client.accept_admin();
    /// assert_eq!(client.admin(), new_admin);
    /// ```
    pub fn accept_admin(env: Env) -> Result<(), Error> {
        let pending: Address = env
            .storage()
            .instance()
            .get(&DataKey::PendingAdmin)
            .ok_or(Error::NoPendingAdmin)?;
        pending.require_auth();

        env.storage().instance().set(&DataKey::Admin, &pending);
        env.storage().instance().remove(&DataKey::PendingAdmin);
        extend_instance_ttl(&env);

        AcceptedAdminEvent { admin: pending }.publish(&env);

        Ok(())
    }

    /// Withdraws a pending admin proposal, leaving nothing pending.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let contract_id = env.register(NgoRegistry, ());
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// # let admin = Address::generate(&env);
    /// # client.init(&admin);
    /// let new_admin = Address::generate(&env);
    /// client.propose_admin(&new_admin);
    ///
    /// // The admin changes their mind before it's accepted.
    /// client.cancel_admin_proposal();
    ///
    /// // Nothing left to accept.
    /// let result = client.try_accept_admin();
    /// assert!(result.is_err());
    /// ```
    pub fn cancel_admin_proposal(env: Env) -> Result<(), Error> {
        require_admin(&env)?;

        if !env.storage().instance().has(&DataKey::PendingAdmin) {
            return Err(Error::NoPendingAdmin);
        }
        env.storage().instance().remove(&DataKey::PendingAdmin);
        extend_instance_ttl(&env);

        CancelledAdminEvent { data: () }.publish(&env);

        Ok(())
    }

    /// Submits an NGO application. Callable by the NGO's own address.
    /// The entry starts unverified until an admin approves it. Names must be
    /// at most 200 bytes (`MAX_NGO_NAME_LEN`), valid UTF-8, contain at least one visible
    /// character (not empty or made only of Unicode whitespace or invisible format characters),
    /// and contain no control characters. Names are not trimmed.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env, String};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let contract_id = env.register(NgoRegistry, ());
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// # let admin = Address::generate(&env);
    /// # client.init(&admin);
    /// let owner = Address::generate(&env);
    /// let name = String::from_str(&env, "Example NGO");
    /// client.register(&owner, &name);
    ///
    /// let ngo = client.get_ngo(&owner);
    /// assert_eq!(ngo.name, name);
    /// assert!(!ngo.verified);
    /// ```
    pub fn register(env: Env, owner: Address, name: String) -> Result<(), Error> {
        owner.require_auth();

        validate_name(&name)?;

        let key = DataKey::Ngo(owner.clone());
        if env.storage().persistent().has(&key) {
            return Err(Error::AlreadyRegistered);
        }

        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NgoCount)
            .unwrap_or(0);
        let new_ngo_count = count.checked_add(1).ok_or(Error::ArithmeticOverflow)?;

        let index = get_owner_count(&env);
        let new_count = index.checked_add(1).ok_or(Error::ArithmeticOverflow)?;

        let ngo = Ngo {
            owner: owner.clone(),
            name: name.clone(),
            verified: false,
            metadata_uri: None,
            verified_until: None,
        };
        env.storage().persistent().set(&key, &ngo);

        env.storage()
            .instance()
            .set(&DataKey::NgoCount, &new_ngo_count);

        // Maintain the registered owners index
        env.storage()
            .persistent()
            .set(&DataKey::OwnerByIndex(index), &owner);
        env.storage()
            .persistent()
            .set(&DataKey::IndexByOwner(owner.clone()), &index);
        set_owner_count(&env, new_count);

        extend_instance_ttl(&env);
        extend_ngo_ttl(&env, &owner);
        extend_owner_index_ttl(&env, index, &owner);

        RegisterEvent { owner, name }.publish(&env);

        Ok(())
    }

    /// Deprecated alias of `ngo_count`.
    ///
    /// Reads the same single stored counter (`NgoCount`) as `ngo_count()`.
    /// Preserved for backwards compatibility with existing callers.
    ///
    /// # Conversion
    /// Returns the live count as `u32`. If the stored `u64` count exceeds
    /// `u32::MAX`, saturates at `u32::MAX`. Callers should prefer `ngo_count`.
    pub fn total_ngos(env: Env) -> u32 {
        u32::try_from(Self::ngo_count(env)).unwrap_or(u32::MAX)
    }

    /// Removes the caller's unverified NGO application.
    ///
    /// Verified registrations are intentionally permanent until an admin
    /// revokes verification.
    pub fn unregister(env: Env, owner: Address) -> Result<(), Error> {
        owner.require_auth();

        let key = DataKey::Ngo(owner.clone());
        let ngo: Ngo = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::NotRegistered)?;
        if ngo.verified {
            return Err(Error::AlreadyVerified);
        }

        delete_ngo_entry(&env, &owner)?;

        extend_instance_ttl(&env);
        UnregisteredEvent { owner, data: () }.publish(&env);

        Ok(())
    }

    /// Removes an unverified NGO entry entirely. Admin-only, so an admin can
    /// clear out an impersonating or abusive application without waiting for
    /// its owner to `unregister`. Returns `Error::NotRegistered` for an
    /// address with no entry and `Error::AlreadyVerified` for a verified one:
    /// an approval is undone with `revoke_ngo` first, so removal can never
    /// silently drop a verification an admin granted.
    ///
    /// The entry, its position in the owners index, and the live counts
    /// (`TotalNgos`/`OwnerIndexCount`) are all cleared, so the owner can
    /// register again. The lifetime `NgoCount` is left alone, matching
    /// `unregister`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env, String};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryArgs, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let admin = Address::generate(&env);
    /// # let contract_id = env.register(NgoRegistry, NgoRegistryArgs::__constructor(&admin));
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// let spammer = Address::generate(&env);
    /// client.register(&spammer, &String::from_str(&env, "Totally Real Charity"));
    ///
    /// // The admin removes the junk application outright.
    /// client.remove_ngo(&spammer);
    /// assert!(client.try_get_ngo(&spammer).is_err());
    /// ```
    pub fn remove_ngo(env: Env, ngo_owner: Address) -> Result<(), Error> {
        require_admin(&env)?;

        let key = DataKey::Ngo(ngo_owner.clone());
        let ngo: Ngo = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::NotRegistered)?;
        if ngo.verified {
            return Err(Error::AlreadyVerified);
        }

        delete_ngo_entry(&env, &ngo_owner)?;

        extend_instance_ttl(&env);
        RemovedEvent {
            owner: ngo_owner,
            data: (),
        }
        .publish(&env);

        Ok(())
    }

    /// Rejects a pending NGO application, removing its entry and publishing the
    /// supplied `reason_code` so the applicant's client can explain why it was
    /// not approved. Callable by the admin or any listed approver (see
    /// `add_approver`), and the counterpart to `approve_ngo` for a reviewer who
    /// decides against an application rather than leaving it pending.
    ///
    /// The entry, its position in the owners index, and the live counts are
    /// cleared exactly as in `remove_ngo`, so the owner can address the problem
    /// and register again. The `reason_code` is an opaque, caller-defined
    /// integer (for example, an off-chain enum agreed between the registry and
    /// its clients); the contract keeps no record of it beyond the event.
    ///
    /// Returns `Error::NotRegistered` for an address with no entry. A verified
    /// NGO cannot be rejected â€” an approval is undone with `revoke_ngo` first â€”
    /// and fails with `Error::AlreadyVerified`. An address that is neither the
    /// admin nor a listed approver fails with `Error::NotApprover`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env, String};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryArgs, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let admin = Address::generate(&env);
    /// # let contract_id = env.register(NgoRegistry, NgoRegistryArgs::__constructor(&admin));
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// let applicant = Address::generate(&env);
    /// client.register(&applicant, &String::from_str(&env, "Unverified Charity"));
    ///
    /// // 3 = "could not verify registration documents".
    /// client.reject_ngo(&admin, &applicant, &3);
    /// assert!(client.try_get_ngo(&applicant).is_err());
    /// ```
    pub fn reject_ngo(
        env: Env,
        approver: Address,
        ngo_owner: Address,
        reason_code: u32,
    ) -> Result<(), Error> {
        require_approver(&env, &approver)?;

        let key = DataKey::Ngo(ngo_owner.clone());
        let ngo: Ngo = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::NotRegistered)?;
        if ngo.verified {
            return Err(Error::AlreadyVerified);
        }

        delete_ngo_entry(&env, &ngo_owner)?;

        extend_instance_ttl(&env);
        RejectedEvent {
            owner: ngo_owner,
            reason_code,
        }
        .publish(&env);

        Ok(())
    }

    /// Adds an address to the approver list. Admin-only.
    ///
    /// Approvers may call `approve_ngo`, `batch_approve`, `reject_ngo`, and
    /// `revoke_ngo`
    /// without holding the admin key, which is how review work is delegated
    /// without granting control over upgrades or admin transfer. The admin is
    /// always able to review whether or not it appears in this list, so adding
    /// the admin is allowed but unnecessary. Fails with
    /// `Error::AlreadyApprover` if the address is already listed.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryArgs, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let admin = Address::generate(&env);
    /// # let contract_id = env.register(NgoRegistry, NgoRegistryArgs::__constructor(&admin));
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// let reviewer = Address::generate(&env);
    /// client.add_approver(&reviewer);
    /// assert!(client.is_approver(&reviewer));
    /// ```
    pub fn add_approver(env: Env, approver: Address) -> Result<(), Error> {
        require_admin(&env)?;

        let mut approvers = read_approvers(&env);
        if approvers.contains(&approver) {
            return Err(Error::AlreadyApprover);
        }
        approvers.push_back(approver.clone());
        env.storage()
            .instance()
            .set(&DataKey::Approvers, &approvers);
        extend_instance_ttl(&env);

        ApproverAddedEvent { approver, data: () }.publish(&env);

        Ok(())
    }

    /// Removes an address from the approver list. Admin-only.
    ///
    /// The address can no longer approve or revoke NGOs; any NGO it approved
    /// stays approved. Fails with `Error::NotApprover` if the address is not
    /// currently listed.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryArgs, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let admin = Address::generate(&env);
    /// # let contract_id = env.register(NgoRegistry, NgoRegistryArgs::__constructor(&admin));
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// let reviewer = Address::generate(&env);
    /// client.add_approver(&reviewer);
    /// client.remove_approver(&reviewer);
    /// assert!(!client.is_approver(&reviewer));
    /// ```
    pub fn remove_approver(env: Env, approver: Address) -> Result<(), Error> {
        require_admin(&env)?;

        let approvers = read_approvers(&env);
        let mut remaining = Vec::new(&env);
        let mut found = false;
        for listed in approvers.iter() {
            if listed == approver {
                found = true;
            } else {
                remaining.push_back(listed);
            }
        }
        if !found {
            return Err(Error::NotApprover);
        }
        env.storage()
            .instance()
            .set(&DataKey::Approvers, &remaining);
        extend_instance_ttl(&env);

        ApproverRemovedEvent { approver, data: () }.publish(&env);

        Ok(())
    }

    /// Returns whether an address is in the approver list.
    ///
    /// Reflects explicit approvers only: the admin can always approve and
    /// revoke NGOs, so this returns `false` for the admin unless it was also
    /// added with `add_approver`. Never errors or panics.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryArgs, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let admin = Address::generate(&env);
    /// # let contract_id = env.register(NgoRegistry, NgoRegistryArgs::__constructor(&admin));
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// let reviewer = Address::generate(&env);
    /// assert!(!client.is_approver(&reviewer));
    /// client.add_approver(&reviewer);
    /// assert!(client.is_approver(&reviewer));
    /// ```
    pub fn is_approver(env: Env, addr: Address) -> bool {
        is_approver_address(&env, &addr)
    }

    /// Returns every address in the approver list, in the order they were
    /// added. Returns an empty list when none are configured. Never errors or
    /// panics.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryArgs, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let admin = Address::generate(&env);
    /// # let contract_id = env.register(NgoRegistry, NgoRegistryArgs::__constructor(&admin));
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// let reviewer = Address::generate(&env);
    /// client.add_approver(&reviewer);
    /// assert_eq!(client.list_approvers().len(), 1);
    /// ```
    pub fn list_approvers(env: Env) -> Vec<Address> {
        read_approvers(&env)
    }

    /// Returns a paginated list of registered NGO owner addresses.
    ///
    /// # Pagination Semantics
    /// - `start`: The 0-based offset into the registered owners index. If `start` is
    ///   greater than or equal to the total number of registered owners, an empty list is returned.
    /// - `limit`: The maximum number of entries to return. Capped at `MAX_LIST_LIMIT` (50).
    ///   If `limit == 0`, an empty list is returned.
    /// - Ordering: Stable and deterministic. Entries are indexed in order of registration,
    ///   with unregistration swapping the last entry into the vacated position (swap-and-pop)
    ///   to keep persistent storage dense and bounded without reallocations.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env, String};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryArgs, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let admin = Address::generate(&env);
    /// # let contract_id = env.register(NgoRegistry, NgoRegistryArgs::__constructor(&admin));
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// let owner = Address::generate(&env);
    /// client.register(&owner, &String::from_str(&env, "Example NGO"));
    ///
    /// let owners = client.list_ngos(&0, &10);
    /// assert_eq!(owners.len(), 1);
    /// assert_eq!(owners.get(0).unwrap(), owner);
    /// ```
    pub fn list_ngos(env: Env, start: u32, limit: u32) -> Vec<Address> {
        let count = get_owner_count(&env);
        if start >= count || limit == 0 {
            return Vec::new(&env);
        }

        let effective_limit = limit.min(MAX_LIST_LIMIT);
        let end = start
            .checked_add(effective_limit)
            .unwrap_or(count)
            .min(count);

        let mut owners = Vec::new(&env);
        for index in start..end {
            if let Some(owner) = env
                .storage()
                .persistent()
                .get::<_, Address>(&DataKey::OwnerByIndex(index))
            {
                owners.push_back(owner);
            }
        }
        owners
    }

    /// Returns one page of registered NGO owner addresses together with the
    /// index to use for the following page.
    ///
    /// Same `start`/`limit` semantics, ordering, and `MAX_LIST_LIMIT` cap as
    /// `list_ngos`, but instead of leaving callers to recompute the cursor it
    /// also returns `next`, the `start` for the next call. Because `next`
    /// reaches the total owner count on the final page, a caller can tell the
    /// end of the list apart from a full page without a follow-up call.
    ///
    /// `next` is the exclusive end of this page (one past its last owner),
    /// clamped to the owner count. Iteration is complete once
    /// `next >= total_ngos()`. An out-of-range `start` or a `limit == 0`
    /// returns an empty `owners` and `next` equal to the owner count, so the
    /// caller stops immediately.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env, String};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryArgs, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let admin = Address::generate(&env);
    /// # let contract_id = env.register(NgoRegistry, NgoRegistryArgs::__constructor(&admin));
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// let owner = Address::generate(&env);
    /// client.register(&owner, &String::from_str(&env, "Example NGO"));
    ///
    /// let page = client.list_ngos_page(&0, &10);
    /// assert_eq!(page.owners.get(0).unwrap(), owner);
    /// assert_eq!(page.next, 1);
    /// ```
    pub fn list_ngos_page(env: Env, start: u32, limit: u32) -> NgoOwnerPage {
        let count = get_owner_count(&env);
        if start >= count || limit == 0 {
            return NgoOwnerPage {
                owners: Vec::new(&env),
                next: count,
            };
        }

        let effective_limit = limit.min(MAX_LIST_LIMIT);
        let end = start
            .checked_add(effective_limit)
            .unwrap_or(count)
            .min(count);

        let mut owners = Vec::new(&env);
        for index in start..end {
            if let Some(owner) = env
                .storage()
                .persistent()
                .get::<_, Address>(&DataKey::OwnerByIndex(index))
            {
                owners.push_back(owner);
            }
        }

        NgoOwnerPage { owners, next: end }
    }

    /// Changes the name on an NGO's own pending application, so a typo or
    /// rename can be fixed without going through an admin. Requires the
    /// owner's auth. Fails with `Error::NotRegistered` for an address with
    /// no entry, and `Error::AlreadyVerified` once an admin has approved
    /// it â€” the name an admin approved is the name that stays.
    ///
    /// The new name must meet the same validation rules as in `register`: at most
    /// 200 bytes, valid UTF-8, at least one visible character, and no control characters.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env, String};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let contract_id = env.register(NgoRegistry, ());
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// # let admin = Address::generate(&env);
    /// # client.init(&admin);
    /// # let owner = Address::generate(&env);
    /// client.register(&owner, &String::from_str(&env, "Exmaple NGO"));
    ///
    /// let fixed = String::from_str(&env, "Example NGO");
    /// client.update_name(&owner, &fixed);
    /// assert_eq!(client.get_ngo(&owner).name, fixed);
    /// ```
    pub fn update_name(env: Env, owner: Address, name: String) -> Result<(), Error> {
        owner.require_auth();

        validate_name(&name)?;

        let key = DataKey::Ngo(owner.clone());
        let mut ngo: Ngo = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::NotRegistered)?;
        if ngo.verified {
            return Err(Error::AlreadyVerified);
        }

        // Only update and emit event if the name actually changes
        if ngo.name != name {
            ngo.name = name.clone();
            env.storage().persistent().set(&key, &ngo);
            extend_instance_ttl(&env);
            extend_ngo_ttl(&env, &owner);
            RenamedEvent { owner, name }.publish(&env);
        }

        Ok(())
    }

    /// Sets or clears the metadata URI on the caller's own NGO entry, so an
    /// entry can point at off-chain metadata such as a website, description,
    /// or logo. Requires the owner's auth; the entry must already be
    /// registered, or the call fails with `Error::NotRegistered`.
    ///
    /// The URI is opaque to the contract â€” any scheme (`https://`, `ipfs://`,
    /// â€¦) is accepted â€” and must be at most `MAX_METADATA_URI_LEN` (512 bytes),
    /// or the call fails with `Error::MetadataUriTooLong`. Pass `None` to clear
    /// it. Unlike `update_name`, this is not locked by verification: an NGO can
    /// keep its published metadata current after approval. Publishes a
    /// `MetadataUriSetEvent`, but only when the value actually changes. See
    /// issue #347.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env, String};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let contract_id = env.register(NgoRegistry, ());
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// # let admin = Address::generate(&env);
    /// # client.init(&admin);
    /// # let owner = Address::generate(&env);
    /// # client.register(&owner, &String::from_str(&env, "Example NGO"));
    /// let uri = Some(String::from_str(&env, "https://example.org/about"));
    /// client.set_metadata_uri(&owner, &uri);
    /// assert_eq!(client.get_ngo(&owner).metadata_uri, uri);
    /// ```
    pub fn set_metadata_uri(
        env: Env,
        owner: Address,
        metadata_uri: Option<String>,
    ) -> Result<(), Error> {
        owner.require_auth();

        validate_metadata_uri(&metadata_uri)?;

        let key = DataKey::Ngo(owner.clone());
        let mut ngo: Ngo = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::NotRegistered)?;

        // Only write and emit when the URI actually changes, matching
        // `update_name`.
        if ngo.metadata_uri != metadata_uri {
            ngo.metadata_uri = metadata_uri.clone();
            env.storage().persistent().set(&key, &ngo);
            extend_instance_ttl(&env);
            extend_ngo_ttl(&env, &owner);

            MetadataUriSetEvent {
                owner,
                metadata_uri,
            }
            .publish(&env);
        }

        Ok(())
    }

    /// Reads back an NGO's metadata URI, or `None` when it has none set.
    ///
    /// Never errors or panics: an unregistered address, and a registered
    /// entry that hasn't called `set_metadata_uri`, both return `None`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env, String};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let contract_id = env.register(NgoRegistry, ());
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// # let admin = Address::generate(&env);
    /// # client.init(&admin);
    /// # let owner = Address::generate(&env);
    /// # client.register(&owner, &String::from_str(&env, "Example NGO"));
    /// assert_eq!(client.metadata_uri(&owner), None);
    ///
    /// let uri = Some(String::from_str(&env, "https://example.org"));
    /// client.set_metadata_uri(&owner, &uri);
    /// assert_eq!(client.metadata_uri(&owner), uri);
    /// ```
    pub fn metadata_uri(env: Env, owner: Address) -> Option<String> {
        env.storage()
            .persistent()
            .get::<_, Ngo>(&DataKey::Ngo(owner))
            .and_then(|ngo| ngo.metadata_uri)
    }

    /// Reads back an NGO's registry entry, registered or not.
    ///
    /// A verification whose `verified_until` has passed is reported as
    /// unverified (`verified: false`), so callers never act on a lapsed
    /// approval; the stored flag and the past timestamp are left in place and
    /// can be reactivated with `renew_ngo`. See issue #348.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env, String};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let contract_id = env.register(NgoRegistry, ());
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// # let admin = Address::generate(&env);
    /// # client.init(&admin);
    /// # let owner = Address::generate(&env);
    /// # let name = String::from_str(&env, "Example NGO");
    /// # client.register(&owner, &name);
    /// let ngo = client.get_ngo(&owner);
    /// assert_eq!(ngo.owner, owner);
    /// assert!(!ngo.verified);
    /// ```
    pub fn get_ngo(env: Env, owner: Address) -> Result<Ngo, Error> {
        let mut ngo: Ngo = env
            .storage()
            .persistent()
            .get(&DataKey::Ngo(owner))
            .ok_or(Error::NotRegistered)?;
        if ngo.verified && has_expired(&env, ngo.verified_until) {
            ngo.verified = false;
        }
        Ok(ngo)
    }

    /// Returns whether an address is a currently verified, registered NGO.
    /// Unknown and unverified addresses both return `false`, as does an
    /// address whose verification has lapsed (`verified_until` passed).
    /// Never errors or panics. See issue #348.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env, String};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryArgs, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let admin = Address::generate(&env);
    /// # let contract_id = env.register(NgoRegistry, NgoRegistryArgs::__constructor(&admin));
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// let owner = Address::generate(&env);
    /// let name = String::from_str(&env, "Example NGO");
    /// client.register(&owner, &name);
    /// assert!(!client.is_verified(&owner));
    /// client.approve_ngo(&admin, &owner);
    /// assert!(client.is_verified(&owner));
    /// ```
    pub fn is_verified(env: Env, owner: Address) -> bool {
        Self::get_ngo(env, owner)
            .map(|ngo| ngo.verified)
            .unwrap_or(false)
    }

    /// Reads back the total number of registered NGOs.
    ///
    /// Lets callers (such as the impact page) display the total count
    /// of registered NGOs without querying a backend indexer.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env, String};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let contract_id = env.register(NgoRegistry, ());
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// assert_eq!(client.ngo_count(), 0);
    ///
    /// # let admin = Address::generate(&env);
    /// # client.init(&admin);
    /// let owner = Address::generate(&env);
    /// let name = String::from_str(&env, "Example NGO");
    /// client.register(&owner, &name);
    /// assert_eq!(client.ngo_count(), 1);
    /// ```
    pub fn ngo_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::NgoCount)
            .unwrap_or(0)
    }

    /// Returns how many registered NGO entries are currently verified.
    ///
    /// Backed by a single stored counter, kept in step by `approve_ngo` /
    /// `batch_approve` (increment), `revoke_ngo` (decrement), and removal of a
    /// verified entry (decrement), so a dashboard can read the verified tally
    /// without listing and filtering every entry.
    ///
    /// The counter tracks the stored verification flag. Time never clears that
    /// flag, so an entry whose `verified_until` has lapsed still counts here
    /// until an admin revokes or renews it, even though `is_verified` already
    /// reports it as unverified. See issue #348.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env, String};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryArgs, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let admin = Address::generate(&env);
    /// # let contract_id = env.register(NgoRegistry, NgoRegistryArgs::__constructor(&admin));
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// let owner = Address::generate(&env);
    /// client.register(&owner, &String::from_str(&env, "Example NGO"));
    /// assert_eq!(client.verified_count(), 0);
    ///
    /// client.approve_ngo(&admin, &owner);
    /// assert_eq!(client.verified_count(), 1);
    /// ```
    pub fn verified_count(env: Env) -> u64 {
        get_verified_count(&env)
    }

    /// Sets the verification period, in seconds, that `approve_ngo` /
    /// `batch_approve` and `renew_ngo` add to the current ledger time to stamp
    /// an entry's `verified_until`. Admin-only.
    ///
    /// A period of `0` (the default) means approvals never expire:
    /// `verified_until` stays `None`. Changing the period only affects future
    /// approvals and renewals; entries already approved keep their existing
    /// expiry until they are renewed. Fails with `Error::NotInitialized`
    /// before `init`. Publishes a `VerifyPeriodSetEvent`. See issue #348.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryArgs, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let admin = Address::generate(&env);
    /// # let contract_id = env.register(NgoRegistry, NgoRegistryArgs::__constructor(&admin));
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// assert_eq!(client.verify_period(), 0);
    ///
    /// // Verifications last one year.
    /// client.set_verify_period(&(365 * 24 * 60 * 60));
    /// assert_eq!(client.verify_period(), 365 * 24 * 60 * 60);
    /// ```
    pub fn set_verify_period(env: Env, seconds: u64) -> Result<(), Error> {
        require_admin(&env)?;

        env.storage()
            .instance()
            .set(&DataKey::VerifyPeriod, &seconds);
        extend_instance_ttl(&env);

        VerifyPeriodSetEvent {
            verify_period: seconds,
        }
        .publish(&env);

        Ok(())
    }

    /// Reads back the configured verification period in seconds. `0` means
    /// approvals never expire. Never errors or panics. See `set_verify_period`
    /// and issue #348.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryArgs, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let admin = Address::generate(&env);
    /// # let contract_id = env.register(NgoRegistry, NgoRegistryArgs::__constructor(&admin));
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// assert_eq!(client.verify_period(), 0);
    /// ```
    pub fn verify_period(env: Env) -> u64 {
        get_verify_period(&env)
    }

    /// Marks a registered NGO as verified. Callable by the admin or a listed
    /// approver; `approver` names the acting address and must have authorized
    /// the call (see `add_approver`). Fails with
    /// `Error::AlreadyVerified` if the NGO is already verified, so a
    /// repeated call can't rewrite the entry or publish a duplicate
    /// `approved` event, and `Error::NotApprover` if `approver` is neither the
    /// admin nor a listed approver.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env, String};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let contract_id = env.register(NgoRegistry, ());
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// # let admin = Address::generate(&env);
    /// # client.init(&admin);
    /// # let owner = Address::generate(&env);
    /// # let name = String::from_str(&env, "Example NGO");
    /// # client.register(&owner, &name);
    /// client.approve_ngo(&admin, &owner);
    /// assert!(client.get_ngo(&owner).verified);
    /// ```
    pub fn approve_ngo(env: Env, approver: Address, ngo_owner: Address) -> Result<(), Error> {
        require_approver(&env, &approver)?;
        approve_registered_ngo(&env, ngo_owner)
    }

    /// Marks several registered NGOs as verified in a single call. Callable by
    /// the admin or a listed approver; `approver` names the acting address and
    /// must have authorized the call.
    ///
    /// Approves each owner in order, publishing one `approved` event per NGO
    /// exactly as `approve_ngo` would. The call is atomic: if any owner has no
    /// entry (`Error::NotRegistered`) or is already verified
    /// (`Error::AlreadyVerified`, which also covers an owner repeated within
    /// the same batch) nothing is approved and no event is published. An empty
    /// batch succeeds and emits nothing.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryArgs, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let admin = Address::generate(&env);
    /// # let contract_id = env.register(NgoRegistry, NgoRegistryArgs::__constructor(&admin));
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// let a = Address::generate(&env);
    /// let b = Address::generate(&env);
    /// client.register(&a, &String::from_str(&env, "Red Cross"));
    /// client.register(&b, &String::from_str(&env, "Oxfam"));
    ///
    /// client.batch_approve(&admin, &vec![&env, a, b]);
    /// ```
    pub fn batch_approve(env: Env, approver: Address, owners: Vec<Address>) -> Result<(), Error> {
        require_approver(&env, &approver)?;
        for ngo_owner in owners.iter() {
            approve_registered_ngo(&env, ngo_owner)?;
        }
        Ok(())
    }

    /// Reverses a prior approval, marking a registered NGO as unverified
    /// again. Callable by the admin or a listed approver; `approver` names the
    /// acting address and must have authorized the call. Returns
    /// `Error::NotRegistered` for an address with no entry, matching
    /// `approve_ngo`'s existing behavior, `Error::NotVerified` if the NGO
    /// isn't currently verified, so a repeated call can't publish a duplicate
    /// `revoked` event, and `Error::NotApprover` if `approver` is neither the
    /// admin nor a listed approver.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env, String};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let contract_id = env.register(NgoRegistry, ());
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// # let admin = Address::generate(&env);
    /// # client.init(&admin);
    /// # let owner = Address::generate(&env);
    /// # let name = String::from_str(&env, "Example NGO");
    /// # client.register(&owner, &name);
    /// # client.approve_ngo(&admin, &owner);
    /// client.revoke_ngo(&admin, &owner);
    /// assert!(!client.get_ngo(&owner).verified);
    /// ```
    pub fn revoke_ngo(env: Env, approver: Address, ngo_owner: Address) -> Result<(), Error> {
        require_approver(&env, &approver)?;

        let key = DataKey::Ngo(ngo_owner.clone());
        let mut ngo: Ngo = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::NotRegistered)?;
        if !ngo.verified {
            return Err(Error::NotVerified);
        }
        ngo.verified = false;
        // A revoked verification has no expiry to keep; clear it so the entry
        // doesn't carry a stale timestamp (#348).
        ngo.verified_until = None;
        env.storage().persistent().set(&key, &ngo);

        let verified = get_verified_count(&env);
        let new_verified = verified.checked_sub(1).ok_or(Error::ArithmeticOverflow)?;
        set_verified_count(&env, new_verified);

        extend_instance_ttl(&env);
        extend_ngo_ttl(&env, &ngo_owner);

        RevokedEvent {
            owner: ngo_owner,
            data: (),
        }
        .publish(&env);

        Ok(())
    }

    /// Extends a lapsed â€” or still-valid â€” verification by the configured
    /// period, moving its `verified_until` forward from the current ledger
    /// time. Admin-only.
    ///
    /// This is the periodic-renewal counterpart to `approve_ngo`: an entry
    /// whose verification has lapsed needs an admin to renew it, because
    /// nothing on-chain flips the stored flag back on its own. The entry must
    /// be registered (`Error::NotRegistered`) and its stored verification flag
    /// must be set (`Error::NotVerified`); the flag stays set after a lapse, so
    /// an expired entry can be renewed, while one that was never approved or
    /// was revoked must be approved first. Renewal requires a non-zero
    /// configured period, or it fails with `Error::NoVerifyPeriod`. Fails with
    /// `Error::NotInitialized` before `init`. Publishes a `RenewedEvent` with
    /// the new expiry. See issue #348.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env, String};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryArgs, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let admin = Address::generate(&env);
    /// # let contract_id = env.register(NgoRegistry, NgoRegistryArgs::__constructor(&admin));
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// client.set_verify_period(&(365 * 24 * 60 * 60));
    /// let owner = Address::generate(&env);
    /// client.register(&owner, &String::from_str(&env, "Example NGO"));
    /// client.approve_ngo(&admin, &owner);
    ///
    /// // A year later the verification has lapsed, and renewing it revives it.
    /// client.renew_ngo(&owner);
    /// assert!(client.is_verified(&owner));
    /// ```
    pub fn renew_ngo(env: Env, ngo_owner: Address) -> Result<(), Error> {
        require_admin(&env)?;

        let period = get_verify_period(&env);
        if period == 0 {
            return Err(Error::NoVerifyPeriod);
        }

        let key = DataKey::Ngo(ngo_owner.clone());
        let mut ngo: Ngo = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(Error::NotRegistered)?;
        if !ngo.verified {
            return Err(Error::NotVerified);
        }

        let verified_until = env
            .ledger()
            .timestamp()
            .checked_add(period)
            .ok_or(Error::ArithmeticOverflow)?;
        ngo.verified_until = Some(verified_until);
        env.storage().persistent().set(&key, &ngo);

        extend_instance_ttl(&env);
        extend_ngo_ttl(&env, &ngo_owner);

        RenewedEvent {
            owner: ngo_owner,
            verified_until,
        }
        .publish(&env);

        Ok(())
    }

    /// Bumps a registered NGO entry's storage TTL without changing
    /// anything about it. Callable by anyone â€” a verified NGO that
    /// `register`, `approve_ngo`, and `revoke_ngo` haven't touched in a
    /// while would otherwise have its entry archived after 90 days, with
    /// no other way to keep it alive.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, Env, String};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let contract_id = env.register(NgoRegistry, ());
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// # let admin = Address::generate(&env);
    /// # client.init(&admin);
    /// # let owner = Address::generate(&env);
    /// # let name = String::from_str(&env, "Example NGO");
    /// # client.register(&owner, &name);
    /// client.touch_ngo(&owner);
    /// ```
    pub fn touch_ngo(env: Env, owner: Address) -> Result<(), Error> {
        if !env.storage().persistent().has(&DataKey::Ngo(owner.clone())) {
            return Err(Error::NotRegistered);
        }
        extend_instance_ttl(&env);
        extend_ngo_ttl(&env, &owner);
        if let Some(index) = env
            .storage()
            .persistent()
            .get::<_, u32>(&DataKey::IndexByOwner(owner.clone()))
        {
            extend_owner_index_ttl(&env, index, &owner);
        }
        Ok(())
    }

    /// Replaces the contract's Wasm bytecode in place. Admin-only.
    /// Lets a bug fix be deployed without changing the contract address,
    /// preserving every existing NGO entry and the admin key.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};
    /// # use ngo_registry::{NgoRegistry, NgoRegistryClient};
    /// # let env = Env::default();
    /// # env.mock_all_auths();
    /// # let contract_id = env.register(NgoRegistry, ());
    /// # let client = NgoRegistryClient::new(&env, &contract_id);
    /// # let admin = Address::generate(&env);
    /// # client.init(&admin);
    /// # let new_wasm_hash = BytesN::from_array(&env, &[0u8; 32]);
    /// client.upgrade(&new_wasm_hash);
    /// ```
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), Error> {
        require_admin(&env)?;
        env.deployer()
            .update_current_contract_wasm(new_wasm_hash.clone());
        UpgradedEvent { new_wasm_hash }.publish(&env);
        Ok(())
    }
}

#[cfg(test)]
#[cfg(test)]
mod test;
