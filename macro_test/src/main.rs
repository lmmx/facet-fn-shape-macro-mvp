use fn_shape_macro::fn_shape;

#[fn_shape]
fn add(x: i32, y: i32) -> i32 {
    x + y
}

fn main() {
    // Access metadata directly (not add::SHAPE)
    println!("{:?}", SHAPE);

    // Call functions normally
    println!("add(2, 3) = {}", add(2, 3));
}
