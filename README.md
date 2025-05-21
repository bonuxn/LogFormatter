# LogFormatter

## how use


## settings json

```json
{
  "filePath" : "filePath",
  "targetWords" : {
    "words"  : ["word1","word2"],
    "regEx" : false
  },
  "dateFormat": "(\\d{4})-(\\d{2})-(\\d{2})",
  "timeFormat": "(\\d{2}):(\\d{2}):(\\d{2}\\.(\\d{3})"
}
```

- filePath
  - 対象ファイルのファイルパスを設定
- words
  - 検索にかけたい単語を設定(複数選択可能)
- regEx
  - 正規化表現を使用するかを設定(true:正規化表現を使用する)

## Output

targetにhitした単語を以下の列で、csv形式で出力する

- date
- time
- word

## build

```cmd
  # build
  cargo build --release
```