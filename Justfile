default: run test
precommit: default

[working-directory: 'macro_test']
run:
    cargo nextest run

[working-directory: 'fn_shape_macro']
test:
    cargo nextest run
