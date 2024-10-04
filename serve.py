#!/usr/bin/env python3

import argparse
import http.server
import os
import socketserver


parser = argparse.ArgumentParser()
parser.add_argument("--address", "-a", type=str, default="0.0.0.0")
parser.add_argument("--port", "-p", type=int, default=8080)
args = parser.parse_args()

addr: str = args.address
port: int = args.port


class RequestHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        super(RequestHandler, self).end_headers()


for k, v in {
    ".html": "text/html",
    ".wasm": "application/wasm",
}.items():
    RequestHandler.extensions_map[k] = v
    with socketserver.TCPServer((addr, port), RequestHandler) as server:
        print(f"HTTP Server listening at port {port} ..")
        try:
            server.serve_forever()
        except KeyboardInterrupt:
            pass
