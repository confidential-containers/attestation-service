# Pre-processor Development Guide

This guide will teach you how to develop a Ware module and integrate it into the source code of the RVPS according to the Pre-processor module standard interface and integration mode. Please refer to [IMPLEMENTATION.md](IMPLEMENTATION.md)  for the details of RVPS module framework.

Now let's start.

## Development

First, create a new Ware module (e.g, LogWare, which can log all the messages going through. Other functionalities can be implemented as this) as following:

```
cd reference-value-provider-service/lib/src/pre_processor/ware
touch log.rs
```

Then, you need to import the definition of `Ware` module standard interface and `Next` in `log.rs`, that is, add the following codes in `log.rs`:

```rust
use crate::pre_processor::{Next, Ware}
```

Add the implementations for log module.

```rust
// lib/src/pre_processor/ware/log.rs

pub struct LogWare {
    // The object of this structure will exist as an LogWare instance
    ... ...
}

impl Ware for LogWare {
    // Process the input message. Each ware can modify, deny or do any 
    // operations needed.
    // Input parameter: 
    // - message: Reference of the being handled message, all the modification
    // can be directly performed on this.
    // - context: A HashMap which can deliver context information among Wares.
    // - next: Used to deliver the current message and context being 
    // processed to the next ware. 
    // Return value: Empty
    fn handle(
        &self,
        message: &mut Message,
        context: &mut HashMap<String, String>,
        next: Next<'_>,
    ) -> Result<()>  {
        // operations
        // In this scenary the type of messaged will be logged.
        ...
        
        // deliver to the next ware
        next.run(message, context)   
    }
}

impl LogWare {
    // The following is the function to create Ware instance object
    // This function is needed when initializing a Core.
    fn new() -> LogWare {...}
    ...
}
```

The detailed Ware implemention requires can be decided on your own.

## Integration

When the Ware is needed to work, you need to integrate LogWare module into RVPS as following:

1. Instantialize a new Ware and `with` it when initializing the Core: 

```rust
let mut core = Core::new();

core.with_ware(Box::new(LogWare::new()));
```

Now the Ware is successfully integrited into RVPS!

## Test

A simple test for the `LogWare`

And let's write a test for the log

```rust
    extern crate testing_logger;

    #[test]
    fn test_core_with_ware() {
        testing_logger::setup();
        let mut core = Core::new(SimpleCache::new());
        core.with_ware(Box::new(LogWare::new()));
        let message = Message {
            version: MESSAGE_VERSION.into(),
            typ: "in-toto".into(),
            payload: generate_in_toto_provenance(),
        };
        core.verify_and_extract(message).unwrap();

        testing_logger::validate( |captured_logs| {
            assert_eq!(captured_logs[0].body, "Get a new provenance of type: in-toto");
            assert_eq!(captured_logs[0].level, Level::Info);
        });
    }
```