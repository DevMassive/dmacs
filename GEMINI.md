Rust製テキストエディタ
完了後 cargo test --test '*' で検証する
ユーザーに動作確認をしてもらうときは cargo build でビルドを通す
テストが通ったら cargo fmt --all && cargo clippy --all-targets --fix --allow-dirty

# タスク

いまdocumentのmodifyは`a\nbc`のようなaddやdeleteに対応していません。
しかし`\n`で分けてforで回せば、addは`a` `\n` `bc` と順番に処理が行われるため、対応できるように思います。