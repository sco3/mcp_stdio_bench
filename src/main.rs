use clap::Parser;
use rmcp::{
    model::CallToolRequestParams,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
    ServiceExt,
};
use serde_json::Value;
use std::fs::File;
use std::time::Instant;
use tokio::process::Command;
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_subscriber::{
    filter::LevelFilter, layer::SubscriberExt, prelude::*, util::SubscriberInitExt, Layer,
}; // Added LevelFilter

/// A benchmark tool for calling a method on an rmcp server over stdio.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the server executable.
    #[arg(long)]
    server: String,

    /// Number of calls to make.
    #[arg(long)]
    number: usize,

    /// name of the method to call.
    #[arg(long)]
    method: String,

    /// JSON parameters for the method call.
    #[arg(long)]
    params: Option<String>,

    /// Log level (e.g., "info", "debug", "trace").
    /// This level applies to both console and file logging if only one is active.
    /// If both are active, it applies to file logging.
    #[arg(long)]
    log_level: Option<String>,

    /// Path to the log file. If provided, logs will be written to this file.
    /// Console output will be suppressed if a log file is specified.
    #[arg(long)]
    log_file: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut _guard: Option<WorkerGuard> = None;

    let current_log_level = args.log_level.as_deref();

    let console_layer = if args.log_file.is_none() {
        let filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| current_log_level.unwrap_or("info").into());
        Some(tracing_subscriber::fmt::layer().with_filter(filter).boxed())
    } else {
        // When logging to file, suppress console output by setting LevelFilter::OFF
        Some(
            tracing_subscriber::fmt::layer()
                .with_filter(LevelFilter::OFF)
                .boxed(),
        )
    };

    let file_layer = if let Some(log_file_path) = &args.log_file {
        let file = File::create(log_file_path)?;
        let (non_blocking_appender, guard) = NonBlocking::new(file);
        _guard = Some(guard);

        let filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| current_log_level.unwrap_or("debug").into());

        Some(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(non_blocking_appender)
                .with_filter(filter)
                .boxed(),
        )
    } else {
        None
    };

    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .init();

    let parsed_params: Option<serde_json::Map<String, Value>> =
        if let Some(params_str) = &args.params {
            let value: Value = serde_json::from_str(params_str)?;
            if let Value::Object(map) = value {
                Some(map)
            } else {
                return Err("Params must be a JSON object.".into());
            }
        } else {
            None
        };

    let client = ()
        .serve(TokioChildProcess::new(
            Command::new(&args.server).configure(|cmd| {
                cmd.stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::null())
                    .env("RUST_LOG", args.log_level.unwrap());
            }),
        )?)
        .await?;

    let tool_call_params = CallToolRequestParams {
        meta: None,
        name: args.method.clone().into(),
        arguments: if let Some(p) = &parsed_params {
            Some(p.clone())
        } else {
            Some(object!({ "name": "world" }))
        },
        task: None,
    };

    let start_time = Instant::now();

    for _i in 0..args.number {
        // Send the method call as a tool call
        let _response = client.call_tool(tool_call_params.clone()).await?;
    }

    let elapsed_time = start_time.elapsed();

    println!("Total calls: {}", args.number);
    println!("Total time: {:?}", elapsed_time);
    let avg_time = if args.number == 0 {
        Default::default()
    } else {
        std::time::Duration::from_nanos((elapsed_time.as_nanos() / args.number as u128) as u64)
    };
    println!("Average time per call: {:?}", avg_time);

    Ok(())
}
