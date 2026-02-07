use clap::Parser;
use rmcp::{
    ServiceExt,
    model::CallToolRequestParams,
    object,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use std::time::Instant;
use tokio::process::Command;
use serde_json::Value;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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
    #[arg(long)]
    log_level: Option<String>,

    /// Path to the log file.
    #[arg(long)]
    log_file: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    args.log_level
                        .as_deref()
                        .unwrap_or("info")
                        .into()
                }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let parsed_params: Option<serde_json::Map<String, Value>> = if let Some(params_str) = &args.params {
        let value: Value = serde_json::from_str(params_str)?;
        if let Value::Object(map) = value {
            Some(map)
        } else {
            return Err("Params must be a JSON object.".into());
        }
    } else {
        None
    };

    let client = ().serve(TokioChildProcess::new(Command::new(&args.server).configure(|cmd| {
        cmd.stdin(std::process::Stdio::piped())
           .stdout(std::process::Stdio::piped());
    }))?).await?;

    let start_time = Instant::now();

    for _i in 0..args.number {
        // Send the method call as a tool call
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
        let _response = client.call_tool(tool_call_params).await?;
    }

    let elapsed_time = start_time.elapsed();

    println!("Total calls: {}", args.number);
    println!("Total time: {:?}", elapsed_time);
    let avg_time = if args.number == 0 { Default::default() } else { std::time::Duration::from_nanos((elapsed_time.as_nanos() / args.number as u128) as u64) };
    println!(
        "Average time per call: {:?}",
        avg_time
    );

    Ok(())
}