# Commands

Watch src/ and templates/ for changes, and run 'cargo run' in response:

```rust
cargo watch -c -w src -w templates -x run
```

Watch templates/ for changes, and recompile tailwindcss in respons:

```rust
cargo watch -c -w templates -- twcli2
```
