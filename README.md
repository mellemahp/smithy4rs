<h1 align="center"><strong>smithy4rs</strong>: Smithy code generators for Rust (unofficial)</h1>
<p align="center">
   <a href="https://github.com/mellemahp/smithy4rs/actions/workflows/rust-checks.yml" title="Build Status">
      <img alt="Build Status" src="https://github.com/mellemahp/smithy4rs/workflows/Rust-CI/badge.svg">
   </a>
   <a href="https://github.com/mellemahp/smithy4rs/blob/main/LICENSE" title="License">
      <img alt="License" src="https://img.shields.io/badge/License-Apache_2.0-blue.svg">
   </a>
   <a href="https://crates.io/crates/smithy4rs-core" title="crates.io">
      <img alt="crates.io" src="https://img.shields.io/crates/v/smithy4rs-core">
   </a>
   <a href="https://deps.rs/repo/github/mellemahp/smithy4rs" title="dependencies">
      <img alt="dependencies" src ="https://deps.rs/repo/github/mellemahp/smithy4rs/status.svg">
   </a>
</p>

[Smithy](https://smithy.io/) code generators for shapes, clients, servers in [Rust](https://rust-lang.org/). 

Generated shapes provide runtime access to schema metadata and implement
schema-guided (de)serialization to support any number of protocols.

> [!NOTE]
> `smithy4rs` is an _unofficial_, _community-supported_ code generator.

---

This repository contains: 
1. `smithy-build` plugins that execute code generation from Smithy model definitions.
2. Core packages to support the runtime behavior of generated shapes.
3. Protocol definitions and other packages for building high-performance clients 
   and services with generated shapes.

## Documentation

For user-guides and general documentation see the [Documentation Site](https://mellemahp.github.io/smithy4rs/).

For API documentation see our [Rust docs](https://docs.rs/smithy4rs-core/latest/smithy4rs_core/).

## Getting started
> [!NOTE]
> If you are new to Smithy, we recommend going through the [Smithy Quickstart](https://smithy.io/2.0/quickstart.html)
> guide before using `smithy4rs`.

### Prerequisites
1. Ensure you have installed rust and cargo 
2. Ensure you have the Smithy CLI installed. To check if you have the CLI installed, Run `smithy --version` in your terminal.
   If you need to install the CLI, see the [Smithy CLI installation guide]().
3. Install [`cargo-generate`](https://github.com/cargo-generate/cargo-generate?tab=readme-ov-file#installation)

### Copy the template

Run the following command to copy the type-codegen template:

```console
cargo generate --git https://github.com:mellemahp/smithy4rs-type-codegen-template.git
```

This template uses the `rust-types` build plugin to generate standalone types from any Smithy models 
in the `model/` directory.

### Build 

Now, `cd` into the project directory created by `cargo generate` and run `cargo test` to build and 
test the generate shapes.

### Next steps
Try modifying the Smithy models in the `model/` directory to see what different shapes you can generate.

For a longer, guided introduction to this project, see the [`smithy4rs` Quick Start Guide](https://mellemahp.github.io/smithy4rs/quick-start.html).

## Core packages

* Core
  * [`smithy4rs-core`](core) - Core functionality for all generated shapes 
  * [`smithy4rs-core-derive`](core-derive) - Provides derive macros used to automatically implement
    schema-guided (de)serialization for generated shapes.

* Codegen 
  * [`codegen:core`](codegen/core) - Provides common functionality for all codegen plugins. Only plugins should depend on this directly. 
  * [`codegen:plugins`](codegen/plugins) - Aggregate package that provides all code generation plugins. Depend on this in `smithy-build.json` files.

* Codecs 
  * [`json`](json-codec) - Schema-guided (de)serialization for JSON.

## Security

See [CONTRIBUTING](CONTRIBUTING.md#security-issues) for more information.

## License

This project is licensed under the Apache-2.0 License.

