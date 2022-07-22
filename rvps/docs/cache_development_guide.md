# Cache Development Guide

This guide will teach you how to develop a Cache module and integrate it into the source code of the RVPS. Please refer to [IMPLEMENTATION.md](IMPLEMENTATION.md)  for the details of RVPS module framework.

Now let's start.

## Development

First, create a new Cache module (e.g, simple) as following:

```
cd reference-value-provider-service/lib/src/cache
mkdir simple
cd simple
touch mod.rs
```

Then, you need to import the definition of Cache module standard interface in mod.rs, that is, add the following codes in mod.rs:

```rust
use super::Cache;
```

Add the implementations for Simple module.

```rust
// lib/src/cache/simple/mod.rs

pub struct SimpleCache {
    // The object of this structure will exist as an Cache instance
    ... ...
}

impl Cache for SimpleCache {
    // store a key-value pair (<artifact-name>, <reference-value>) into the storage
    fn set(&mut self, name: String, rv: ReferenceValue) -> Result<()> {...}

    // get a key-value pair (<artifact-name>, <reference-value>) from the storage
    // by the key (<artifact-name>)
    fn get(&self, name: &str) -> Result<Option<ReferenceValue>> {...}
}

impl SimpleCache {
    // This function will instanlize a new object
    fn new() -> SimpleCache {...}
    ...
}
```

The detailed implementation of the Cache will be contents for `set` and `get`.
No matter how the underlying storage engine works, is must implement the two
interfaces well.

## Integration

When a specific Cache is chosen to use in the RVPS instance, new it like this

```rust
let mut core = Core::new(SimpleCache::new());
```

Here the RVPS core will use SimpleCache as Cache.

And a test using `set` and `get` interface is as following

```rust
    #[test]
    #[serial]
    fn test_core() {
        let mut core = Core::new(SimpleCache::new());
        let message = Message {
            version: MESSAGE_VERSION.into(),
            typ: "in-toto".into(),
            payload: generate_in_toto_provenance(),
        };
        core.verify_and_extract(message).unwrap();
        let rv = ReferenceValue::new()
            .set_name("foo.tar.gz")
            .set_expired(Utc.ymd(1970, 1, 1).and_hms(0, 0, 0))
            .set_version("0.1")
            .add_hash_value("sha256".into(), sha256_for_in_toto_test_artifact());
        let res = core.get_rv("foo.tar.gz").unwrap();
        assert_eq!(res, Some(rv));
    }
```