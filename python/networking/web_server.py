"""Small Web Server — HTTP server with routing, middleware, and static file serving."""

from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Optional, Callable
from enum import Enum


class HttpMethod(Enum):
    GET = "GET"
    POST = "POST"
    PUT = "PUT"
    DELETE = "DELETE"
    HEAD = "HEAD"
    OPTIONS = "OPTIONS"


@dataclass
class HttpRequest:
    """An incoming HTTP request."""
    method: HttpMethod
    path: str
    headers: dict[str, str] = field(default_factory=dict)
    query_params: dict[str, str] = field(default_factory=dict)
    body: str = ""
    remote_addr: str = ""

    @staticmethod
    def parse(raw: str) -> "HttpRequest":
        lines = raw.split("\r\n") if "\r\n" in raw else raw.split("\n")
        if not lines:
            return HttpRequest(method=HttpMethod.GET, path="/")
        parts = lines[0].split(" ")
        method = HttpMethod(parts[0]) if len(parts) > 0 else HttpMethod.GET
        full_path = parts[1] if len(parts) > 1 else "/"
        path = full_path.split("?")[0]
        query_params: dict[str, str] = {}
        if "?" in full_path:
            qs = full_path.split("?", 1)[1]
            for pair in qs.split("&"):
                if "=" in pair:
                    k, v = pair.split("=", 1)
                    query_params[k] = v
        headers: dict[str, str] = {}
        body_start = None
        for i, line in enumerate(lines[1:], 1):
            if line == "":
                body_start = i + 1
                break
            if ":" in line:
                k, v = line.split(":", 1)
                headers[k.strip().lower()] = v.strip()
        body = "\n".join(lines[body_start:]) if body_start and body_start < len(lines) else ""
        return HttpRequest(method=method, path=path, headers=headers,
                           query_params=query_params, body=body)


@dataclass
class HttpResponse:
    """An outgoing HTTP response."""
    status_code: int = 200
    status_text: str = "OK"
    headers: dict[str, str] = field(default_factory=dict)
    body: str = ""

    def set_header(self, key: str, value: str) -> "HttpResponse":
        self.headers[key] = value
        return self

    def serialize(self) -> str:
        self.headers.setdefault("Content-Length", str(len(self.body.encode("utf-8"))))
        self.headers.setdefault("Content-Type", "text/html; charset=utf-8")
        header_lines = "\r\n".join(f"{k}: {v}" for k, v in self.headers.items())
        return f"HTTP/1.1 {self.status_code} {self.status_text}\r\n{header_lines}\r\n\r\n{self.body}"


def response_200(body: str, content_type: str = "text/html; charset=utf-8") -> HttpResponse:
    return HttpResponse(200, "OK", {"Content-Type": content_type}, body)

def response_404(path: str = "") -> HttpResponse:
    return HttpResponse(404, "Not Found", {}, f"404 Not Found: {path}")

def response_500(error: str = "") -> HttpResponse:
    return HttpResponse(500, "Internal Server Error", {}, f"500 Internal Server Error: {error}")

def response_redirect(location: str, permanent: bool = False) -> HttpResponse:
    code = 301 if permanent else 302
    text = "Moved Permanently" if permanent else "Found"
    return HttpResponse(code, text, {"Location": location}, "")

def response_json(data: str) -> HttpResponse:
    return HttpResponse(200, "OK", {"Content-Type": "application/json"}, data)


# Handler type
Handler = Callable[[HttpRequest], HttpResponse]
Middleware = Callable[[HttpRequest, Handler], HttpResponse]


@dataclass
class Route:
    """A registered route."""
    method: HttpMethod
    path: str
    handler: Handler
    name: str = ""


class Router:
    """URL router with exact and prefix matching."""

    def __init__(self) -> None:
        self.routes: list[Route] = []
        self.middlewares: list[Middleware] = []

    def add_route(self, method: HttpMethod, path: str, handler: Handler, name: str = "") -> None:
        self.routes.append(Route(method=method, path=path, handler=handler, name=name))

    def get(self, path: str, handler: Handler, name: str = "") -> None:
        self.add_route(HttpMethod.GET, path, handler, name)

    def post(self, path: str, handler: Handler, name: str = "") -> None:
        self.add_route(HttpMethod.POST, path, handler, name)

    def use(self, middleware: Middleware) -> None:
        self.middlewares.append(middleware)

    def resolve(self, method: HttpMethod, path: str) -> Optional[Handler]:
        for route in self.routes:
            if route.method == method and route.path == path:
                return route.handler
        # Prefix match for static dirs
        for route in self.routes:
            if route.method == method and route.path.endswith("/*") and \
               path.startswith(route.path[:-2]):
                return route.handler
        return None

    def handle(self, request: HttpRequest) -> HttpResponse:
        handler = self.resolve(request.method, request.path)
        if handler is None:
            return response_404(request.path)
        final_handler = handler
        for mw in reversed(self.middlewares):
            prev = final_handler
            final_handler = lambda req, _mw=mw, _prev=prev: _mw(req, _prev)
        return final_handler(request)


class WebServer:
    """A small web server with routing, middleware, static files, and request logging."""

    def __init__(self, name: str = "rosetta-server") -> None:
        self.name = name
        self.router = Router()
        self.static_files: dict[str, tuple[str, str]] = {}  # path -> (content, content_type)
        self.access_log: list[dict] = []

    def route(self, method: HttpMethod, path: str, handler: Handler, name: str = "") -> None:
        self.router.add_route(method, path, handler, name)

    def get(self, path: str, handler: Handler, name: str = "") -> None:
        self.router.get(path, handler, name)

    def post(self, path: str, handler: Handler, name: str = "") -> None:
        self.router.post(path, handler, name)

    def use(self, middleware: Middleware) -> None:
        self.router.use(middleware)

    def serve_static(self, path: str, content: str, content_type: str = "text/html") -> None:
        self.static_files[path] = (content, content_type)
        self.get(path, lambda req, _p=path: response_200(
            self.static_files[_p][0], self.static_files[_p][1]))

    def handle_request(self, request: HttpRequest) -> HttpResponse:
        response = self.router.handle(request)
        self.access_log.append({
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "method": request.method.value,
            "path": request.path,
            "status": response.status_code,
            "remote_addr": request.remote_addr,
        })
        return response

    def process_raw(self, raw_request: str) -> str:
        request = HttpRequest.parse(raw_request)
        response = self.handle_request(request)
        return response.serialize()


# --- demo ---
if __name__ == "__main__":
    server = WebServer("EM Server")

    server.get("/", lambda req: response_200("<h1>Eckenrode Muziekopname</h1>"))
    server.get("/health", lambda req: response_json('{"status":"ok"}'))
    server.get("/api/time", lambda req: response_json(
        f'{{"time":"{datetime.now(timezone.utc).isoformat()}"}}'))
    server.serve_static("/style.css", "body { background: #0A0A0F; color: #E8E4DC; }",
                        "text/css")

    # Simulate requests
    for raw in [
        "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n",
        "GET /health HTTP/1.1\r\n\r\n",
        "GET /api/time HTTP/1.1\r\n\r\n",
        "GET /missing HTTP/1.1\r\n\r\n",
    ]:
        resp = server.process_raw(raw)
        first_line = resp.split("\r\n")[0]
        print(first_line)
    print(f"\nAccess log: {len(server.access_log)} entries")
