#!/usr/bin/env bash
set -eou pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PARENT_DIR="$(dirname "$DIR")"

cd "$PARENT_DIR"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Expected values for validation
EXPECTED_OUTPUT="Found 214 matches above threshold 750. Results written to output.csv"
EXPECTED_LINE_COUNT=215  # 214 matches + 1 header line

# Function to log error and exit
log_error_and_exit() {
    local error_message="$1"
    printf "${RED}${BOLD}ERROR:${NC} %b\n" "$error_message" >&2
    exit 1
}

# Function to log info messages
log_info() {
    local message="$1"
    printf "${BLUE}${BOLD}INFO:${NC} %s\n" "$message"
}

# Function to log success messages
log_success() {
    local message="$1"
    printf "${GREEN}${BOLD}SUCCESS:${NC} %s\n" "$message"
}

# Function to run tests for a specific build
run_test() {
    local build_type="$1"
    local make_target="$2"
    # Clean up between builds
    rm -f duplexscan output.csv
    log_info "Cleaned up build artifacts and output.csv"

    log_info "Building duplexscan with ${build_type}..."
    if ! make "${make_target}"; then
        log_error_and_exit "Failed to build duplexscan with ${build_type}"
    fi
    log_success "Build successful with ${build_type}"

    log_info "Running smoke test for ${build_type} build..."

    # Run duplexscan and capture output
    ACTUAL_OUTPUT=$(./duplexscan -f input.csv -o output.csv -t 750)

    # Validate command output
    if [ "$ACTUAL_OUTPUT" != "$EXPECTED_OUTPUT" ]; then
        log_error_and_exit "Unexpected command output for ${build_type} build.\nExpected: ${EXPECTED_OUTPUT}\nActual: ${ACTUAL_OUTPUT}"
    fi

    # Validate output CSV line count
    if [ ! -f "output.csv" ]; then
        log_error_and_exit "output.csv file not found for ${build_type} build"
    fi

    ACTUAL_LINE_COUNT=$(wc -l < output.csv)
    if [ "$ACTUAL_LINE_COUNT" != "$EXPECTED_LINE_COUNT" ]; then
        log_error_and_exit "Unexpected line count in output.csv for ${build_type} build.\nExpected: ${EXPECTED_LINE_COUNT} lines\nActual: ${ACTUAL_LINE_COUNT} lines"
    fi

    log_success "Smoke test passed for ${build_type} build"
}

# Main execution
log_info "Starting smoke tests for all build types..."

# Test glibc build
run_test "glibc" "duplexscan-glibc"

# Test musl build
run_test "musl" "duplexscan"

log_success "All smoke tests completed successfully!"