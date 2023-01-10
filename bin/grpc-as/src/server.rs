use anyhow::{Context, Result};
use attestation_service::{AttestationService as Service, Tee};
use futures::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use crate::as_api::attestation_service_server::{AttestationService, AttestationServiceServer};
use crate::as_api::{AttestationRequest, AttestationResponse, Tee as GrpcTee};

type FutureVec = Vec<Pin<Box<dyn Future<Output = Result<(), anyhow::Error>>>>>;

fn to_kbs_tee(tee: GrpcTee) -> Tee {
    match tee {
        GrpcTee::Sev => Tee::Sev,
        GrpcTee::Sgx => Tee::Sgx,
        GrpcTee::Snp => Tee::Snp,
        GrpcTee::Tdx => Tee::Tdx,
    }
}

pub struct AttestationServer {
    attestation_service: Arc<RwLock<Service>>,
}

impl AttestationServer {
    pub fn new(attestation_service: Arc<RwLock<Service>>) -> Result<Self> {
        Ok(Self {
            attestation_service,
        })
    }
}

#[tonic::async_trait]
impl AttestationService for AttestationServer {
    async fn attestation_evaluate(
        &self,
        request: Request<AttestationRequest>,
    ) -> Result<Response<AttestationResponse>, Status> {
        let request: AttestationRequest = request.into_inner();

        debug!("Evidence: {}", &request.evidence);

        let attestation_results = self
            .attestation_service
            .read()
            .await
            .evaluate(
                to_kbs_tee(
                    GrpcTee::from_i32(request.tee)
                        .ok_or_else(|| Status::aborted(format!("Invalid TEE {}", request.tee)))?,
                ),
                &request.nonce,
                &request.evidence,
            )
            .await
            .map_err(|e| Status::aborted(format!("Attestation: {e}")))?;

        let results = serde_json::to_string(&attestation_results)
            .map_err(|e| Status::aborted(format!("Parse attestation results: {e}")))?;

        debug!("Attestation Results: {}", &results);

        let res = AttestationResponse {
            attestation_results: results,
        };
        Ok(Response::new(res))
    }
}

pub fn start(socket: &str, redis_url: &str) -> Result<FutureVec> {
    debug!("Listen socket: {}", socket);
    let socket = socket.parse().context("parse socket addr failed")?;
    let service = Service::new().context("create AS failed")?;
    let inner = Arc::new(RwLock::new(service));
    let attestation_server = AttestationServer::new(inner.clone())?;

    let redis_url = redis_url.to_string();
    let subscriber_client = Box::pin(crate::subscriber::subscribe(redis_url, inner));

    let grpc_server = Box::pin(async move {
        Server::builder()
            .add_service(AttestationServiceServer::new(attestation_server))
            .serve(socket)
            .await
            .context("gRPC error")
    });

    Ok(vec![grpc_server, subscriber_client])
}
