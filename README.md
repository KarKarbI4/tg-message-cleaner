# tg-message-cleaner (Rust)

Script for cleaning Telegram messages by keyword, rewritten in Rust.

## Requirements

- Rust (via `rustup`) -> see install below
- Telegram API credentials: `TG_API_ID`, `TG_API_HASH`

## Install Rust

```bash
# macOS / Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# verify
rustc --version
cargo --version
```

## Setup

```bash
# clone repo
git clone git@github.com:KarKarbI4/tg-message-cleaner.git
cd ./tg-message-cleaner

# create .env with your Telegram credentials
cat > .env <<'EOF'
TG_API_ID=123456
TG_API_HASH=your_api_hash_here
EOF

# build
cargo build --release
```

## Usage

Search and delete messages by keyword across your dialogs. The tool performs two passes:

- Deletes your own messages for everyone (revoke = true)
- Deletes others' messages for you in user dialogs (revoke = false)

It also writes found messages to `./tmp/messages.json` before deletion.

```bash
# run
./target/release/tg-message-cleaner <keyword>

# example
./target/release/tg-message-cleaner spam
```

You will be prompted for phone number, login code, and password (if two-factor is enabled). Sessions are stored under `./sessionStorage/` and reused.

## Notes

- The tool uses the `grammers` Telegram client under the hood.
- Ensure your API ID/Hash belong to a Telegram application created at `https://my.telegram.org`.

## Development

```bash
# run in debug mode
cargo run -- <keyword>

# format
cargo fmt

# lint (if you have clippy)
cargo clippy -- -D warnings
```
