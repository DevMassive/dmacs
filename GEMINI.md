Rust製テキストエディタ
完了後cargo testで検証する
テストが通ったら cargo fmt --all && cargo clippy --all-targets --fix --allow-dirty
git addして、git diff --staged で最終確認
