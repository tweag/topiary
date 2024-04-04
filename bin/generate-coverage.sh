#!/usr/bin/env bash
#shellcheck shell=bash

set -eu

# Create temporary working directory
WORKING_DIR="$(mktemp --directory)"
readonly WORKING_DIR

trap 'rm -rf "${WORKING_DIR}"' EXIT

# Setup subdirectories for rustup and the profile data
RUSTUP_HOME="${WORKING_DIR}/rustup"
export RUSTUP_HOME

PROFRAW_DIR="${WORKING_DIR}/profraw"
readonly PROFRAW_DIR

mkdir --parents "${RUSTUP_HOME}" "${PROFRAW_DIR}"

# Install Rust toolchain and necessary components
rustup default stable
rustup component add llvm-tools-preview

# Build and output profiling data
CARGO_INCREMENTAL=0 \
RUSTFLAGS='-Cinstrument-coverage' \
LLVM_PROFILE_FILE="${PROFRAW_DIR}/cargo-test-%p-%m.profraw" \
cargo test

# Render HTML coverage report
REPORT_DIR="target/coverage/html"
readonly REPORT_DIR
mkdir --parents "${REPORT_DIR}"
grcov --branch \
      --output-type html \
      --source-dir src \
      --binary-path target/debug/deps \
      --output-path "${REPORT_DIR}" \
      "${PROFRAW_DIR}"
