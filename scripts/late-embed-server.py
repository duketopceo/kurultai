#!/usr/bin/env python3
"""Late-interaction embed sidecar for Kurultai `[embed.late] backend = "http"`.

Serves per-token multi-vectors for a ColBERT-family model. The intended model
is Perplexity's `pplx-embed-v2-late-0.6b` (MIT, 128-dim per token); any
sentence-transformers `MultiVectorEncoder`-compatible checkpoint works.

Requires: pip install 'sentence-transformers>=6.0.0' 'transformers>=5.4.0'

    LATE_MODEL=perplexity-ai/pplx-embed-v2-late-0.6b \
    LATE_PORT=8790 \
    python3 scripts/late-embed-server.py

Then config.toml:

    [embed.late]
    backend = "http"
    url = "http://127.0.0.1:8790"
    token_dim = 128
    max_doc_tokens = 512

Contract (what kurultai's LateHttpEmbedder calls):
  POST /embed_query    {"texts": [".."]} -> {"vectors": [[[f32;dim];T];N]}
  POST /embed_document {"texts": [".."]} -> same
"""

import os
import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

from sentence_transformers import MultiVectorEncoder

MODEL = os.environ.get("LATE_MODEL", "perplexity-ai/pplx-embed-v2-late-0.6b")
PORT = int(os.environ.get("LATE_PORT", "8790"))

model = MultiVectorEncoder(MODEL)


def encode(kind: str, texts):
    fn = model.encode_query if kind == "query" else model.encode_document
    return [m.tolist() for m in fn(texts)]


class Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        kind = {"/embed_query": "query", "/embed_document": "document"}.get(
            self.path
        )
        if kind is None:
            self.send_error(404)
            return
        try:
            body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
            texts = body["texts"]
            assert isinstance(texts, list) and all(isinstance(t, str) for t in texts)
            out = encode(kind, texts)
        except Exception as e:  # noqa: BLE001 — surface the message to the caller
            self.send_response(500)
            self.send_header("Content-Type", "text/plain")
            self.end_headers()
            self.wfile.write(str(e).encode())
            return
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps({"vectors": out}).encode())

    def log_message(self, fmt, *args):  # keep stderr quiet per request
        pass


if __name__ == "__main__":
    print(f"late-embed sidecar: {MODEL} on 127.0.0.1:{PORT}")
    ThreadingHTTPServer(("127.0.0.1", PORT), Handler).serve_forever()
