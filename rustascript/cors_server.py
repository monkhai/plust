#!/usr/bin/env python3
"""
Simple HTTP server with CORS enabled for development.
Run: python3 cors_server.py
Then open: http://localhost:3000/canvas.html
"""
from http.server import HTTPServer, SimpleHTTPRequestHandler
import sys

class CORSRequestHandler(SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', '*')
        self.send_header('Cache-Control', 'no-store, no-cache, must-revalidate')
        super().end_headers()

    def do_OPTIONS(self):
        self.send_response(200)
        self.end_headers()

if __name__ == '__main__':
    port = 3000
    server = HTTPServer(('localhost', port), CORSRequestHandler)
    print(f'Server running on http://localhost:{port}')
    print(f'Open http://localhost:{port}/canvas.html in your browser')
    print('Press Ctrl+C to stop')
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print('\nServer stopped.')
        sys.exit(0)
