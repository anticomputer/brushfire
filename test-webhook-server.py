#!/usr/bin/env python3
"""Simple webhook receiver for testing brushfire policy events."""

import json
from http.server import HTTPServer, BaseHTTPRequestHandler
from datetime import datetime

class WebhookHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        content_length = int(self.headers['Content-Length'])
        body = self.rfile.read(content_length)

        try:
            event = json.loads(body)
            timestamp = datetime.now().strftime('%H:%M:%S')

            print(f"\n[{timestamp}] Policy Event Received:")
            print(f"  Type: {event.get('event_type')}")
            print(f"  Resource: {event['action']['resource']}")
            print(f"  Mode: {event['action']['mode']}")

            # Display args for command spawn checks
            if 'args' in event['action'] and event['action']['args']:
                print(f"  Args: {' '.join(event['action']['args'])}")

            print(f"  Result: {event['result']}")
            print(f"  Reason: {event['reason']}")
            print(f"  Session: {event.get('session_id', 'N/A')[:8]}...")

            # Send success response
            self.send_response(200)
            self.send_header('Content-Type', 'application/json')
            self.end_headers()
            self.wfile.write(b'{"status": "received"}')

        except Exception as e:
            print(f"Error processing webhook: {e}")
            self.send_response(500)
            self.end_headers()

    def log_message(self, format, *args):
        # Suppress default logging
        pass

if __name__ == '__main__':
    port = 8765
    server = HTTPServer(('localhost', port), WebhookHandler)
    print(f"Webhook server listening on http://localhost:{port}")
    print("Waiting for policy events...\n")

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down webhook server...")
        server.shutdown()
