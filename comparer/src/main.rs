use std::{
    collections::{HashMap, HashSet}, fs::File, path::PathBuf
};

use clap::Parser;
use parser::{Format, ReaderError, Transaction, TransactionReaderFactory};

#[derive(Parser)]
#[command(name = "comparer")]
#[command(version = "1.0")]
struct Args {
    #[arg(long)]
    file1: PathBuf,

    #[arg(long, value_enum)]
    file1_format: Format,

    #[arg(long)]
    file2: PathBuf,

    #[arg(long, value_enum)]
    file2_format: Format,
}

fn read_tx(file: &PathBuf, format: Format) -> Result<HashMap<u64, Transaction>, ReaderError> {
    let reader =
        File::open(file).map_err(|e| parser::ReaderError::FileFormatError(format!("{e}")))?;

    let mut tx_reader = TransactionReaderFactory::create_transaction_reader(format, reader)
        .map_err(|e| parser::ReaderError::FileFormatError(format!("{e}")))?;

    let mut txes = HashMap::new();

    for tx in tx_reader.read_vector()? {
        txes.insert(tx.tx_id, tx);
    }

    Ok(txes)
}

fn main() -> Result<(), ReaderError> {
    let args = Args::parse();

    let txes_1 = read_tx(&args.file1, args.file1_format)?;
    let txes_2 = read_tx(&args.file2, args.file2_format)?;

    let ids_1: HashSet<u64> = txes_1.keys().copied().collect();
    let ids_2: HashSet<u64> = txes_2.keys().copied().collect();

    let only_in_1: Vec<_> = ids_1.difference(&ids_2).copied().collect();

    let only_in_2: Vec<_> = ids_2.difference(&ids_1).copied().collect();

    let difference: Vec<_> = ids_1
        .intersection(&ids_2)
        .copied()
        .filter(|tx_id| txes_1[tx_id] != txes_2[tx_id])
        .collect();

    if only_in_1.is_empty() && only_in_2.is_empty() && difference.is_empty() {
        println!(
            "The transaction records in '{}' and '{}' are identical.",
            args.file1.display(),
            args.file2.display()
        );
    } else {
        let mut first = true;

        if !only_in_1.is_empty() {
            println!("Only in '{}':", args.file1.display());
            for tx_id in &only_in_1 {
                println!("{}", &txes_1[tx_id]);
            }
        }

        if !only_in_2.is_empty() {
            if first {
                first = false;
            } else {
                println!();
            }
            println!("Only in '{}':", args.file2.display());
            for tx_id in &only_in_2 {
                println!("{}", &txes_2[tx_id]);
            }
        }
        if !difference.is_empty() {
            if !first {
                println!();
            }
            first = true;
            println!("Different transactions:");
            for tx_id in &difference {
                if first {
                    first = false;
                } else {
                    println!();
                }
                println!("In {}: {}", args.file1.display(), &txes_1[tx_id]);
                println!("In {}: {}", args.file2.display(), &txes_2[tx_id]);
            }
        }
    }

    Ok(())
}
