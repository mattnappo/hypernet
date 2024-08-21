use anyhow::Result;
use clap::Parser;

use hypernet::node::Node;

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long, default_value_t = 8080)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    let mut node = Node::new(args.port)?;
    node.start().await?;

    Ok(())
}
