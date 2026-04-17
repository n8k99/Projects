# G046 — Small Web Server

> HTTP server with routing, middleware, and static file serving.

```yaml
id: G046
title: Small Web Server
category: networking
requires: [G022-rss-feed, G030-text-to-html, G033-ftp-protocol, G044-remote-login]
provides: [request-handling, url-routing, response-generation, access-logging]
```

## Insight: The Other Side of the Boundary

Every networking project before G046 was a **client** — sending requests, consuming responses. The web server is the first **server** — receiving requests, producing responses. This completes the bilateral protocol from G033 (FTP): we built the client side then; now we build the server side. The same protocol, viewed from the opposite end.

This is architecturally significant for the noosphere. Every ghost has been a requester — asking for data, probing services, checking mail. The web server model says: a ghost can also *serve*. `@kathryn/finance_positions` isn't just Kathryn requesting data — it's Kathryn's service *endpoint* that other agents call. Every capability in the Ghost Registry is a route in Kathryn's web server.

## Insight: Routing Is `match` Applied to URLs

The router matches a request's method and path to a handler function. `GET /health` → `health_handler`. `POST /api/data` → `data_handler`. This is G013's prefix dispatch and G014's range dispatch applied to URL paths. The route table IS the resolver's namespace: each entry maps an identifier (the URL) to a handler (the function).

The resolver's `@reference` resolution is routing. `@kathryn/finance_positions` is `GET /kathryn/finance_positions` — the method is implicit (read), the path identifies the resource, the handler is the capability. URL routing and reference resolution are the same algorithm.

## Insight: Request-Response Is the Universal Protocol Shape

The HTTP request-response cycle: client sends a structured request (method, path, headers, body), server returns a structured response (status, headers, body). This is the shape of every interaction in the Rosetta Stone: FTP commands, NTP queries, chat messages, weather API calls. HTTP just formalizes it with a standard structure.

In InnateScript, every `@reference` resolution follows this shape. The request: what do you want (`@reference`), with what context (`{params}`). The response: the resolved value, with metadata (type, freshness, confidence). HTTP makes this explicit. The resolver has been doing it implicitly since G001.

## Insight: Static File Serving Is Cached Resolution

`serve_static("/style.css", content)` maps a path to a fixed response. No computation — just lookup and return. This is the resolver's cache: if the value hasn't changed, return the stored result without re-computing. Static files are the degenerate case of resolution where the answer never changes.

## Insight: The Access Log Is the Observer's Feed

Every request generates a log entry: method, path, status, timestamp. The access log is the packet sniffer (G040) applied to HTTP — a chronological record of who asked for what and what they got. The server is simultaneously serving and observing itself.

## Choreographic Case: The dpn-api-client Is a Web Server

Nathan's `dpn-api-client` at `144.126.251.126:8080` is exactly this: a server that receives IPC requests, routes them to handlers (read, write, refresh), and returns JSON responses. The Rosetta Stone's web server is the simplified version of what's already running on the droplet.

```innate
(@api-server){
  @server <- @web-server{name: "dpn-api", port: 8080}
  @server/route{method: :get, path: "/health", handler: @health_check}
  @server/route{method: :post, path: "/read", handler: @db_read}
  @server/route{method: :post, path: "/write", handler: @db_write}
  @server/route{method: :post, path: "/refresh", handler: @refresh_handler}
  
  // Each route is a capability endpoint. The route table IS the Ghost Registry
  // for this service, expressed as URL paths instead of @reference names.
}
```

## Structures

```innate
(defstruct http-request
  method       : :get | :post | :put | :delete
  path         : String
  headers      : {String -> String}
  query-params : {String -> String}
  body         : String)

(defstruct http-response
  status-code  : Nat
  status-text  : String
  headers      : {String -> String}
  body         : String)

(defstruct route
  method  : HttpMethod
  path    : String
  handler : HttpRequest -> HttpResponse)
```

## Resolver Natives

```innate
@server{name: String, port: Nat}                    -> WebServer
@server/route{method: HttpMethod, path: String, handler: Handler}  -> Bool
@server/static{path: String, content: String, type: String}        -> Bool
@server/handle{request: HttpRequest}                 -> HttpResponse
@server/process-raw{raw: String}                     -> String
@server/access-log                                    -> [AccessEntry]
```

## Demo

```innate
(@demo){
  @server <- @web-server{name: "EM Server"}
  @server/route{method: :get, path: "/", handler: -> @response-200{"<h1>EM</h1>"}}
  @server/route{method: :get, path: "/health", handler: -> @response-json{"{\"status\":\"ok\"}"}}
  @resp <- @server/handle{request: {method: :get, path: "/"}}
  @print{@resp.status-code}  ;; => 200
  @print{@resp.body}          ;; => <h1>EM</h1>
}
```
