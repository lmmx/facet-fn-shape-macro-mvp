use core::ops::Add;
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

#[facet_fn]
fn generic_add<T: Add<Output = T>>(x: T, y: T) -> T {
    x + y
}

fn main() {
    // Access metadata using fn_shape! macro
    println!("add shape: {:?}", fn_shape!(add));
    println!("greet shape: {:?}", fn_shape!(greet));
    println!("no_params shape: {:?}", fn_shape!(no_params));
    println!(
        "generic_add<usize> shape: {:?}",
        fn_shape!(generic_add<usize>)
    );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_functionality() {
        // 1) Verify that add(2, 3) returns 5
        assert_eq!(add(2, 3), 5);

        // 2) Check the FunctionShape fields for `add`
        let shape = fn_shape!(add);
        assert_eq!(shape.name, "add");
        assert_eq!(shape.param_count, 2);
        assert_eq!(shape.param_names, vec!["x", "y"]);
    }

    #[test]
    fn test_greet_functionality() {
        // 1) Verify that greet("Alice") returns "Hello, Alice!"
        let input = "Alice".to_string();
        assert_eq!(greet(input.clone()), format!("Hello, {}!", input));

        // 2) Check the FunctionShape fields for `greet`
        let shape = fn_shape!(greet);
        assert_eq!(shape.name, "greet");
        assert_eq!(shape.param_count, 1);
        assert_eq!(shape.param_names, vec!["name"]);
    }

    #[test]
    fn test_no_params_functionality() {
        // 1) Verify that no_params() returns the expected &str
        assert_eq!(no_params(), "No parameters here!");

        // 2) Check the FunctionShape fields for `no_params`
        let shape = fn_shape!(no_params);
        assert_eq!(shape.name, "no_params");
        assert_eq!(shape.param_count, 0);
        assert!(shape.param_names.is_empty());
    }

    #[test]
    fn test_generic_add_functionality() {
        // 1) Verify that generic_add works for i32, i64, usize
        assert_eq!(generic_add::<i32>(4, 5), 9);
        assert_eq!(generic_add::<i64>(10, 20), 30);
        assert_eq!(generic_add::<usize>(7, 8), 15);

        // 2) Check the FunctionShape fields for each concrete instantiation
        let shape_usize = fn_shape!(generic_add<usize>);
        assert_eq!(shape_usize.name, "generic_add");
        assert_eq!(shape_usize.param_count, 2);
        assert_eq!(shape_usize.param_names, vec!["x", "y"]);

        let shape_i32 = fn_shape!(generic_add<i32>);
        assert_eq!(shape_i32.name, "generic_add");
        assert_eq!(shape_i32.param_count, 2);
        assert_eq!(shape_i32.param_names, vec!["x", "y"]);

        let shape_i64 = fn_shape!(generic_add<i64>);
        assert_eq!(shape_i64.name, "generic_add");
        assert_eq!(shape_i64.param_count, 2);
        assert_eq!(shape_i64.param_names, vec!["x", "y"]);
    }
}
