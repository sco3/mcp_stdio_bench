package main

import (
	"context"
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"log"
	"os"
	"time"

	"github.com/mark3labs/mcp-go/client"
	"github.com/mark3labs/mcp-go/mcp"
)

type Args struct {
	Server   string
	Number   int
	Method   string
	Params   string
	LogLevel string
	LogFile  string
}

func parseArgs() Args {
	args := Args{}
	flag.StringVar(&args.Server, "server", "", "Path to the server executable")
	flag.IntVar(&args.Number, "number", 0, "Number of calls to make")
	flag.StringVar(&args.Method, "method", "", "Name of the method to call")
	flag.StringVar(&args.Params, "params", "", "JSON parameters for the method call")
	flag.StringVar(&args.LogLevel, "log-level", "off", "Log level (e.g., info, debug, trace)")
	flag.StringVar(&args.LogFile, "log-file", "", "Path to the log file")
	flag.Parse()

	if args.Server == "" {
		log.Fatal("--server is required")
	}
	if args.Number <= 0 {
		log.Fatal("--number must be greater than 0")
	}
	if args.Method == "" {
		log.Fatal("--method is required")
	}

	return args
}

func setupLogging(logLevel, logFile string) {
	if logLevel == "off" {
		log.SetOutput(io.Discard)
		return
	}

	if logFile != "" {
		file, err := os.Create(logFile)
		if err != nil {
			log.Fatalf("Failed to create log file: %v", err)
		}
		log.SetOutput(file)
	} else {
		log.SetOutput(os.Stderr)
	}
}

func main() {
	args := parseArgs()
	setupLogging(args.LogLevel, args.LogFile)

	// Parse params if provided
	var params map[string]interface{}
	if args.Params != "" {
		if err := json.Unmarshal([]byte(args.Params), &params); err != nil {
			log.Fatalf("Failed to parse params: %v", err)
		}
	} else {
		params = map[string]interface{}{
			"name": "world",
		}
	}

	// Create stdio MCP client
	ctx := context.Background()
	mcpClient, err := client.NewStdioMCPClient(args.Server, nil)
	if err != nil {
		log.Fatalf("Failed to create MCP client: %v", err)
	}
	defer mcpClient.Close()

	// Consume stderr in background to prevent pipe blocking
	if stderr, ok := client.GetStderr(mcpClient); ok {
		go io.Copy(io.Discard, stderr)
	}

	// Initialize the connection
	initReq := mcp.InitializeRequest{
		Params: mcp.InitializeParams{
			ProtocolVersion: mcp.LATEST_PROTOCOL_VERSION,
			ClientInfo: mcp.Implementation{
				Name:    "mcp-bench-go",
				Version: "1.0.0",
			},
			Capabilities: mcp.ClientCapabilities{},
		},
	}
	if _, err := mcpClient.Initialize(ctx, initReq); err != nil {
		log.Fatalf("Failed to initialize: %v", err)
	}

	// Prepare tool call request
	toolRequest := mcp.CallToolRequest{
		Params: mcp.CallToolParams{
			Name:      args.Method,
			Arguments: params,
		},
	}

	// Benchmark loop
	startTime := time.Now()

	for i := 0; i < args.Number; i++ {
		_, err := mcpClient.CallTool(ctx, toolRequest)
		if err != nil {
			log.Fatalf("Tool call failed at iteration %d: %v", i, err)
		}
	}

	elapsed := time.Since(startTime)

	// Print results
	fmt.Printf("Total calls: %d\n", args.Number)
	fmt.Printf("Total time: %v\n", elapsed)
	avgTime := elapsed / time.Duration(args.Number)
	fmt.Printf("Average time per call: %v\n", avgTime)
	rps := float64(args.Number) / elapsed.Seconds()
	fmt.Printf("Requests per second (RPS): %.2f\n", rps)
}
