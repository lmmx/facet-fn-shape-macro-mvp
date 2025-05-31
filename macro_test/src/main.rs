use fn_shape_macro::{facet_fn, fn_shape};

#[facet_fn]
fn add(x: i32, y: i32) -> i32 {
    x + y
}

fn main() {
    // Access metadata using fn_shape! macro
    println!("{:?}", fn_shape!(add));

    // Call functions normally
    println!("add(2, 3) = {}", add(2, 3));
}