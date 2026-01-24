<h1 align="center">smithy4rs: Smithy code generators for Rust (unofficial)</h1>
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
</p>

[Smithy](https://smithy.io/) code generators for shapes, clients, servers in [Rust](https://rust-lang.org/). 

Generate shapes provide runtime access to schema metadata and implement
schema-guided (de)serialization to support any number of protocols.

> [!NOTE]
> `smithy4rs` is an _unofficial_, _community-supported_ code generator.

---

This repository contains: 
1. `smithy-build` plugins that execute code generation from Smithy model definitions.
2. Core packages to support the runtime behavior of generated shapes.
3. Protocol definitions and other packages for building high-performance clients 
   and services with generated shapes.

## Getting started
> [!NOTE]
> If you are new to Smithy, we recommend going through the [Smithy Quickstart](https://smithy.io/2.0/quickstart.html)
> guide before using `smithy4rs`.

For a guided introduction to this project, see the [`smithy4rs` Quick Start Guide](https://mellemahp.github.io/smithy4rs/quick-start.html).

## Documentation

For user-guides and general documentation see the [Documentation Site](https://mellemahp.github.io/smithy4rs/). 

For API documentation see our [Rust docs](https://docs.rs/smithy4rs-core/latest/smithy4rs_core/).

## Security

See [CONTRIBUTING](CONTRIBUTING.md#security-issues) for more information.

## License

This project is licensed under the Apache-2.0 License.

