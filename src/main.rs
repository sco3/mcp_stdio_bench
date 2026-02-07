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
    filter::LevelFilter, layer::SubscriberExt, util::SubscriberInitExt, Layer,
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
    #[arg(long, default_value = "off")]
    log_level: String,

    /// Path to the log file. If provided, logs will be written to this file.
    /// Console output will be suppressed if a log file is specified.
    #[arg(long)]
    log_file: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut _guard: Option<WorkerGuard> = None;

    let filter = tracing_subscriber::EnvFilter::new(&args.log_level);
    let builder = tracing_subscriber::fmt().with_env_filter(filter);

    if let Some(log_path) = &args.log_file {
        let file = std::fs::File::create(log_path)?;
        let (writer, guard) = tracing_appender::non_blocking(file);
        _guard = Some(guard);
        builder.with_writer(writer).with_ansi(false).init(); // Пишем в файл
    } else {
        builder.init(); // Пишем в консоль
    }
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
                    .env("RUST_LOG", args.log_level.clone());
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
