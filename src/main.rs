use hypernet::Hypercube;

fn main() {
    let cube = Hypercube::new(2);
    let pids = cube.start();

    println!("started nodes: {:?}", pids);
}
