use std::{fs::File, path::PathBuf};

use clap::Parser;
use parser::{Format, ParserError, TransactionReaderFactory, TransactionWriterFactory};

#[derive(Parser)]
#[command(name = "convertrer")]
#[command(version = "1.0")]
struct Args {
    #[arg(long)]
    input: PathBuf,

    #[arg(long, value_enum)]
    input_format: Format,

    #[arg(long, value_enum)]
    output_format: Format,
}

fn main() -> Result<(), ParserError> {
    let args = Args::parse();

    let reader = File::open(&args.input).map_err(|e| {
        ParserError::ReaderError(parser::ReaderError::FileFormatError(format!("{e}")))
    })?;

    let mut tx_reader =
        TransactionReaderFactory::create_transaction_reader(args.input_format, reader).map_err(
            |e| ParserError::ReaderError(parser::ReaderError::FileFormatError(format!("{e}"))),
        )?;

    let txes = tx_reader.read_vector().map_err(|e| ParserError::ReaderError(e))?;

    let writer = std::io::stdout().lock();

    let mut tx_writer =
        TransactionWriterFactory::create_transaction_writer(args.output_format, writer).map_err(
            |e| ParserError::WriterError(parser::WriterError::WriterError(format!("{e}"))))?;

    tx_writer.write_vector(&txes).map_err(|e| ParserError::WriterError(e))?;

    Ok(())
}
