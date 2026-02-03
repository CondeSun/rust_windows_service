# rust_windows_service

# Build on Mac for Windows MVSC
```shell
brew install llvm
cargo install --locked cargo-xwin
cargo xwin build --release --target x86_64-pc-windows-msvc
```
