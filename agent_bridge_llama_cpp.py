import json
import os

import requests

# Configuration for the separately running MCP server
MCP_SERVER_URL = os.environ.get("MCP_SERVER_URL", "http://127.0.0.1:8080/mcp")


def call_kubelens_tool(tool_name, arguments):
    """Send JSON-RPC request to kubelens-mcp via HTTP"""
    req = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {"name": tool_name, "arguments": arguments},
    }
    try:
        response = requests.post(MCP_SERVER_URL, json=req, timeout=10)
        response.raise_for_status()
        return response.json()
    except requests.exceptions.RequestException as e:
        raise RuntimeError(f"Failed to connect to MCP server at {MCP_SERVER_URL}: {e}")


# 2. Fetch data from Kubernetes via MCP
print("🔧 Fetching pod data via kubelens-mcp...")
tool_response = call_kubelens_tool("analyze_pod_failure", {"namespace": "default"})

# Debug: Print raw response to inspect structure and catch errors early
print(f"🔍 Raw MCP Response: {tool_response}")

if "error" in tool_response:
    print(f"❌ MCP Tool Error: {tool_response['error']}")
    pod_data = "Error: MCP tool returned an error."
else:
    result_data = tool_response.get("result")
    print(f"📦 result_data: {result_data}")

    if result_data is None:
        print("⚠️ result_data is None. Falling back to empty string.")
        pod_data = "No data returned from tool."
    else:
        # Handle standard MCP tool response format: [{"type": "text", "text": "..."}]
        if isinstance(result_data, list) and len(result_data) > 0:
            pod_data = result_data[0].get("text", "")
        elif isinstance(result_data, dict):
            pod_data = result_data.get("content", [{}])[0].get("text", "")
        else:
            pod_data = str(result_data)

print(f"✅ Parsed pod_data length: {len(pod_data)} chars")

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
