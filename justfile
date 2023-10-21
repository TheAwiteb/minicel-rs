# This justfile is for the contrbutors of this project, not for the end user.

set shell := ["/usr/bin/bash", "-c"]

JUST_EXECUTABLE := "just -u -f " + justfile()
header := "Available tasks:\n"
# Get all tests from the justfile and put them in a string
tests := `just --summary | rg -o "test_[[A-Za-z0-9]|_]+" | xargs`
# Get the MSRV from the Cargo.toml
msrv := `cat Cargo.toml | rg "rust-version" | sed 's/.*"\(.*\)".*/\1/'`


_default:
    @{{JUST_EXECUTABLE}} --list-heading "{{header}}" --list

# Run the CI
@ci: 
    cargo +stable build -q
    cargo +stable fmt -- --check
    cargo +stable clippy -- -D warnings
    {{JUST_EXECUTABLE}} tests
    {{JUST_EXECUTABLE}} msrv

# Run the tests
@tests:
    {{JUST_EXECUTABLE}} {{tests}}

# Run CSV tests
@test_csv:
    echo "CSV tests"
    cargo +stable test -q

# Check that the current MSRV is correct
@msrv:
    echo "Checking MSRV ({{msrv}})"
    cargo +{{msrv}} check -q    
    echo "MSRV is correct"

alias b := bench
# Run the benchmarks
@bench:
    echo "Running benchmarks"
    cargo b -r
    hyperfine  "target/release/minicel test.csv out.csv"

