# Macro Expansion Tests

This directory contains snapshot tests for the `Smithy<Type>Trait` derive macros using [macrotest](https://docs.rs/macrotest).

## How It Works

Each `.rs` file in this directory is a test case that uses the derive macros. When tests run, `cargo expand` generates the fully expanded code showing what the macros produce, and these are saved as `.expanded.rs` files.

```

## Updating Snapshots

When you modify the derive macros, regenerate the snapshots:

```bash
MACROTEST=overwrite cargo test
```