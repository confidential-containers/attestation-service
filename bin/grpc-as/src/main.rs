use anyhow::Result;
use clap::{App, Arg};
use futures::future;
use shadow_rs::shadow;

pub mod as_api {
    tonic::include_proto!("attestation");
}

#[macro_use]
extern crate log;
shadow!(build);

mod server;
mod subscriber;

const DEFAULT_SOCK: &str = "127.0.0.1:3000";

const DEFAULT_REDIS_URL: &str = "redis://localhost/";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let version = format!(
        "\nv{}\ncommit: {}\nbuildtime: {}",
        build::PKG_VERSION,
        build::COMMIT_HASH,
        build::BUILD_TIME
    );

    let matches = App::new("grpc-attestation-service")
        .version(version.as_str())
        .long_version(version.as_str())
        .author("Confidential-Containers Team")
        .arg(
            Arg::with_name("socket")
                .long("socket")
                .value_name("SOCKET")
                .help("Socket that the server will listen on to accept requests.")
                .takes_value(true)
                .default_value(DEFAULT_SOCK)
                .required(false),
        )
        .arg(
            Arg::with_name("redis-url")
                .long("redis")
                .value_name("redis")
                .help("URL of redis to subscribe published reference values")
                .takes_value(true)
                .default_value(DEFAULT_REDIS_URL)
                .required(false),
        )
        .get_matches();

    let socket = matches.value_of("socket").expect("socket addr get failed.");
    let redis_url = matches
        .value_of("redis-url")
        .expect("redis-url get failed.");
    let server = server::start(socket, redis_url)?;
    future::try_join_all(server).await?;

    Ok(())
}
