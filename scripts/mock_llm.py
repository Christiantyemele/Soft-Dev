from http.server import BaseHTTPRequestHandler, HTTPServer
import json

class MockAnthropicHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        content_length = int(self.headers['Content-Length'])
        data = json.loads(self.rfile.read(content_length))
        
        print(f"Mock LLM received request for model {data.get('model')}")
        
        # Hardcoded logic for the demo: Nexus always assigns T-101 to forge-1
        resp = {
            "id": "msg_mock_123",
            "type": "message",
            "role": "assistant",
            "model": "claude-3-5-sonnet-20240620",
            "content": [
                {"type": "text", "text": 'I have analyzed the current state. forge-1 is idle and T-101 is open.\n{"action": "work_assigned", "notes": "Assigning T-101 to forge-1", "assign_to": "forge-1", "ticket_id": "T-101"}'}
            ],
            "stop_reason": "end_turn",
            "usage": {"input_tokens": 100, "output_tokens": 50}
        }
        
        self.send_response(200)
        self.send_header('Content-type', 'application/json')
        self.end_headers()
        self.wfile.write(json.dumps(resp).encode('utf-8'))

    def log_message(self, format, *args):
        return # silence logs

def run(port=8082):
    server_address = ('', port)
    httpd = HTTPServer(server_address, MockAnthropicHandler)
    print(f'Starting mock LLM on port {port}...')
    httpd.serve_forever()

if __name__ == "__main__":
    run()
