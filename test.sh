#!/bin/bash

# Test script for the Go MCP stdio benchmark tool

set -e

echo "Building Go benchmark tool..."
go build -o mcp_bench_go main.go

echo ""
echo "Running benchmark test..."
echo ""

target/release/mcp_stdio_benchmark --server servers_counter_stdio --number 80000 --method say_hello

echo ""
echo "Test completed successfully!"
