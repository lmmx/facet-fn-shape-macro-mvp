[working-directory: 'macro_test']
run:
    cargo run

[working-directory: 'fn_shape_macro']
test:
    cargo nextest run
