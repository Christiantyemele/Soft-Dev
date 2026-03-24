import sys
import json

def main():
    for line in sys.stdin:
        if not line.strip():
            continue
        try:
            req = json.loads(line)
            method = req.get("method")
            msg_id = req.get("id")
            
            if method == "initialize":
                resp = {
                    "jsonrpc": "2.0",
                    "id": msg_id,
                    "result": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {},
                        "serverInfo": {"name": "mock-mcp", "version": "0.1.0"}
                    }
                }
            elif method == "tools/list":
                resp = {
                    "jsonrpc": "2.0",
                    "id": msg_id,
                    "result": {"tools": [
                        {
                            "name": "list_issues",
                            "description": "List GitHub issues",
                            "inputSchema": {"type": "object", "properties": {}}
                        }
                    ]}
                }
            elif method == "tools/call":
                tool_name = req.get("params", {}).get("name")
                if tool_name == "list_issues":
                    resp = {
                        "jsonrpc": "2.0",
                        "id": msg_id,
                        "result": {
                            "content": [{"type": "text", "text": json.dumps([
                                {"id": "T-101", "title": "Implement auth middleware", "body": "We need a JWT middleware in src/auth.rs", "status": "open"}
                            ])}]
                        }
                    }
                else:
                    resp = {"jsonrpc": "2.0", "id": msg_id, "result": {"content": [{"type": "text", "text": "Success"}]}}
            elif method == "notifications/initialized":
                continue # No response needed for notifications
            else:
                resp = {
                    "jsonrpc": "2.0",
                    "id": msg_id,
                    "result": {}
                }
            
            sys.stdout.write(json.dumps(resp) + "\n")
            sys.stdout.flush()
        except Exception as e:
            sys.stderr.write(f"Error: {e}\n")

if __name__ == "__main__":
    main()
