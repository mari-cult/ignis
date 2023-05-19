use mocha_ident::Ident;

fn main() {
    let ident = "hello world".parse::<Ident<12>>();

    println!("{ident:?}");
}
