use clap::{arg, Command};
use futures::executor::block_on;
use grpc::grpc_client::GrpcClient;
use grpc::HelloRequest;

pub mod grpc {
    tonic::include_proto!("catla");
}

fn cli() -> Command {
    Command::new("catla-cli")
        .about("Client to interact with catla")
        .author("waldoweng@gmail.com")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(
            Command::new("query")
                .about("Issue query to catla")
                .arg(arg!(<Addr> "Addr to catla server"))
                .arg(arg!(--expr <Expr>).required(true)),
        )
}

fn main() {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("query", sub_matches)) => {
            println!(
                "Query {} with query: {}",
                sub_matches.get_one::<String>("Addr").expect("required"),
                sub_matches.get_one::<String>("expr").expect("required")
            );
            let future = send_req();
            block_on(future).expect("send request fail")
        }
        _ => unreachable!(),
    }
}

#[tokio::main]
async fn cli_init() {}

async fn send_req() -> Result<(), Box<dyn std::error::Error>> {
    let mut cli = GrpcClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let resp = cli.say_hello(request).await?;
    print!("RESPONSE={:?}", resp);

    Ok(())
}
