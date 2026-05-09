# CLI Converter

Крейт содержит утилиту для преобразования файла транзакций из одного формата в другой.

Формат вызова:

```
Usage: converter [OPTIONS] --input-format <INPUT_FORMAT> --output-format <OUTPUT_FORMAT>

Options:
      --input <file>
          Input file. Default is stdin

      --output <file>
          Output file. Default is stdout

      --input-format <INPUT_FORMAT>
          Input format

          Possible values:
          - bin: Binary
          - csv: CSV
          - txt: Text

      --output-format <OUTPUT_FORMAT>
          Output format

          Possible values:
          - bin: Binary
          - csv: CSV
          - txt: Text
```

Где:

* input - Необязательный праметер. Имя входного файла. Если не указан - берется из stdin.
* input-format - Обязательный параметр. Формат входного файла.
* output - Необязательный праметер. Имя выходного файла. Если не указан - берется из stdout.
* output-format - Обязательный параметр. Формат выходного файла.

