# Poller

Poller is a small tool for polling reference values ​​in [git repository]().
Whenever the latest reference value is obtained, it will publish the reference value to the subscribed RVPS through the publisher API.


To make this demo work, we use `redis` to be the underlying implementation.
It will use redis to provide in-memory and local publisher and subscriber (in RVPS).

## Usage

Set up a local redis instance via docker-compose

```bash
docker-compose up -d
```

Run the `poller`

```bash
cargo run
```

## Configurations about Publisher

- `Channel` name: `provenance`
