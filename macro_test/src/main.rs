use fn_shape_macro::{facet_fn, fn_shape};

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

fn main() {
    // Access metadata using fn_shape! macro
    println!("add shape: {:?}", fn_shape!(add));
    println!("greet shape: {:?}", fn_shape!(greet));
    println!("no_params shape: {:?}", fn_shape!(no_params));
    
    // Call functions normally
    println!("add(2, 3) = {}", add(2, 3));
    println!("{}", greet("World".to_string()));
    println!("{}", no_params());
}