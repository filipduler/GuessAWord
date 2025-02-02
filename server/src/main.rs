use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();

    let addr = &args[1];
    let password = &args[2];

    library::run_async(addr, password).await
}
