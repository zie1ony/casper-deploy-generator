use casper_types::{RuntimeArgs, TransactionArgs, TransactionEntryPoint, TransactionScheduling, TransactionTarget};

use super::{Sample, TransactionV1Meta};

pub mod delegate;
pub mod redelegate;
pub mod transfer;
pub mod undelegate;
pub mod add_bid;
pub mod activate_bid;

pub(crate) fn make_samples_with_schedulings<T: Into<RuntimeArgs> + Clone>(
    from_samples: Vec<Sample<T>>,
    entry_point: TransactionEntryPoint,
    schedulings: &[(TransactionScheduling, &str)]
) -> Vec<Sample<TransactionV1Meta>> {
    let mut samples: Vec<Sample<TransactionV1Meta>> = vec![];
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