use loopy::Loop;

fn main() {
    let _device = Loop::options()
        .offset(1024)
        .read_only(true)
        .open("package.mocha")
        .unwrap();
}
