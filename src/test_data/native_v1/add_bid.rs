use casper_types::{
    runtime_args, AsymmetricType, PublicKey, RuntimeArgs, TransactionArgs, TransactionEntryPoint,
    TransactionScheduling, TransactionTarget, U512,
};

use crate::sample::Sample;

use crate::test_data::TransactionV1Meta;

/// Represents native 'add bid' sample.
#[derive(Clone, Debug)]
struct AddBid {
    public_key: PublicKey,
    delegation_rate: u8,
    amount: U512,
    minimum_delegation_amount: Option<u64>,
    maximum_delegation_amount: Option<u64>,
    reserved_slots: Option<u32>,
}

impl AddBid {
    pub fn new(
        public_key: PublicKey,
        delegation_rate: u8,
        amount: U512,
        minimum_delegation_amount: Option<u64>,
        maximum_delegation_amount: Option<u64>,
        reserved_slots: Option<u32>,
    ) -> Self {
        Self {
            public_key,
            delegation_rate,
            amount,
            minimum_delegation_amount,
            maximum_delegation_amount,
            reserved_slots,
        }
    }
}

impl From<AddBid> for RuntimeArgs {
    fn from(d: AddBid) -> Self {
        let mut args = RuntimeArgs::new();
        args.insert("public_key", d.public_key).unwrap();
        args.insert("delegation_rate", d.delegation_rate).unwrap();
        args.insert("amount", d.amount).unwrap();
        args.insert("minimum_delegation_amount", d.minimum_delegation_amount)
            .unwrap();
        args.insert("maximum_delegation_amount", d.maximum_delegation_amount)
            .unwrap();
        args.insert("reserved_slots", d.reserved_slots).unwrap();
        args
    }
}

// Generate a native add bid sample for every possible combination of parameters
fn native_add_bid_samples(
    amounts: &[U512],
    public_keys: &[PublicKey],
    delegation_rates: &[u8],
    minimum_delegation_amounts: &[Option<u64>],
    maximum_delegation_amounts: &[Option<u64>],
    reserved_slots: &[Option<u32>],
) -> Vec<Sample<AddBid>> {
    let mut samples: Vec<Sample<AddBid>> = vec![];

    for amount in amounts {
        for public_key in public_keys {
            for delegation_rate in delegation_rates {
                for minimum_delegation_amount in minimum_delegation_amounts {
                    for maximum_delegation_amount in maximum_delegation_amounts {
                        for reserved_slot in reserved_slots {
                            let label = "native_add_bid_v1".to_string();
                            let bid = AddBid::new(
                                public_key.clone(),
                                *delegation_rate,
                                *amount,
                                *minimum_delegation_amount,
                                *maximum_delegation_amount,
                                *reserved_slot,
                            );
                            let sample = Sample::new(label, bid, true);
                            samples.push(sample);
                        }
                    }
                }
            }
        }
    }

    samples
}

/// Returns valid native add bid samples.
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

    let delegation_rates = vec![1, 2, 4];

    let minimum_delegation_amounts = [None, Some(0u64), Some(10u64)];
    let maximum_delegation_amounts = [None, Some(20u64), Some(30u64)];
    let reserved_slots = [None, Some(0u32), Some(3u32)];

    super::make_samples_with_schedulings(
        native_add_bid_samples(
            &amounts,
            &public_keys,
            &delegation_rates,
            &minimum_delegation_amounts,
            &maximum_delegation_amounts,
            &reserved_slots,
        ),
        TransactionEntryPoint::AddBid,
    )
}

/// Returns invalid native transfer samples.
pub(crate) fn invalid() -> Vec<Sample<TransactionV1Meta>> {
    let valid_pk = PublicKey::ed25519_from_bytes([9u8; 32]).unwrap();
    let valid_amount = U512::from(10_000_000u64);
    let valid_delegation_rate = 1u8;
    let valid_min = Option::<u64>::None;
    let valid_max = Option::<u64>::None;
    let valid_slots = Option::<u32>::None;

    let missing_pk = runtime_args! {
        "amount" => valid_amount,
        "delegation_rate" => valid_delegation_rate,
        "minimum_delegation_amount" => valid_min,
        "maximum_delegation_amount" => valid_max,
        "reserved_slots" => valid_slots,
    };

    let missing_amount = runtime_args! {
        "public_key" => valid_pk.clone(),
        "delegation_rate" => valid_delegation_rate,
        "minimum_delegation_amount" => valid_min,
        "maximum_delegation_amount" => valid_max,
        "reserved_slots" => valid_slots,
    };

    let missing_delegation_rate = runtime_args! {
        "public_key" => valid_pk.clone(),
        "amount" => valid_amount,
        "minimum_delegation_amount" => valid_min,
        "maximum_delegation_amount" => valid_max,
        "reserved_slots" => valid_slots,
    };

    let missing_minimum_delegation_amount = runtime_args! {
        "public_key" => valid_pk.clone(),
        "amount" => valid_amount,
        "delegation_rate" => valid_delegation_rate,
        "maximum_delegation_amount" => valid_max,
        "reserved_slots" => valid_slots,
    };

    let missing_maximum_delegation_amount = runtime_args! {
        "public_key" => valid_pk.clone(),
        "amount" => valid_amount,
        "delegation_rate" => valid_delegation_rate,
        "minimum_delegation_amount" => valid_min,
        "reserved_slots" => valid_slots,
    };

    let missing_reserved_slots = runtime_args! {
        "public_key" => valid_pk.clone(),
        "amount" => valid_amount,
        "delegation_rate" => valid_delegation_rate,
        "minimum_delegation_amount" => valid_min,
        "maximum_delegation_amount" => valid_max,
    };

    let invalid_args = vec![
        Sample::new("missing_public_key", missing_pk, false),
        Sample::new("missing_amount", missing_amount, false),
        Sample::new("missing_delegation_rate", missing_delegation_rate, false),
        Sample::new(
            "missing_minimum_delegation_amount",
            missing_minimum_delegation_amount,
            false,
        ),
        Sample::new(
            "missing_maximum_delegation_amount",
            missing_maximum_delegation_amount,
            false,
        ),
        Sample::new("missing_reserved_slots", missing_reserved_slots, true),
    ];

    invalid_args
        .into_iter()
        .map(|sample_ra| {
            let (label, ra, validity) = sample_ra.destructure();
            let sample_invalid_delegate = TransactionV1Meta::new(
                TransactionArgs::Named(ra),
                TransactionTarget::Native,
                TransactionEntryPoint::AddBid,
                TransactionScheduling::Standard,
            );
            let new_label = format!("native_add_bid_{}", label);
            Sample::new(new_label, sample_invalid_delegate, validity)
        })
        .collect()
}
