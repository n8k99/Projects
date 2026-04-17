# G036 — Fetch Current Weather

> HTTP client for live weather data from external APIs.

```yaml
id: G036
title: Fetch Current Weather
category: networking
requires: [G012-distance-cities, G033-ftp-protocol, G034-atomic-time]
provides: [http-client, api-consumption, fact-freshness, frame-translation-live]
```

## Insight: The Resolver Reaches the Internet

G012 (Distance Between Two Cities) introduced the **fact store** — world-state retrieved from a database. But the city table was local and static. Weather is the first **live external fact**: a value that changes continuously, that requires an HTTP request to a remote authority, and that expires the moment it's received.

This is the resolver's first encounter with **fact freshness as a first-class concern**. `@weather{city: "Jacksonville"}` doesn't return a cached constant — it returns a snapshot of the atmosphere, valid for minutes at best. The resolver must treat the result as ephemeral: useful now, stale soon, gone by tomorrow.

## Insight: HTTP Is FTP Generalized

G033's FTP gave us the bilateral request-response protocol: client sends a command, server returns data. HTTP is the same pattern with a universal addressing scheme (URLs instead of file paths) and richer metadata (headers, status codes, content types). The `GET /data/2.5/weather?q=Jacksonville` request is `RETR weather-jacksonville` with extra steps.

But HTTP adds something FTP didn't: **the response is structured data** (JSON), not a raw file. The client must parse the response into typed fields. This is G022's RSS serialization pattern (structured content across a boundary) applied to an API response. The resolver needs HTTP + JSON parsing as a combined primitive: fetch and structure in one operation.

## Insight: API Keys Are Shared Secrets

G028 (Ciphers) introduced shared secrets between two parties. The API key is the same pattern applied to trust between a client and a service: the key proves identity, gates access, and tracks usage. Without it, the server refuses the request. This is G013's progressive trust model applied to API consumption: the key is the cheapest `<-` gate (does this client have permission?), and the server's response is the expensive operation it protects.

## Insight: Unit Conversion Returns as Frame Translation

G010 discovered that agents think in frames and choreographies negotiate at boundaries. Weather data arrives in Kelvin (the API's canonical frame). Nathan thinks in Fahrenheit. European agents think in Celsius. The `@convert` pattern from G010 is embedded in the weather report: every temperature is stored in the canonical frame (Kelvin) and projected into the consumer's frame on demand.

This is the normalize-to-base pattern confirmed in a live context. The API is the authority. Kelvin is the base unit. Every consumer gets a frame-translated view.

## Insight: Comparison Is Concurrent Fact Gathering

`compare_weather([Jacksonville, London, Tokyo])` requires three independent API calls. Each returns a snapshot. The comparison aggregates snapshots into a composite view: warmest, coldest, most humid, windiest. This is G012's proximity-based coordination pattern: gather facts from multiple sources concurrently, compute derived metrics in parallel, aggregate into a decision.

The `where` for a weather-aware choreography: Kathryn checks weather before scheduling an outdoor event. The weather report isn't the decision — it's a fact that informs the judgment. The choreography coordinates the fact-gathering, and the `where` scores the aggregate.

## Choreographic Case: Weather-Aware Operations

```innate
(@weather-check){
  concurrent {
    @jax_weather <- @weather{city: "Jacksonville", country: "US"}
    @london_weather <- @weather{city: "London", country: "GB"}
  }
  join {
    @comparison <- @compare{reports: [@jax_weather, @london_weather]}
  }
  where {
    both_reports_fresh: @jax_weather.age < 15min AND @london_weather.age < 15min
    // The where doesn't judge the weather. It judges the freshness.
    // Stale weather is worse than bad weather — at least bad weather is true.
  }
}
```

The choreography doesn't compute weather. It **coordinates the gathering** of weather from independent sources into a single coherent view. The `concurrent` block parallelizes the API calls. The `join` synchronizes. The `where` validates freshness — because a weather report from an hour ago is a fact about the past, not a fact about now.

## Structures

```innate
(defstruct wind-info
  speed-mps     : Float
  direction-deg : Float
  gust-mps      : Float?)

(defstruct weather-condition
  id          : Nat
  main        : String     ;; "Rain", "Clear", "Clouds"
  description : String     ;; "light rain", "clear sky"
  icon        : String)

(defstruct weather-report
  city          : String
  country       : String
  latitude      : Float
  longitude     : Float
  temperature-k : Float       ;; canonical unit
  feels-like-k  : Float
  humidity      : Nat         ;; percent
  pressure-hpa  : Nat
  visibility-m  : Nat
  conditions    : [WeatherCondition]
  wind          : WindInfo
  clouds-pct    : Nat
  timestamp     : Instant
  sunrise       : Instant?
  sunset        : Instant?)
```

## Resolver Natives

```innate
@weather{city: String, country: String?}  -> WeatherReport
@weather{lat: Float, lon: Float}          -> WeatherReport
@weather{zip: String, country: String?}   -> WeatherReport
@compare{reports: [WeatherReport]}        -> WeatherComparison
```

The `@weather` native is a **live fact lookup** — G012's `@city` pattern extended to a remote API with freshness semantics. The resolver caches the result with a TTL (G026's temporal scope). Subsequent requests within the TTL return the cached snapshot. After expiry, the resolver re-fetches.

## Frame Projections

```innate
@weather{city: "Jacksonville"}.temperature-k     ;; 300.0 (canonical)
@weather{city: "Jacksonville"}.temperature-c      ;; 26.85 (Celsius projection)
@weather{city: "Jacksonville"}.temperature-f      ;; 80.33 (Fahrenheit projection)
@weather{city: "Jacksonville"}.wind.cardinal       ;; "S" (compass projection)
```

Same data, different frames. The canonical form (Kelvin, degrees, m/s) is what the API returns. The projections are what agents consume. The report doesn't change — the view does.

## Demo

```innate
(@demo){
  @client <- @weather-client{api-key: @env{OWM_API_KEY}}
  @report <- @client/by-city{city: "Jacksonville", country: "US"}
  @print{@report.summary}
  ;; => "Jacksonville, US: 26.8°C (80.3°F), clear sky, humidity 65%, wind 3.5 m/s S"
}
```
