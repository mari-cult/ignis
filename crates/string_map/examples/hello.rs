use string_map::StringMap;

fn main() {
    let map = vec![("foo", 3.5_f32), ("bar", 7.0), ("foobar", 14.0)]
        .into_iter()
        .collect::<StringMap<_>>();

    println!("{map:?}");
    println!("starts_with");

    for pair in map.starts_with("f") {
        println!("{pair:?}");
    }

    println!("sub_sequence");

    for pair in map.sub_sequence("fb") {
        println!("{pair:?}");
    }
}
