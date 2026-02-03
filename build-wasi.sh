#!/bin/bash
set -euo pipefail

# Build protoc-gen-prost as a WASI module with size optimizations

echo "Adding wasm32-wasip1 target..."
rustup target add wasm32-wasip1

echo "Building protoc-gen-prost for WASI (optimized for size)..."
cargo build --target wasm32-wasip1 --profile release-wasi -p protoc-gen-prost

# Create dist directory
mkdir -p dist

# Copy the WASM file
WASM_FILE="target/wasm32-wasip1/release-wasi/protoc_gen_prost.wasm"
if [ -f "$WASM_FILE" ]; then
    cp "$WASM_FILE" dist/protoc-gen-prost.wasm
    echo "WASM binary copied to dist/protoc-gen-prost.wasm"
else
    echo "Error: WASM file not found at $WASM_FILE"
    exit 1
fi

# Optimize with wasm-opt if available (binaryen)
# Enable bulk-memory feature since modern Rust uses memory.copy/memory.fill
if command -v wasm-opt &> /dev/null; then
    echo "Optimizing with wasm-opt..."
    wasm-opt -Oz --enable-bulk-memory dist/protoc-gen-prost.wasm -o dist/protoc-gen-prost.wasm
    echo "Optimization complete"
else
    echo "wasm-opt not found, skipping optimization (install binaryen to enable)"
fi

# Strip with wasm-strip if available (wabt)
if command -v wasm-strip &> /dev/null; then
    echo "Stripping with wasm-strip..."
    wasm-strip dist/protoc-gen-prost.wasm
    echo "Strip complete"
else
    echo "wasm-strip not found, skipping strip (install wabt to enable)"
fi

# Print file size
echo ""
echo "Final binary size:"
ls -lh dist/protoc-gen-prost.wasm

echo ""
echo "Exported functions:"
echo "  - prost_malloc(size) -> ptr"
echo "  - prost_free(ptr, size)"
echo "  - prost_execute(input_ptr, input_len) -> output_len"
echo "  - prost_get_output_ptr() -> ptr"
echo "  - prost_get_output_len() -> len"
echo "  - prost_clear_output()"
echo ""
echo "Build complete!"
