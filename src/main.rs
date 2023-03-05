use crate::key_gen::{make_keys, print_key};

mod key_gen;

fn main() {
    println!("Hello, world!");

    let keys = make_keys();
    print_key(keys[0])
    //print!("{}\n", affine_transform(number));

    // let value_1 = output[0];
    // let string_utf8_result = String::from_utf8_lossy(&output);
    // println!("{}", string_utf8_result);
}
