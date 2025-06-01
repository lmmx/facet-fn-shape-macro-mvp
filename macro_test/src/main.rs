use fn_shape_macro::{facet_fn, fn_shape};
use core::ops::Add;

#[facet_fn]
fn add(x: i32, y: i32) -> i32 { 
    x + y 
}

#[facet_fn]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[facet_fn]
fn no_params() -> &'static str {
    "No parameters here!"
}

#[facet_fn]
fn generic_add<T: Add<Output = T>>(
    x: T, y: T
) -> T
{
    x + y
}

fn main() {
    // Access metadata using fn_shape! macro
    println!("add shape: {:?}", fn_shape!(add));
    println!("greet shape: {:?}", fn_shape!(greet));
    println!("no_params shape: {:?}", fn_shape!(no_params));
    println!("generic_add<usize> shape: {:?}", fn_shape!(generic_add<usize>));
    println!("generic_add<i32> shape: {:?}", fn_shape!(generic_add<i32>));
    println!("generic_add<i64> shape: {:?}", fn_shape!(generic_add<i64>));
    
    // Call functions normally
    println!("add(2, 3) = {}", add(2, 3));
    println!(r#"greet("World") = {}"#, greet("World".to_string()));
    println!("no_params() = {}", no_params());
    println!("generic_add<usize>(2,3) = {}", generic_add::<usize>(2, 3));
    println!("generic_add<i32>(2,3) = {}", generic_add::<i32>(2, 3));
    println!("generic_add<i64>(2,3) = {}", generic_add::<i64>(2, 3));
}
