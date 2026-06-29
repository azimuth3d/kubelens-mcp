import json
import subprocess

import requests

# 1. Start the kubelens-mcp server as a subprocess
mcp_server = subprocess.Popen(
    ["./target/release/kubelens-mcp"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    text=True,
)


def call_kubelens_tool(tool_name, arguments):
    """Send JSON-RPC request to kubelens-mcp via stdin"""
    req = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {"name": tool_name, "arguments": arguments},
    }
    mcp_server.stdin.write(json.dumps(req) + "\n")
    mcp_server.stdin.flush()

    # Read response from stdout, skipping empty lines and handling potential buffering
    response_line = ""
    while not response_line.strip():
        response_line = mcp_server.stdout.readline()
        if not response_line:
            raise RuntimeError("MCP server closed stdout or returned empty response")

    return json.loads(response_line)


# 2. Fetch data from Kubernetes via MCP
print("🔧 Fetching pod data via kubelens-mcp...")
tool_response = call_kubelens_tool("get_pod_failures", {"namespace": "default"})
result_data = tool_response.get("result")
print(f"result from tool: {result_data}")
pod_data = result_data.get("content", [{"text": ""}])[0]["text"]

print("Sending data to Local LLM (llama.cpp) for analysis...\n")
prompt = f"Analyze this Kubernetes pod status and summarize any issues:\n{pod_data}"
print(prompt)
# 3. Connect to llama.cpp server (OpenAI-compatible endpoint)
# Assuming llama-server is running on default port 8080
llama_cpp_url = "http://10.42.0.89:8080/v1/chat/completions"
payload = {
    "messages": [
        {
            "role": "system",
            "content": "You are an expert DevOps engineer and Kubernetes administrator.",
        },
        {"role": "user", "content": prompt},
    ],
    "temperature": 0.2,  # Keep temperature low for factual technical analysis
    "stream": False,
}

try:
    response = requests.post(llama_cpp_url, json=payload)
    response.raise_for_status()  # Check for HTTP errors

    print("🤖 LLM Analysis:")
    # Parse the response using the standard OpenAI format
    print(response.json()["choices"][0]["message"]["content"])

except requests.exceptions.ConnectionError:
    print("❌ Error: Could not connect to llama.cpp server.")
    print(
        "💡 Ensure you have started it, e.g.: ./llama-server -m your-model.gguf --port 8080"
    )
except Exception as e:
    print(f"❌ An error occurred: {e}")

# Clean up
mcp_server.terminate()
