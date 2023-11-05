use hypernet::{Hypernode, Identity};

use std::env;

const USAGE: &str = "usage: ./hypernode <id> <dimension> <port>";

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        panic!("{USAGE}");
    }

    let id = args[1].parse().expect("invalid node id");
    let d = args[2].parse().expect("invalid dimension");
    let port = &args[3];

    let mut node: Hypernode = Hypernode::new(
        Identity::new(
            id,
            format!("127.0.0.1:{port}")
                .parse()
                .expect("could not construct address"),
        ),
        d,
    );

    node.start().await
}
