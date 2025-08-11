Rust製テキストエディタ
完了後 cargo test --test '*' で検証する
ユーザーに動作確認をしてもらうときは cargo build でビルドを通す
テストが通ったら cargo fmt --all && cargo clippy --all-targets --fix --allow-dirty

# タスク

いまdocumentのmodify_single_charの引数はまとめてstruct Diffにしてください。
これは将来的にDiffのstackとしてundo/redo stackを実現するためです。