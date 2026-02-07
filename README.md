# stdio-benchmark-tool

A benchmark tool for calling methods on an RMCP server over stdio.

## Build

To build the project, navigate to the project root and run:

```bash
cargo build --release
```

## Usage

This tool benchmarks the performance of calling methods on an RMCP server over standard I/O.

### Arguments

*   `--server <PATH_TO_SERVER_EXECUTABLE>`: Path to the RMCP server executable.
*   `--number <NUMBER_OF_CALLS>`: The number of times to call the specified method.
*   `--method <METHOD_NAME>`: The name of the method to call on the server.

### Example

To run a benchmark of 100 calls to the `say_hello` method on the `mcservers_counter_stdio` server (assuming it's located at `/home/dz/bin/mcservers_counter_stdio`):

```bash
cargo run --release -- --server /home/dz/bin/mcservers_counter_stdio --number 100 --method say_hello
```

The output will include the total number of calls, the total time taken, and the average time per call.

```
Total calls: 100
Total time: 8.10311ms
Average time per call: 81.031µs
```