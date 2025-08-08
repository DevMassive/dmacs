Rust製テキストエディタ
完了後 cargo test --test '*' で検証する
ユーザーに動作確認をしてもらうときは cargo build でビルドを通す
テストが通ったら cargo fmt --all && cargo clippy --all-targets --fix --allow-dirty

# タスク

- `Alt + _` でredoをつくってほしい。
- `cat -v` では `Alt + _` = `^[_`

# 制約事項

- 既存のプログラムの理解、テストの理解を行い、実装方針を立てる。ただし実装に移る前にテストをつくること