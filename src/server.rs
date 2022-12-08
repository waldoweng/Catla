use grpc::grpc_server::{Grpc, GrpcServer};
use grpc::{HelloReply, HelloRequest};
use tonic::{transport::Server, Request, Response, Status};

pub mod grpc {
    tonic::include_proto!("catla");
}

#[derive(Debug, Default)]
pub struct CatlaGrpcService {}

#[tonic::async_trait]
impl Grpc for CatlaGrpcService {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        print!("Got a request: {:?}", request);

        let reply = grpc::HelloReply {
            message: format!("Hello {}!", request.into_inner().name).into(),
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let grpc = CatlaGrpcService::default();

    Server::builder()
        .add_service(GrpcServer::new(grpc))
        .serve(addr)
        .await?;

    Ok(())
}
