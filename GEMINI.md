Rust製テキストエディタ
完了後 cargo test --test '*' で検証する
ユーザーに動作確認をしてもらうときは cargo build でビルドを通す
テストが通ったら cargo fmt --all && cargo clippy --all-targets --fix --allow-dirty

# タスク

editorからdocumentのline削除を行っているが、それらをdocument側に移動したい