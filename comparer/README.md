# CLI Comparer

Крейт содержит утилиту для сравнения двух файлов с транзакциями.

Формат вызова:

```
comparer --file1 <file> --file1-format <FILE1_FORMAT> --file2 <file> --file2-format <FILE2_FORMAT>

Options:
      --file1 <file>
          First file to compare

      --file1-format <FILE1_FORMAT>
          Format of the first file

          Possible values:
          - bin: Binary
          - csv: CSV
          - txt: Text

      --file2 <file>
          Second file to compare

      --file2-format <FILE2_FORMAT>
          Format of the second file

          Possible values:
          - bin: Binary
          - csv: CSV
          - txt: Text
```

Где:

* file1 - первый сравниваемый файл.
* file1-format - формат первого файла.
* file2 - второй сравниваемый файл.
* file2-format - формат второго файла.

Транзакции сравниваются сначала по tx_id, а затем по содержанию.

Если файлы одинаковые - то утилита собщает:

```
The transaction records in 'records_example.bin' and '123.txt' are identical.
```

Если файлы разные утилита сообщает о транзакциях которы есть только в первом файле, но отсутствуют во втором (если такие есть), затем о транзакциях которые есть только во втором (если такие есть), а затем о транзациях которые есть в обоих файлах, но при этом они отличаются (если такие есть).

```
Only in 'records_example1.csv':
tx_id: 1000000000000999, tx_type: DEPOSIT, from_user_id: 0, to_user_id: 3314635390654657431, amount: 100000, timestamp: 1633096800000, status: FAILURE, description: "Record number 1000"
tx_id: 1000000000000139, tx_type: TRANSFER, from_user_id: 9223372036854775807, to_user_id: 5872870581127660458, amount: 14000, timestamp: 1633045200000, status: PENDING, description: "Record number 140"

Only in 'records_example2.csv':
tx_id: 1000000000000000, tx_type: DEPOSIT, from_user_id: 0, to_user_id: 9223372036854775807, amount: 100, timestamp: 1633036860000, status: FAILURE, description: "Record number 1"
tx_id: 1000000000000120, tx_type: DEPOSIT, from_user_id: 0, to_user_id: 4628780074356221680, amount: 12100, timestamp: 1633044060000, status: FAILURE, description: "Record number 121"

Different transactions:
In 'records_example1.csv': tx_id: 1000000000000039, tx_type: DEPOSIT, from_user_id: 0, to_user_id: 9223372036854775807, amount: 4000, timestamp: 1633039200000, status: FAILURE, description: "Record number 40"
In 'records_example2.csv': tx_id: 1000000000000039, tx_type: DEPOSIT, from_user_id: 0, to_user_id: 9223372036854775803, amount: 4000, timestamp: 1633039200000, status: FAILURE, description: "Record number 40"

In 'records_example1.csv': tx_id: 1000000000000001, tx_type: TRANSFER, from_user_id: 9223372036854775807, to_user_id: 9223372036854775807, amount: 200, timestamp: 1633036920000, status: PENDING, description: "Record number 2"
In 'records_example2.csv': tx_id: 1000000000000001, tx_type: TRANSFER, from_user_id: 9223372036854775807, to_user_id: 9223372036854775807, amount: 201, timestamp: 1633036920000, status: PENDING, description: "Record number 2"
```