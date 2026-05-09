# Библиотека paresr

Крейт реализует библиотеку для чтения или записи набора транзакций в/из файла определенного формата.

Поддерживаемые форматы:

* YPBankCsv — таблица банковских операций.
* YPBankText — текстовый формат описания списка операций.
* YPBankBin — бинарное предоставление списка операций.

Для каждого из форматов есть релизация трейтов TransactionReader и TransactionWriter

```
pub trait TransactionReader {
    // Required method
    fn read_tx(&mut self) -> Result<Option<Transaction>, ReaderError>;
    
    // Provided method
    fn read_vector(&mut self) -> Result<Vec<Transaction>, ReaderError> { ... }
}

pub trait TransactionWriter {
    // Required method
    fn write_tx(&mut self, tx: &Transaction) -> Result<(), WriterError>;

    // Provided method
    fn write_vector(&mut self, txs: &[Transaction]) -> Result<(), WriterError> { ... }
}
```

Для выбора формата из реализованных имеются фабрики TransactionReaderFactory и TransactionWriterFactory

```
use std::io::{Read, Write};
use parser::{TransactionReaderFactory, TransactionReader, TransactionWriterFactory, TransactionWriter, Format};

let reader: Read = ...;
let tx_reader = TransactionReaderFactory::create_transaction_reader(Format::CSV, reader);

let writer: Write = ...;
let tx_writer = TransactionWriterFactory::create_transaction_writer(Format::BIN, cursor);

```