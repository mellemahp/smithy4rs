# Quick Start

This guide introduces [`smithy4rs`] with a simple working example of generating (de)serializable types .


```admonish tip title="New to Smithy?"
If you are new to Smithy, we recommend going through the [Smithy Quickstart](https://smithy.io/2.0/quickstart.html)
guide before using `smithy4rs`. The guide will walk you through the basics 
of creating a simple Smithy model.
```

## Prerequisites 
- Ensure you have [installed rust](https://rust-lang.org/tools/install/) and [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- Ensure you have the Smithy CLI installed. 
  - To check if you have the CLI installed, Run `smithy --version` in your terminal. 
  - If you need to install the CLI, see [the Smithy CLI installation guide](https://smithy.io/2.0/guides/smithy-cli/cli_installation.html).

## Setup

```admonish tip title="Fast-path: Templates" collapsible=true
TODO: Update once a template is available via cargo and smithy-init templating
```

### Create a new rust project

First, create a new Rust project by running `cargo init` in a new directory. 
This will set up the basic boilerplate for our crate.

Next, add [`smithy-cargo`](https://lib.rs/crates/smithy-cargo) as a build dependency: 
```bash
cargo add --build smithy-cargo
```
This tool will integrate the Smithy build tooling with Cargo. 

Also add `smithy4rs-core` as a dependency:
```bash
cargo add smithy4rs-core
```
This package defines the Smithy data model in rust and adds all 
the core functionality (such as serialization) for generate shapes.

### Set up Smithy build tooling

We will now set up `smithy-cargo` to execute the Smithy CLI as 
part of the `cargo` build process.

First, create a `build.rs` file at the root of your project 
and add the following build script:

```rust
use smithy_cargo::SmithyBuild;

fn main() {
  /// Executes the `smithy` CLI `build` command from cargo
  /// and configures some environment variables to point to the 
  /// generated output folder.
  SmithyBuild::new().execute().expect("Smithy Build failed");
}
```

Then, add a `smithy-build.json` file to the root of your project 
to configure the Smithy build:

```json
{
  "version": "1.0",
  "maven": {
    "dependencies": [
      "dev.hmellema.smithy4rs:type-codegen:1.0.0"
    ]
  },
  "plugins": {
    "rust-types": {
      "TODO": "ADD REAL CONFIG"
    }
  }
}
```

Now, we are ready to create our model!

## Event Model

For this quickstart we are going to generate a few event
shapes.

TODO: Come up with a bit more interesting framing

```smithy
namespace com.quickstart.example

/// Doc comment
structure EventA {
    int: Integer
}
```

## Using the generated shapes

To use our generated shapes, simply create a module add import 
them using the `generated_shapes!` macro.

```rust 
mod shapes {
    use smithy4rs_core::generated_shapes;

    generated_shapes![];
}
```

We can now use our generated shapes elsewhere in our rust code:

```rust 
fn main() {
  // Create a new, validated event instance
  let event = EventA::builder()
          .int(42)
          .build()
          .expect("Should Build");
  // pretty-print the event
  println!("{:#?}", event)
}
```

