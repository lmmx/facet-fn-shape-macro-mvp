default: run test
precommit: run-ci test-ci

[working-directory: 'macro_test']
run:
    cargo nextest run

[working-directory: 'macro_test']
run-ci:
    cargo test

[working-directory: 'fn_shape_macro']
test:
    cargo nextest run

[working-directory: 'fn_shape_macro']
test-ci:
    cargo test
