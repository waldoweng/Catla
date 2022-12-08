use grpc::grpc_client::GrpcClient;
use grpc::HelloRequest;

pub mod grpc {
    tonic::include_proto!("catla");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cli = GrpcClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let resp = cli.say_hello(request).await?;
    print!("RESPONSE={:?}", resp);

    Ok(())
}
