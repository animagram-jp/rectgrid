# Development

## Rule

- Follow [ORG_CONTRIBUTING.md](./ORG_CONTRIBUTING.md)

If "ORG_CONTRIBUTING.md" does not exist in the repository root of your working environment, download it by executing the following.

```bash
curl -fsSL -H "Accept: application/vnd.github.raw+json" "https://api.github.com/repos/animagram-jp/.github/contents/.github/CONTRIBUTING.md?ref=main" -o "ORG_CONTRIBUTING.md"
```

---

## Commands

```bash
cargo +nightly fmt # formatting

# unit test
cargo test --all-features

# unit test (examples)
cd examples && cargo test

# wasm build (examples; main thread, imported memory)
cd examples
RUSTFLAGS="-Clink-arg=--import-memory -Clink-arg=--max-memory=134217728" \
cargo build --release --target wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version "$(grep -A1 '^name = "wasm-bindgen"$' Cargo.lock | sed -n 's/^version = "\(.*\)"$/\1/p')" --locked
wasm-bindgen --target web --out-dir app --out-name app target/wasm32-unknown-unknown/release/app.wasm
```