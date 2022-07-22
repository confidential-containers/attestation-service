# IMPLEMENTATION

## Definition of terms

### CC

CNCF Confidential Containers.

### AS

Attestation Server.

### RVPS

Reference Value Provider Service.

### Pre-Processor

A component of RVPS, processing messages received before giving them to the Extractors.

### Extractors

A component of RVPS, which is responsible for process different kinds of provenance.

### Extractor

A submodule of Extractors, which is responsible for a specific kind of provenance.

### Extractor instance

A specific instance of Extractor class, responsible for processing provenance during runtime.

### Cache

A Key-Value storage for storing provenance and its reference values.

### BPDS

Binary Provenance Distribution Service

## Architecture

The base information stream through RVPS Core is as

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

### Pre-Processor

Pre-Processor contains a set of Wares (like Middleware). The Wares can
process the input Message and then deliver it to the Extractors.

### Extractors

Extractors has sub-modules to process different type of provenance.
Each sub-module will consume the input Message, and then generate
an output Reference Value.

### Cache

Cache is a trait object, which can provide `set` and `get` function.
All verified reference values will be stored in the Cache. When requested
by Attestation Service, related reference value will be provided.

## Protocols

### Message

A protocol helps to distribute provenance of binaries. It will be received and processed
by RVPS, then RVPS will generate Reference Value if working correctly. 

```
{
    "version": <VERSION-NUMBER-STRING>,
    "type": <TYPE-OF-THE-PROVENANCE-STRING>,
    "provenance": #provenance,
}
```

The `"version"` field is the version of this message, making extensibility possible.

The `"type"` field specifies the concrete type of the provenance the message carries.

The `"provenance"` field is the main content passed to RVPS. This field contains the payload to be decrypted by RVPS. 
The meaning of the provenance depends on the type and concrete Extractor which process this.

### Reference Value

A protocol that will be consumed by AS. A Reference Value may contain various fields,
so the implementation is a HashMap. Developers can freely add new fields in it to 
support different scenarios. 

```json
{
    "version" : "<REFERENCE_VALUE_VERSION>",
    "name" : "<NAME-OF-THE-ARTIFACT>",
    "hash-value" : [
        {
            "alg": "<HASH-ALGORITHM>",
            "value": "<HASH-VALUE>"
        },
        ...
    ],
    "expired":"<EXPIRED-TIME>"
}
```
The default value of `"version"` is `0.1`.