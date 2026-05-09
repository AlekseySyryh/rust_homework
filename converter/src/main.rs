use std::{fs::File, io::Read, io::Write, path::PathBuf};

use clap::Parser;
use parser::{Format, ParserError, TransactionReaderFactory, TransactionWriterFactory};

#[derive(Parser)]
#[command(name = "convertrer")]
#[command(version = "1.0")]
struct Args {
    /// Input file. Default is stdin
    #[arg(long, value_name = "file")]
    input: Option<PathBuf>,
    /// Output file. Default is stdout
    #[arg(long, value_name = "file")]
    output: Option<PathBuf>,
    /// Input format
    #[arg(long, value_enum)]
    input_format: Format,
    /// Output format
    #[arg(long, value_enum)]
    output_format: Format,
}

fn main() -> Result<(), ParserError> {
    let args = Args::parse();

    let reader: Box<dyn Read> = match args.input {
        Some(value) => Box::new(File::open(value).map_err(|e| {
            ParserError::ReaderError(parser::ReaderError::FileFormatError(format!("{e}")))
        })?),
        None => Box::new(std::io::stdin().lock()),
    };

    let mut tx_reader =
        TransactionReaderFactory::create_transaction_reader(args.input_format, reader).map_err(
            |e| ParserError::ReaderError(parser::ReaderError::FileFormatError(format!("{e}"))),
        )?;

    let txes = tx_reader
        .read_vector()
        .map_err(|e| ParserError::ReaderError(e))?;

    let writer: Box<dyn Write> = match args.output {
        Some(value) => Box::new(File::create(value).map_err(|e| {
            ParserError::WriterError(parser::WriterError::WriterError(format!("{e}")))
        })?),
        None => Box::new(std::io::stdout().lock()),
    };

    let mut tx_writer =
        TransactionWriterFactory::create_transaction_writer(args.output_format, writer).map_err(
            |e| ParserError::WriterError(parser::WriterError::WriterError(format!("{e}"))),
        )?;

    tx_writer
        .write_vector(&txes)
        .map_err(|e| ParserError::WriterError(e))?;

    Ok(())
}
