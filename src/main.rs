use hypernet::Hypercube;

fn main() {
    // Start a cube
    let mut cube = Hypercube::new(2);
    cube.start();
    println!("made cube: {:#?}", cube);

    // Query the values
    let values = cube.query();
    println!("queried values: {:#?}", values);
}
