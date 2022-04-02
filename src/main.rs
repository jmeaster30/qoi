mod qoi;

fn main() {
    println!("Hello, world!");
    qoi::qoi_decode(&[66, 79, 79, 66, 0, 0, 0, 60, 0, 0, 0, 60, 1, 1]);
}
