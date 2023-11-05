use hypernet::Hypercube;

fn main() {
    let mut cube = Hypercube::new(2);
    cube.start();
    println!("made cube: {:#?}", cube);
}
