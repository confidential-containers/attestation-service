# Extractor Development Guide

This guide will teach you how to develop a Extractor module and integrate it into the source code of the RVPS according to the Extractors module standard interface and integration mode. Please refer to [IMPLEMENTATION.md](IMPLEMENTATION.md)  for the details of RVPS module framework.

Now let's start.

## Development

First, create a new Extractor module (e.g, my_extractor) as following:

```
cd reference-value-provider-service/lib/src/extractors/extractor_modules
mkdir my_extractor
cd my_extractor
touch mod.rs
```

Then, you need to import the definition of Extractor module standard interface in mod.rs, that is, add the following codes in mod.rs:

```rust
use super::Extractor;
```

Add the implementations for my_extractor module.

```rust
// lib/src/extractors/extractor_modules/my_extractor/mod.rs

pub struct MyExtractor {
    // The object of this structure will exist as an Extractor instance
    ... ...
}

impl Extractor for MyExtractor {
    // Verify and extract from the provenance.
    // Used to firstly parse provenance, and then verify it.
    // If the verification passes, extract relative 
    // reference value from it.
    // Input parameter: provenance from Message
    // Return value: ReferenceValue
    fn verify_and_extract(&self, provenance: &str) -> Result<ReferenceValue> {...}
}

impl MyExtractor {
    // The following is the function to create Extractor instance object
    // This function needs to be integrated into ExtractorModuleList of Extractors,
    // So its parameters and return value format must be implemented according to the example given here.
    fn new() -> MyExtractor {...}
    ...
}
```

The detailed Extractor module implemention requires verification of and extraction 
from the given provenance. Of course, deserialization of the `provenance` (type `&str`) field into
target provenance type is needed.


Also, documents about the provenance format SHOULD be attached.

## Integration

You need to integrate my_extractor module into RVPS as following:

1. Import my_extractor module: 

```rust
// lib/src/extractors/extractor_modules/mod.rs

// Add my specific extractor declaration here.
// For example: "pub mod my_extractor;"
#[cfg(feature = "my_extractor")]
pub mod my_extractor;
```

2. Register the function to create Extractor instance in ExtractorModuleList: 

```rust
// lib/src/extractors/extractor_modules/mod.rs

impl ExtractorModuleList {
    fn new() -> ExtractorModuleList {
        let mut mod_list = HashMap::new();

        #[cfg(feature = "my_extractor")]
        {
            let instantiate_func: ExtractorInstantiateFunc = Box::new(|| -> ExtractorInstance {
                Box::new(my_extractor::MyExtractor::new())
            });
            mod_list.insert("my_extractor".to_string(), instantiate_func);
        }

        ExtractorModuleList { mod_list: mod_list }
    }
```

3. Add the compilation options for my_extractor in Cargo.toml:

```
# Cargo.toml

[features]
default = ["my_extractor"]
my_extractor = []
```

## Compilation

After development and integration, you can compile the reference-value-provider-service 
that supports your Extractor module. You only need to specify feature parameter during compilation:

```
cargo build --release --no-default-features --features my_extractor
```
