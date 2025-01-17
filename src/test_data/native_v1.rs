use casper_types::{EraId, RuntimeArgs, Timestamp, TransactionArgs, TransactionEntryPoint, TransactionScheduling, TransactionTarget};

use super::{Sample, TransactionV1Meta};

pub mod delegate;
pub mod redelegate;
pub mod transfer;
pub mod undelegate;
pub mod add_bid;
pub mod activate_bid;
pub mod change_bid_pk;
pub mod add_reservations;
pub mod cancel_reservations;

pub(crate) fn make_samples_with_schedulings<T: Into<RuntimeArgs> + Clone>(
    from_samples: Vec<Sample<T>>,
    entry_point: TransactionEntryPoint
) -> Vec<Sample<TransactionV1Meta>> {
    let mut samples: Vec<Sample<TransactionV1Meta>> = vec![];
    let schedulings = make_sample_schedulings();
    for sample in from_samples {
        let (prefix, sample, validity) = sample.destructure();
        for (scheduling, label) in schedulings {
            let new_label = format!("{prefix}_{label}");
            let transaction_sample = Sample::new(
                new_label,
                TransactionV1Meta::new(
                    TransactionArgs::Named(sample.clone().into()),
                    TransactionTarget::Native,
                    entry_point.clone(),
                    scheduling.to_owned(),
                ),
                validity
            );
            samples.push(transaction_sample);
        }
    }
    samples
}

fn make_sample_schedulings() -> [(TransactionScheduling, &'static str); 3] {
    [
        (TransactionScheduling::Standard, "standard_scheduling"),
        (TransactionScheduling::FutureEra(EraId::new(6000)), "future_era"),
        (TransactionScheduling::FutureTimestamp(Timestamp::from(6000)), "future_timestamp"),
    ]
}