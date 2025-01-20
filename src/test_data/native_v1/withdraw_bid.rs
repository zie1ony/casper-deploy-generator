use casper_types::{
    runtime_args, AsymmetricType, PublicKey, RuntimeArgs, TransactionArgs, TransactionEntryPoint,
    TransactionScheduling, TransactionTarget, U512,
};

use crate::sample::Sample;

use crate::test_data::TransactionV1Meta;

/// Represents native 'withdraw bid' sample.
#[derive(Clone, Debug)]
struct WithdrawBid {
    public_key: PublicKey,
    amount: U512,
}

impl WithdrawBid {
    pub fn new(public_key: PublicKey, amount: U512) -> Self {
        Self { public_key, amount }
    }
}

impl From<WithdrawBid> for RuntimeArgs {
    fn from(d: WithdrawBid) -> Self {
        let mut args = RuntimeArgs::new();
        args.insert("public_key", d.public_key).unwrap();
        args.insert("amount", d.amount).unwrap();
        args
    }
}

// Generate a native withdraw bid sample for every possible combination of parameters
fn native_withdraw_bid_samples(
    amounts: &[U512],
    public_keys: &[PublicKey],
) -> Vec<Sample<WithdrawBid>> {
    let mut samples: Vec<Sample<WithdrawBid>> = vec![];

    for amount in amounts {
        for public_key in public_keys {
            let label = "native_withdraw_bid_v1".to_string();
            let bid = WithdrawBid::new(public_key.clone(), *amount);
            let sample = Sample::new(label, bid, true);
            samples.push(sample);
        }
    }

    samples
}

/// Returns valid native withdraw bid samples.
pub(crate) fn valid() -> Vec<Sample<TransactionV1Meta>> {
    let amount_min = U512::from(0u8);
    let amount_mid = U512::from(100000000);
    let amount_max = U512::MAX;
    let amounts = vec![amount_min, amount_mid, amount_max];

    let public_keys = vec![
        PublicKey::secp256k1_from_bytes(
            hex::decode(b"026e1b7a8e3243f5ff14e825b0fde15103588bb61e6ae99084968b017118e0504f")
                .unwrap(),
        )
        .unwrap(),
        PublicKey::ed25519_from_bytes([1u8; 32]).unwrap(),
        PublicKey::ed25519_from_bytes([3u8; 32]).unwrap(),
    ];

    super::make_samples_with_schedulings(
        native_withdraw_bid_samples(&amounts, &public_keys),
        TransactionEntryPoint::WithdrawBid,
    )
}

/// Returns invalid native transfer samples.
pub(crate) fn invalid() -> Vec<Sample<TransactionV1Meta>> {
    let valid_pk = PublicKey::ed25519_from_bytes([9u8; 32]).unwrap();
    let valid_amount = U512::from(10_000_000u64);

    let missing_pk = runtime_args! {
        "amount" => valid_amount,
    };

    let missing_amount = runtime_args! {
        "public_key" => valid_pk.clone(),
    };

    let invalid_args = vec![
        Sample::new("missing_public_key", missing_pk, false),
        Sample::new("missing_amount", missing_amount, false),
    ];

    invalid_args
        .into_iter()
        .map(|sample_ra| {
            let (label, ra, validity) = sample_ra.destructure();
            let sample_invalid_delegate = TransactionV1Meta::new(
                TransactionArgs::Named(ra),
                TransactionTarget::Native,
                TransactionEntryPoint::WithdrawBid,
                TransactionScheduling::Standard,
            );
            let new_label = format!("native_withdraw_bid_{}", label);
            Sample::new(new_label, sample_invalid_delegate, validity)
        })
        .collect()
}
