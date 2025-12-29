# Weather API Upgrade Architecture

**Issue**: [#32 - Investigate Alternate Weather API's](https://github.com/nickfang/vestaboard-local/issues/32)
**Date**: 2025-12-28
**Status**: Draft - Awaiting Decision on Open Questions

---

## Session State (for continuation)

### What Has Been Done
- [x] Fetched and analyzed GitHub issue #32
- [x] Explored full codebase architecture (widgets, error handling, config patterns)
- [x] Read current weather widget implementation (`src/widgets/weather/weather.rs`)
- [x] Researched NWS API (endpoints, auth, data format, precipitation fields)
- [x] Researched AccuWeather API (endpoints, auth, rate limits)
- [x] Created this architecture document with recommendation

### Key Research Findings
1. **Current implementation**: Uses weatherapi.com, hardcoded to Austin, TX, displays temp/rain/pressure
2. **NWS API**: Free, no API key, US-only, two-step lookup (points -> forecast), has `probabilityOfPrecipitation`
3. **AccuWeather**: 50 calls/day free tier (too limiting for daemon), requires API key
4. **Recommendation**: NWS API (free, authoritative source, fits US-only use case)

### Codebase Patterns to Follow
- Error handling: Use `VestaboardError` enum with factory methods (`api_error()`, `reqwest_error()`, etc.)
- Config: Environment variables for secrets/location (via `dotenv`)
- Widget interface: `async fn get_weather() -> Result<WidgetOutput, VestaboardError>`
- Logging: Use `log::info!`, `log::debug!`, `log::error!` macros
- HTTP: Use `reqwest::Client` with proper error context

### Next Steps
1. **Get answers to open questions** (see below)
2. **Implement NWS API client** in `src/widgets/weather/weather.rs`
3. **Update `.env.example`** - remove `WEATHER_API_KEY`, add `WEATHER_LOCATION`
4. **Test manually** against weather.gov
5. **Add unit tests** with mock responses

### Open Questions Requiring User Input
1. **User-Agent contact email**: What email for `User-Agent: (vestaboard-local, ???)`?
2. **Pressure display**: Remove barometric pressure (NWS doesn't provide in basic forecast) or add observation station call?
3. **Location config**: Add `WEATHER_LOCATION` env var or keep Austin hardcoded?

### Key Files
| File | Purpose |
|------|---------|
| `src/widgets/weather/weather.rs` | Main file to modify |
| `src/errors.rs` | Error types (no changes needed) |
| `src/widgets/resolver.rs` | Widget dispatcher (no changes needed) |
| `src/config.rs` | Config patterns reference |

---

## Problem Statement

The current weather widget uses [WeatherAPI.com](https://weatherapi.com) but rain predictions appear inaccurate. The issue suggests investigating alternative APIs:
- National Weather Service (NWS) API
- AccuWeather API

## Current Implementation Analysis

**File**: `src/widgets/weather/weather.rs`

### Current Flow
1. Load `WEATHER_API_KEY` from environment
2. Make HTTP request to WeatherAPI.com forecast endpoint
3. Parse JSON response into strongly-typed structs
4. Format output for 6-line Vestaboard display
5. Handle errors with appropriate user-facing messages

### Data Currently Displayed
- Local time
- Current temp (W), daily min (B), daily max (R)
- Weather condition text
- Chance of rain (%) and expected precipitation amount
- Barometric pressure (current and forecast)

### Current Limitations
- **Hardcoded location**: "austin" is embedded in URL
- **Single API provider**: No fallback if service unavailable
- **Reported inaccuracy**: Rain predictions not matching reality

## API Comparison

| Feature | WeatherAPI.com (Current) | NWS API | AccuWeather |
|---------|-------------------------|---------|-------------|
| **Cost** | Free tier available | Free | Free tier (50 calls/day) |
| **API Key** | Required | Not required* | Required |
| **Coverage** | Global | US only | Global |
| **Rate Limits** | 1M calls/month (free) | Undisclosed, generous | 50/day (free) |
| **Precipitation Data** | `daily_chance_of_rain`, `totalprecip_in` | `probabilityOfPrecipitation`, detailed forecasts | `HasPrecipitation`, `PrecipitationType` |
| **Location Lookup** | Direct (city name, zip, coords) | Two-step (coords -> grid -> forecast) | Location key required |

*NWS requires a User-Agent header with contact info

### NWS API Details

**Pros**:
- Free with no API key management
- Official US government source (authoritative data)
- Detailed precipitation probability per forecast period
- Hourly and 12-hour period forecasts available
- Grid-based forecasts (~2.5km resolution)

**Cons**:
- US-only coverage
- Two-step location lookup required:
  1. `/points/{lat},{lon}` -> returns grid coordinates and forecast URLs
  2. `/gridpoints/{office}/{gridX},{gridY}/forecast` -> returns forecast
- Occasional grid changes (should cache but verify periodically)
- Response format is more verbose (GeoJSON)

**Authentication**: User-Agent header required
```
User-Agent: (vestaboard-local, your-email@example.com)
```

### AccuWeather API Details

**Pros**:
- Global coverage
- Structured precipitation data
- Well-documented

**Cons**:
- 50 calls/day on free tier (severely limiting for a daemon running every 3 seconds)
- Requires location key lookup before getting weather
- Cost for higher tiers

## Recommendation

**Use the National Weather Service (NWS) API as the primary provider.**

Rationale:
1. **Accuracy**: Official government source, likely more accurate for US locations
2. **Cost**: Completely free with generous rate limits
3. **Simplicity**: No API key to manage (just User-Agent header)
4. **Use Case Fit**: This is a personal Vestaboard project for Austin, TX (US location)

The main tradeoff is US-only coverage, but since the current implementation is already hardcoded to Austin, TX, this isn't a practical limitation.

## Implementation Approach

### Phase 1: Replace WeatherAPI.com with NWS (Minimal Change)

Keep the same widget interface and display format. Only change the data source.

#### Files to Modify

| File | Changes |
|------|---------|
| `src/widgets/weather/weather.rs` | Replace API client, response structs, and parsing logic |
| `.env` / `.env.example` | Remove `WEATHER_API_KEY`, add optional `WEATHER_LOCATION` |
| `src/errors.rs` | No changes needed (existing error types sufficient) |

#### New Response Structs

Replace existing `WeatherResponse`, `Current`, `Forecast`, etc. with NWS-specific types:

```rust
// Points endpoint response (for grid lookup)
struct PointsResponse {
    properties: PointsProperties,
}
struct PointsProperties {
    forecast: String,           // URL for forecast
    forecast_hourly: String,    // URL for hourly forecast
    grid_id: String,            // e.g., "EWX"
    grid_x: i32,
    grid_y: i32,
}

// Forecast endpoint response
struct ForecastResponse {
    properties: ForecastProperties,
}
struct ForecastProperties {
    periods: Vec<ForecastPeriod>,
}
struct ForecastPeriod {
    number: i32,
    name: String,               // e.g., "Tonight", "Saturday"
    temperature: i32,
    temperature_unit: String,   // "F" or "C"
    probability_of_precipitation: Option<PrecipValue>,
    short_forecast: String,     // e.g., "Partly Cloudy"
    detailed_forecast: String,
}
struct PrecipValue {
    value: Option<i32>,
    unit_code: String,
}
```

#### Location Configuration

Options (in order of preference):
1. **Environment variable**: `WEATHER_LOCATION=30.2672,-97.7431` (lat,lon for Austin)
2. **Config file**: Add `weather_location` to `vblconfig.toml`
3. **Keep hardcoded**: If location rarely changes, simplest is to keep it in code

**Recommendation**: Use environment variable for now. Matches existing pattern (`WEATHER_API_KEY`) and avoids config file changes.

#### HTTP Client Changes

```rust
// Current
let url = format!(
    "https://api.weatherapi.com/v1/forecast.json?key={}&q=austin&days=3",
    weather_api_key
);
let response = client.get(&url).send().await?;

// New (NWS)
const USER_AGENT: &str = "(vestaboard-local, contact@example.com)";
let coords = env::var("WEATHER_LOCATION").unwrap_or_else(|_| "30.2672,-97.7431".to_string());
let points_url = format!("https://api.weather.gov/points/{}", coords);

let points_response = client
    .get(&points_url)
    .header("User-Agent", USER_AGENT)
    .send()
    .await?;
```

#### Caching Strategy

The NWS documentation recommends caching grid lookups. For simplicity in Phase 1:
- Cache the forecast URL in memory (static or lazy)
- Re-fetch on error (handles grid changes)
- No persistent cache needed

```rust
static FORECAST_URL: OnceCell<String> = OnceCell::new();
```

#### Error Handling

Map NWS-specific errors to existing `VestaboardError` types:

| NWS Response | Error Type |
|--------------|------------|
| Network failure | `ReqwestError` with context "requesting NWS forecast" |
| 404 on points | `ApiError` with "Location not found" |
| 503 Service Unavailable | `ApiError` with "Weather service temporarily unavailable" |
| Parse failure | `JsonError` with context |

Update `to_user_message()` in `errors.rs` if NWS-specific messages needed (likely not required).

#### Display Format

Keep the current 6-line format but adapt to NWS data:

```
Line 1: Current date/time (from system, NWS doesn't provide "now")
Line 2: Temperatures (current period, low, high from periods)
Line 3-5: Forecast summary + precipitation chance
Line 6: (TBD - pressure not in basic NWS forecast)
```

**Note**: NWS basic forecast doesn't include barometric pressure. Options:
- Remove pressure line (simplifies display)
- Use observation station data (additional API call)
- Display extended forecast instead

**Recommendation**: Remove pressure, show forecast for next periods.

### Testing Strategy

1. **Manual testing**: Verify output against weather.gov website
2. **Unit tests**: Add tests with mock NWS responses (test parsing)
3. **Integration test**: Optional, verify API connectivity

### Migration Path

1. Add NWS implementation alongside existing code (feature flag or separate function)
2. Test NWS implementation
3. Remove WeatherAPI.com code and `WEATHER_API_KEY` dependency
4. Update documentation

## Out of Scope

The following are explicitly excluded from this change:
- Configurable location in UI/CLI (would require new command args)
- Multiple weather providers with fallback
- Weather alerts
- Extended forecasts beyond current display
- Caching to disk/persistent storage
- Non-US location support

## Open Questions

1. **User-Agent contact email**: What email should be used in the User-Agent header?
2. **Pressure display**: Keep or remove barometric pressure from display?
3. **Location format**: Stay with hardcoded Austin or add env var for configurability?

## References

- [NWS API Documentation](https://www.weather.gov/documentation/services-web-api)
- [NWS API OpenAPI Spec](https://api.weather.gov/openapi.json)
- [Current weather.rs implementation](../src/widgets/weather/weather.rs)
