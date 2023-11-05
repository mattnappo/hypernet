use hypernet::Hypercube;

#[async_std::main]
async fn main() {
    // Start a cube
    let mut cube = Hypercube::new(2);
    cube.start();
    println!("made cube: {:#?}", cube);

    std::thread::sleep(std::time::Duration::from_millis(1000));

    // Query the values
    let values = cube.query().await;
    println!("queried values: {:#?}", values);
}
