Rust製テキストエディタ
完了後 cargo test --test '*' で検証する
ユーザーに動作確認をしてもらうときは cargo build でビルドを通す
テストが通ったら cargo fmt --all && cargo clippy --all-targets --fix --allow-dirty

# タスク

いま testはfeatures testで実行することでmock_instant::thread_local::Instantを使うようになり、undoのテストが動作している。
モックはやめたい。
- editorにundoをグループ化する時間を設定する関数を用意して、それをtestではごく短い時間にする
- testでは連続入力は待ち時間無しで行い、時間の経過は、設定したごく短い時間だけ待つ
