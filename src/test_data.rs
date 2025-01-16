use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
};

use casper_types::{
    bytesrepr::{Bytes, ToBytes}, Deploy, DeployHash, DeployHeader, Digest, ExecutableDeployItem, InitiatorAddr, PricingMode, PublicKey, SecretKey, TimeDiff, Timestamp, Transaction, TransactionArgs, TransactionInvocationTarget, TransactionRuntimeParams, TransactionTarget, TransactionV1, TransactionV1Payload
};
use rand::{prelude::*, Rng};

use auction::{delegate, undelegate};

use crate::{
    parser::v1::{
        TransactionV1Meta, ARGS_MAP_KEY, ENTRY_POINT_MAP_KEY, SCHEDULING_MAP_KEY, TARGET_MAP_KEY,
    },
    sample::Sample,
};

use self::auction::redelegate;

mod auction;
mod commons;
mod generic;
pub(crate) mod native_transfer;
mod native_v1;
pub(crate) mod sign_message;
mod system_payment;

// TODO: Investigate these values
// From the chainspec.
// 1 minute.
const MIN_TTL: TimeDiff = TimeDiff::from_seconds(60);
// 1 day.
const MAX_TTL: TimeDiff = TimeDiff::from_seconds(60 * 60 * 24);
// 1 hour.
const TTL_HOUR: TimeDiff = TimeDiff::from_seconds(60 * 60);

// From the chainspec.
const MIN_DEPS_COUNT: u8 = 0;
const MAX_DEPS_COUNT: u8 = 10;

// From the chainspec.
const MIN_APPROVALS_COUNT: u8 = 1;
const MAX_APPROVALS_COUNT: u8 = 10;

/// Returns a sample `Deploy`, given the input data.
fn make_deploy_sample(
    session: Sample<ExecutableDeployItem>,
    payment: Sample<ExecutableDeployItem>,
    ttl: TimeDiff,
    dependencies: Vec<DeployHash>,
    signing_keys: &[SecretKey],
) -> Sample<Transaction> {
    let (main_key, secondary_keys) = signing_keys.split_at(1);
    let (payment_label, payment, payment_validity) = payment.destructure();
    let (session_label, session, session_validity) = session.destructure();

    let header = DeployHeader::new(
        PublicKey::from(&main_key[0]),
        Timestamp::from_str("2021-05-04T14:20:35.104Z").unwrap(),
        ttl,
        2,
        Digest::hash([1u8; 32]),
        dependencies,
        "mainnet".into(),
    );

    let hash = DeployHash::new(Digest::hash([1u8; 32]));
    let deploy = Deploy::new(hash, header, payment, session);
    let transaction = Transaction::from_deploy(deploy);

    let mut sample = Sample::new(
        session_label,
        transaction,
        session_validity && payment_validity,
    );
    sample.add_label(payment_label);

    // Sign deploy with possibly multiple keys.
    for key in secondary_keys {
        let (label, mut deploy, validity) = sample.destructure();
        deploy.sign(key);
        sample = Sample::new(label, deploy, validity);
    }
    sample
}

fn make_v1_sample(
    meta: Sample<TransactionV1Meta>,
    pricing_mode: PricingMode,
    ttl: TimeDiff,
    signing_keys: &[SecretKey],
) -> Sample<Transaction> {
    let (main_key, _) = signing_keys.split_at(1);
    let (label, meta, is_valid) = meta.destructure();
    let initiator_addr = InitiatorAddr::PublicKey(PublicKey::from(&main_key[0]));

    let fields = {
        let mut fields: BTreeMap<u16, Bytes> = BTreeMap::new();
        let args = TransactionArgs::Named(meta.args.into_named().unwrap());
        fields.insert(ARGS_MAP_KEY, args.to_bytes().unwrap().into());
        fields.insert(
            ENTRY_POINT_MAP_KEY,
            meta.entry_point.to_bytes().unwrap().into(),
        );
        fields.insert(TARGET_MAP_KEY, meta.target.to_bytes().unwrap().into());
        fields.insert(
            SCHEDULING_MAP_KEY,
            meta.scheduling.to_bytes().unwrap().into(),
        );
        fields
    };

    let payload = TransactionV1Payload::new(
        "mainnet".into(),
        Timestamp::from_str("2021-05-04T14:20:35.104Z").unwrap(),
        ttl,
        pricing_mode,
        initiator_addr,
        fields,
    );

    let mut transaction_v1 =
        TransactionV1::new(Digest::hash([1u8; 32]).into(), payload, BTreeSet::new());

    transaction_v1.sign(&main_key[0]);

    let transaction = Transaction::V1(transaction_v1);

    Sample::new(label, transaction, is_valid)
}

fn make_dependencies(count: u8) -> Vec<DeployHash> {
    if count == 0 {
        return vec![];
    }

    let mut dependencies = vec![];
    for i in 0..count {
        dependencies.push(DeployHash::new([i; 32].into()));
    }
    dependencies
}

fn random_keys(key_count: u8) -> Vec<SecretKey> {
    let mut out = vec![];
    for i in 0..key_count {
        let key = if i % 2 == 0 {
            SecretKey::ed25519_from_bytes([i; 32]).expect("successful key construction")
        } else {
            SecretKey::secp256k1_from_bytes([i; 32]).expect("successful key construction")
        };
        out.push(key);
    }
    out
}

// Given input collections for session samples and payment samples,
// returns a combination of all - every session samples is matched with every payment sample,
// creating n^2 deploy samples.
fn construct_deploy_samples<R: Rng>(
    rng: &mut R,
    session_samples: Vec<Sample<ExecutableDeployItem>>,
    payment_samples: Vec<Sample<ExecutableDeployItem>>,
) -> Vec<Sample<Transaction>> {
    let mut samples = vec![];

    // These params do not change validity of a sample.
    let mut ttls = [MIN_TTL, TTL_HOUR, MAX_TTL];
    let mut deps_count = [MIN_DEPS_COUNT, 3, MAX_DEPS_COUNT];
    let mut key_count = [MIN_APPROVALS_COUNT, 3, MAX_APPROVALS_COUNT];

    for session in session_samples {
        for payment in &payment_samples {
            // Random number of keys.
            key_count.shuffle(rng);
            // Random signing keys count.
            let mut keys: Vec<SecretKey> = random_keys(*key_count.first().unwrap());
            // Randomize order of keys, so that both alg have chance to be the main one.
            keys.shuffle(rng);

            // Random dependencies within correct limits.
            deps_count.shuffle(rng);
            let dependencies = make_dependencies(deps_count.first().cloned().unwrap());

            // Pick a random TTL value.
            ttls.shuffle(rng);
            let ttl = ttls.first().cloned().unwrap();

            let sample_deploy =
                make_deploy_sample(session.clone(), payment.clone(), ttl, dependencies, &keys);
            samples.push(sample_deploy);
        }
    }
    samples
}

fn construct_transaction_samples<R: Rng>(
    rng: &mut R,
    transaction_metas: Vec<Sample<TransactionV1Meta>>,
) -> Vec<Sample<Transaction>> {
    let mut samples = vec![];

    // These params do not change validity of a sample.
    let mut ttls = [MIN_TTL, TTL_HOUR, MAX_TTL];
    let mut key_count = [MIN_APPROVALS_COUNT, 3, MAX_APPROVALS_COUNT];

    for meta in transaction_metas {
        // Random number of keys.
        key_count.shuffle(rng);

        // Random signing keys count.
        let mut keys: Vec<SecretKey> = random_keys(*key_count.first().unwrap());

        // Randomize order of keys, so that both alg have chance to be the main one.
        keys.shuffle(rng);

        // Pick a random TTL value.
        ttls.shuffle(rng);
        let ttl = ttls.first().cloned().unwrap();

        // TODO: Verify payment params later
        let sample_transaction_payment_limited = make_v1_sample(
            meta.clone(),
            PricingMode::PaymentLimited {
                payment_amount: 10_000,
                gas_price_tolerance: 100,
                standard_payment: false,
            },
            ttl,
            &keys,
        );

        let sample_transaction_fixed = make_v1_sample(
            meta.clone(),
            PricingMode::Fixed {
                additional_computation_factor: 1,
                gas_price_tolerance: 100,
            },
            ttl,
            &keys,
        );

        let sample_transaction_prepaid = make_v1_sample(
            meta.clone(),
            PricingMode::Prepaid {
                receipt: Digest::from_raw([1u8; 32]),
            },
            ttl,
            &keys,
        );

        samples.push(sample_transaction_payment_limited);
        samples.push(sample_transaction_fixed);
        samples.push(sample_transaction_prepaid);
    }

    samples
}

pub(crate) fn deploy_redelegate_samples<R: Rng>(rng: &mut R) -> Vec<Sample<Transaction>> {
    let valid_samples = redelegate::valid();
    let valid_payment_samples = vec![system_payment::valid()];

    let mut samples = construct_deploy_samples(rng, valid_samples, valid_payment_samples);
    let invalid_samples = redelegate::invalid();
    let invalid_payment_samples = vec![system_payment::invalid(), system_payment::valid()];
    samples.extend(construct_deploy_samples(
        rng,
        invalid_samples,
        invalid_payment_samples,
    ));
    samples
}

pub(crate) fn deploy_generic_samples<R: Rng>(rng: &mut R) -> Vec<Sample<Transaction>> {
    let valid_samples = generic::valid(rng);
    let valid_payment_samples = vec![system_payment::valid()];

    let mut samples = construct_deploy_samples(rng, valid_samples.clone(), valid_payment_samples);

    // Generic transactions are invalid only if their payment contract is invalid.
    // Otherwise there are no rules that could be violated and make txn invalid -
    // if it has correct structure it's valid b/c we don't know what the contracts expect.
    samples.extend(construct_deploy_samples(
        rng,
        valid_samples,
        vec![system_payment::invalid()],
    ));
    samples
}

pub(crate) fn deploy_native_transfer_samples<R: Rng>(rng: &mut R) -> Vec<Sample<Transaction>> {
    let mut native_transfer_samples =
        construct_deploy_samples(rng, native_transfer::valid(), vec![system_payment::valid()]);

    native_transfer_samples.extend(construct_deploy_samples(
        rng,
        native_transfer::invalid(),
        vec![system_payment::invalid(), system_payment::valid()],
    ));
    native_transfer_samples
}

pub(crate) fn deploy_delegate_samples<R: Rng>(rng: &mut R) -> Vec<Sample<Transaction>> {
    let mut delegate_samples =
        construct_deploy_samples(rng, delegate::valid(), vec![system_payment::valid()]);

    delegate_samples.extend(construct_deploy_samples(
        rng,
        delegate::invalid(),
        vec![system_payment::invalid(), system_payment::valid()],
    ));

    delegate_samples
}

pub(crate) fn deploy_undelegate_samples<R: Rng>(rng: &mut R) -> Vec<Sample<Transaction>> {
    let mut undelegate_samples =
        construct_deploy_samples(rng, undelegate::valid(), vec![system_payment::valid()]);

    undelegate_samples.extend(construct_deploy_samples(
        rng,
        undelegate::invalid(),
        vec![system_payment::invalid(), system_payment::valid()],
    ));

    undelegate_samples
}

pub(crate) fn v1_native_transfer_samples<R: Rng>(rng: &mut R) -> Vec<Sample<Transaction>> {
    native_samples(
        rng,
        native_v1::transfer::valid,
        native_v1::transfer::invalid
    )
}

pub(crate) fn native_delegate_samples<R: Rng>(rng: &mut R) -> Vec<Sample<Transaction>> {
    native_samples(
        rng,
        native_v1::delegate::valid,
        native_v1::delegate::invalid
    )
}

pub(crate) fn native_undelegate_samples<R: Rng>(rng: &mut R) -> Vec<Sample<Transaction>> {
    native_samples(
        rng,
        native_v1::undelegate::valid,
        native_v1::undelegate::invalid
    )
}

pub(crate) fn native_redelegate_samples<R: Rng>(rng: &mut R) -> Vec<Sample<Transaction>> {
    native_samples(
        rng,
        native_v1::redelegate::valid,
        native_v1::redelegate::invalid
    )
}

pub(crate) fn native_add_bid_samples<R: Rng>(rng: &mut R) -> Vec<Sample<Transaction>> {
    native_samples(
        rng,
        native_v1::add_bid::valid,
        native_v1::add_bid::invalid
    )
}

pub(crate) fn native_activate_bid_samples<R: Rng>(rng: &mut R) -> Vec<Sample<Transaction>> {
    native_samples(
        rng,
        native_v1::activate_bid::valid,
        native_v1::activate_bid::invalid
    )
}

pub(crate) fn native_samples<R: Rng>(
    rng: &mut R,
    valid_generator: fn() -> Vec<Sample<TransactionV1Meta>>,
    invalid_generator: fn() -> Vec<Sample<TransactionV1Meta>>
) -> Vec<Sample<Transaction>> {
    // populate with valid native samples
    let mut samples = construct_transaction_samples(
        rng,
        valid_generator()
    );

    // extend with invalid samples (from generator)
    samples.extend(construct_transaction_samples(
        rng,
        invalid_generator()
    ));

    // extend with invalid samples (force target/entrypoint mismatch)
    samples.extend(construct_transaction_samples(
        rng,
        valid_generator().into_iter().map(|sample| {
            let (label, mut meta, _) = sample.destructure();
            
            meta.target = TransactionTarget::Stored {
                id: TransactionInvocationTarget::ByName("auction".into()),
                runtime: TransactionRuntimeParams::VmCasperV1
            };

            let new_label = format!("{label}_native_ep_with_stored_contract");
            Sample::new(new_label, meta, false)
        }).collect()
    ));

    samples
}