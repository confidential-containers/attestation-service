# Simple Payload

Simple payload is the format of reference value format published by CoCo Community. 

## Format

The basic format is as following
```json
{
    "<key-1-of-the-parsed-claims-in-evidence>": "<reference-value-1>",
    "<key-2-of-the-parsed-claims-in-evidence>": "<reference-value-2>"
    ...
}
```

For example,

```json
{
    "tdx-kernel-size0x10000000": "5b7aa6572f649714ff00b6a2b9170516a068fd1a0ba72aa8de27574131d454e6396d3bfa1727d9baf421618a942977fa",
    "tdx-kernel-parameter": "64ed1e5a47e8632f80faf428465bd987af3e8e4ceb10a5a9f387b6302e30f4993bded2331f0691c4a38ad34e4cbbc627",
    ...
}
```
