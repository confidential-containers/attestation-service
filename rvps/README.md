# Reference Value Provider Service

This repo implements RVPS as [proposal](https://github.com/confidential-containers/documentation/issues/37)
for confidential containers.

## Usage

The crate interface is pretty simple.

```rust
// Instantialize a new RVPS core instance
// with a simple kv store
let mut core = rvps::Core::new(SimpleCache::new());

// process the input message and generate reference value
// rv will be stored in to the core's cache
core.verify_and_extract(message).unwrap();

// get rv from the core
let rv = core.get_rv("<ARTIFACT_NAME>").unwrap();
```

The `core` can be integrite with multiple self-customized Wares
```rust
// Instantialize a new RVPS core instance
let mut core = rvps::Core::new(SimpleCache::new());

// With a example log ware
core.with_ware(Box::new(LogWare::new()));

// process the input message and generate reference value.
// Due to LogWare, the message's provenance type will be 
// logged
core.verify_and_extract(message).unwrap();
```

## Implementation

Please refer to [IMPLEMENTATION](docs/IMPLEMENTATION.md)

```
          Message 
            v
            v verify_and_extract()
            v
+------------v---------------+
| +-----------------------+  |
| |                       |  |
| |     Pre-Processor     |  |
| |                       |  |
| +-----------------------+  |
|             v              |
|     Message (Modified)     |
|             v              |
| +-----------------------+  |
| |                       |  |
| |     Extractors        |  |
| |                       |  |
| +-----------------------+  |         
|             v              |
|     Reference Value        |
|             v              |
| +-----------------------+  |
| |                       |  |
| |         Cache         |  |  ....get_rv()..> Reference Value
| |                       |  |
| +-----------------------+  |     
+----------------------------+
```

## Development Guide

| Component Name     | README                                                                      |
| ------------------ | --------------------------------------------------------------------------- |
| Pre-Processor Ware | [Pre-Processor Development Guide](docs/preprocessor_development_guide.md)   |
| Extractor          | [Extractor Development Guide](docs/extractor_development_guide.md)          |
| Cache              | [Cache Development Guide](docs/cache_development_guide.md)                  |

## Supported Provenance Types

| Provenance         | README                                                              |
| ------------------ | ------------------------------------------------------------------- |
| in-toto            | [In-toto](lib/src/extractors/extractor_modules/in_toto/README.md)   |

## Supported Cache Types

| Cache Type         | README                                                              |
| ------------------ | ------------------------------------------------------------------- |
| Simple Cache       | [Simple Cache](lib/src/cache/simple/README.md)                      |