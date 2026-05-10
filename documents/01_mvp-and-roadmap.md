# SwingBot — Vision, Architecture & Full Roadmap

> This document is the authoritative build plan. `00_initial-plan.md` is kept for
> historical reference only.

---

## Vision

SwingBot is an **autonomous, multi-asset trading intelligence system**. It continuously
monitors financial markets across stocks, crypto, rare earth metals, REITs/housing, and
commodities. It ingests market data, runs technical analysis, monitors global events
(news, social media, macroeconomics, prediction markets), scores opportunities, and
notifies the user via Telegram — enabling fast, well-informed, risk-controlled trading
decisions with manual execution.

**The bot is the brain. It never sleeps.**

---

## Core Principles

- **Autonomous scanning.** The bot actively pulls data and finds opportunities. No
  external signal generator (e.g. TradingView) is required or assumed.
- **Multi-asset from day one.** Architecture handles stocks, crypto, ETFs, REITs, and
  commodities uniformly.
- **All data sources.** Price/volume, news sentiment, social media signals, macroeconomic
  events, and prediction markets are all first-class inputs.
- **Risk first.** Every accepted signal passes through the risk engine. Position sizing,
  exposure caps, and regime filters are not optional.
- **Paper mode from the first Telegram milestone.** Never debug with real money.
- **Alternative data is the primary alpha source.** The TA layer finds setups; options
  flow, insider activity, news sentiment, and LLM synthesis are what generate outsized
  edge that cannot be easily arbitraged away by other systematic traders.
- **Event-driven moves are where large returns live.** Earnings gaps, analyst upgrades,
  unusual options activity, and macro catalysts create the non-linear price moves that
  compound wealth. TA strategies find entries; catalysts determine magnitude.
- **Validate early, build late.** Run a simplified backtest on the TA engine before
  building the full scanner and Telegram infrastructure. Months of engineering on an
  unvalidated strategy is the most common way to waste time.
- **Security baked in everywhere.** Rate limiting, secrets management, input validation,
  and audit logging are part of every milestone, not an afterthought.
- **Production quality from commit one.** Structured logging, tests, CI, graceful
  shutdown, Docker.

---

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                        DATA INGESTION LAYER                      │
│                                                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌──────────────────────────┐ │
│  │ Market Data │  │    News     │  │     Social / Alt Data    │ │
│  │  Provider   │  │  Ingester   │  │  (Twitter/X, Reddit,     │ │
│  │ (OHLCV,     │  │ (NewsAPI,   │  │   StockTwits, Polymarket)│ │
│  │  tick data) │  │  FMP, EDGAR)│  │                          │ │
│  └──────┬──────┘  └──────┬──────┘  └────────────┬─────────────┘ │
│         │                │                       │               │
│  ┌──────▼──────┐  ┌──────▼──────┐  ┌────────────▼─────────────┐ │
│  │ Candle Store│  │  News Store │  │     Social Event Store   │ │
│  │ (PostgreSQL)│  │ (PostgreSQL)│  │      (PostgreSQL)        │ │
│  └──────┬──────┘  └──────┬──────┘  └────────────┬─────────────┘ │
└─────────┼────────────────┼─────────────────────-┼───────────────┘
          │                │                       │
┌─────────▼────────────────▼───────────────────────▼───────────────┐
│                        ANALYSIS LAYER                             │
│                                                                   │
│  ┌──────────────────┐   ┌──────────────────┐                      │
│  │  Technical       │   │  Sentiment &     │                      │
│  │  Analysis Engine │   │  Event Scorer    │                      │
│  │  (indicators,    │   │  (news, social,  │                      │
│  │  patterns, MTF)  │   │  macro, PM)      │                      │
│  └────────┬─────────┘   └────────┬─────────┘                      │
│           │                      │                                │
│  ┌────────▼──────────────────────▼─────────┐                      │
│  │         Signal Aggregator + Scorer      │                      │
│  │  (multi-source confluence, final score) │                      │
│  └────────────────────┬────────────────────┘                      │
└───────────────────────┼───────────────────────────────────────────┘
                        │
┌───────────────────────▼───────────────────────────────────────────┐
│                        DECISION LAYER                             │
│                                                                   │
│  ┌─────────────────┐   ┌─────────────────┐                        │
│  │  Market Regime  │   │   Risk Engine   │                        │
│  │  Filter         │   │  (sizing, caps, │                        │
│  │  (bull/bear/    │   │   exposure)     │                        │
│  │   sideways)     │   │                 │                        │
│  └────────┬────────┘   └────────┬────────┘                        │
└───────────┼─────────────────────┼──────────────────────────────────┘
            └────────────┬────────┘
┌───────────────────────▼───────────────────────────────────────────┐
│                      NOTIFICATION LAYER                           │
│                                                                   │
│            Telegram Bot (teloxide, webhook mode)                  │
│            Accept / Reject / Execute / Close                      │
└───────────────────────┬───────────────────────────────────────────┘
                        │
┌───────────────────────▼───────────────────────────────────────────┐
│                       TRACKING LAYER                              │
│                                                                   │
│   Trade Tracker · Open Trade Monitor · P&L · Audit Log           │
└───────────────────────────────────────────────────────────────────┘
```

---

## Technology Stack

| Layer | Crate / Tool |
|---|---|
| HTTP server | `axum` + `tower` + `tower-http` |
| Per-IP rate limiting | `tower_governor` (GCRA, keyed — **not** `tower::limit::RateLimitLayer`) |
| Async runtime | `tokio` (full features) |
| Database | PostgreSQL 16 via `sqlx` (compile-time queries, `sqlx-cli` migrations) |
| Telegram | `teloxide` (webhook mode — no long polling) |
| Serialization | `serde` + `serde_json` |
| Decimal arithmetic | `rust_decimal` (all prices/quantities — never `f64`) |
| IDs | `uuid` (v4) |
| Time | `chrono` (all timestamps UTC/TIMESTAMPTZ) |
| HTTP client | `reqwest` (for market data, news, social APIs) |
| Error handling | `anyhow` (application) + `thiserror` (library errors) |
| Logging | `tracing` + `tracing-subscriber` (structured JSON, env-filter) |
| Config | `dotenvy` |
| Concurrency | `dashmap` (in-memory conversation state) |
| CI | GitHub Actions: `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `cargo deny` |

---

## Asset Classes

The system treats all assets uniformly through the `Symbol` type. Asset class is a
metadata tag that influences which data providers are queried and which strategies apply.

| Class | Examples |
|---|---|
| US Stocks | AAPL, NVDA, TSLA, individual equities |
| US ETFs | SPY, QQQ, ARKK, sector ETFs |
| Crypto | BTC-USD, ETH-USD, SOL-USD (via exchange APIs) |
| Rare Earth Metals | REMX (ETF), MP (MP Materials), LYSDY, ALTHF |
| Housing / REITs | VNQ, IYR, AVB, EQR, PLD, homebuilder stocks (DHI, LEN) |
| Commodities | GLD, SLV, USO, PDBC (via ETFs) |
| Broad Indices | SPY, QQQ, IWM, DIA (for regime detection) |

---

## Data Sources

### Market Data (OHLCV)
- **Primary:** Polygon.io — US stocks, ETFs, indices, crypto, forex; REST + WebSocket;
  free tier supports end-of-day and 2-year-delayed intraday only. **Intraday timeframes
  (4H, 1H, 15M, 5M) require a paid Polygon tier (Starter ~$29/month).** Budget this
  from M4 onward; the free tier is sufficient for daily-only scanning during development.
- **Crypto supplement:** CoinGecko (free) or Binance API (free) for crypto OHLCV.
  Prefer Binance for intraday — CoinGecko's `/coins/{id}/ohlc` endpoint is limited to
  30-day history on the free tier and has coarse granularity.
- **Fallback / secondary:** Alpha Vantage — stocks, crypto, forex, economic indicators.

### News
- **NewsAPI.org** — broad financial news coverage, free tier.
- **Financial Modeling Prep (FMP)** — stock-specific news with sentiment scores.
- **SEC EDGAR** — earnings releases, 8-K filings, insider activity.
- **Alpha Vantage News & Sentiment** — AI-tagged financial news.

### Social Media / Social Signals
- **Twitter/X API v2** — targeted accounts (heads of state, central bankers, major
  financial personalities, company CEOs), trending hashtags, keyword monitoring.
  **Note: filtered stream access requires a paid plan (~$100/month Basic tier as of
  2025). Evaluate cost before building M-V2-03 Layer A; Reddit + StockTwits are free
  and can substitute as the initial social signal layer.**
- **Reddit API** — r/wallstreetbets, r/investing, r/stocks, r/cryptocurrency; post
  velocity and sentiment scoring.
- **StockTwits API** — stock-specific social sentiment (bullish/bearish tags).

### Macroeconomic / Calendar
- **FRED (Federal Reserve Economic Data)** — CPI, unemployment, GDP, interest rates.
- **Trading Economics API** — economic calendar with impact ratings.
- **Alpha Vantage Economic Indicators** — earnings calendar, IPO calendar.

### Prediction Markets
- **Polymarket REST API** — event markets relevant to macro/political outcomes.

---

## Technical Analysis Indicators

All indicators are implemented as pure functions operating on `Vec<Candle>`. No floating
point — `rust_decimal` throughout.

### Trend
| Indicator | Params |
|---|---|
| SMA | 20, 50, 100, 200 |
| EMA | 9, 12, 20, 26, 50, 200 |
| MACD | 12/26/9 — line, signal, histogram |
| ADX + DI+ / DI− | 14 |
| Parabolic SAR | 0.02 step, 0.20 max |
| Supertrend | ATR multiplier configurable |
| Ichimoku Cloud | Tenkan 9, Kijun 26, Senkou B 52, Chikou 26 |

### Momentum
| Indicator | Params |
|---|---|
| RSI | 14 |
| Stochastic | %K 14, %D 3, smooth 3 |
| CCI | 20 |
| Williams %R | 14 |
| Rate of Change (ROC) | 10 |
| Awesome Oscillator | 5/34 SMA of midpoints |

### Volatility
| Indicator | Params |
|---|---|
| ATR | 14 |
| Bollinger Bands | 20, 2σ; also 1σ and 3σ bands |
| Keltner Channels | EMA 20, ATR 10, multiplier 2 |
| Historical Volatility | 20-period rolling |

### Volume
| Indicator | Params |
|---|---|
| OBV | — |
| VWAP | Intraday reset at session open |
| CMF (Chaikin Money Flow) | 20 |
| MFI (Money Flow Index) | 14 |
| Accumulation/Distribution | — |
| Volume SMA | 20 |

### Support / Resistance
- Pivot points (Classic, Fibonacci, Camarilla) — daily and weekly
- Fibonacci retracements: 23.6%, 38.2%, 50%, 61.8%, 78.6%
- Prior day / week high and low
- Round-number levels (configurable step)

### Candlestick Patterns
Single-candle: Hammer, Hanging Man, Shooting Star, Inverted Hammer, Doji (standard,
dragonfly, gravestone), Marubozu.

Multi-candle: Bullish/Bearish Engulfing, Morning Star, Evening Star, Three White
Soldiers, Three Black Crows, Harami, Tweezer Top/Bottom, Pin Bar.

---

## Signal Scoring Model

Each candidate signal is assigned a composite score (0–100). Weights are configurable
per strategy.

> **MVP weights (TA-only data available):** The weights below are the starting point.
> As V2 data sources come online, rebalance using the backtesting gate as the empirical
> validator. Target allocations once fully live: options flow / institutional → 10,
> LLM verdict → 8. Reduce TA weights proportionally — guided by what the backtest
> shows actually drives edge. Total always sums to 100.

| Category | MVP Weight | Full Weight (V2+) | Sources |
|---|---|---|---|
| Trend alignment (HTF) | 25 | 20 | Weekly/daily EMA stack, ADX, Ichimoku |
| Setup quality (MTF) | 20 | 15 | 4H/1H pullback to key level, RSI zone, Stochastic |
| Entry confirmation (LTF) | 15 | 10 | Candlestick pattern, volume, MACD cross |
| Risk/Reward | 15 | 15 | ≥ 3:1 required; higher R/R scores better |
| News sentiment | 10 | 10 | Positive/negative news in last 24h for asset |
| Options flow / institutional | 0 → **10** | 10 | Unusual options OI, insider buys (V2-02) |
| LLM signal verdict | 0 → **8** | 8 | Claude synthesis of news + social + TA context (V2-05) |
| Social sentiment | 5 | 5 | StockTwits bullish ratio, Reddit mention velocity |
| Macro / regime | 5 | 4 | Market regime score, economic calendar clearance |
| Prediction market | 5 | 3 | Relevant Polymarket event probability |

Minimum score threshold to trigger a Telegram notification is configurable via env
(`SCORE_THRESHOLD`, default 60).

---

---

# Part 1 — MVP

**MVP definition:** The bot autonomously scans a configured watchlist, runs technical
analysis across multiple timeframes, scores setups, and notifies the user via Telegram.
The user accepts/rejects, manually executes in Trade Republic (or any broker), and the
bot tracks the trade. Paper mode is always available.

**MVP does NOT include:** news/social/prediction-market ingestion (V2), backtesting (V2),
trade monitoring (V2), chart pattern detection (V2), market regime filter (V2).

Build order: M1 → M2 → M3 → M4 → M5 → M6 → M7 → M8 → M9 → M10

---

## M1 — Project Skeleton

### Todos

- [ ] `cargo new swingbot --bin`
- [ ] Add dependencies:
  - [ ] `axum`, `tower`, `tower-http` (features: `trace`, `request-id`, `limit`, `cors`)
  - [ ] `tower_governor` (per-IP GCRA rate limiting — **not** `tower::limit::RateLimitLayer`)
  - [ ] `tokio` (full features)
  - [ ] `sqlx` (features: postgres, runtime-tokio, rust_decimal, uuid, chrono, macros)
  - [ ] `teloxide` (features: webhooks, macros)
  - [ ] `reqwest` (features: json, rustls-tls)
  - [ ] `serde`, `serde_json`
  - [ ] `uuid` (features: v4, serde)
  - [ ] `chrono` (features: serde)
  - [ ] `rust_decimal` (features: serde-with-str)
  - [ ] `anyhow`, `thiserror`
  - [ ] `tracing`, `tracing-subscriber` (features: env-filter, json)
  - [ ] `dotenvy`
  - [ ] `dashmap`
  - [ ] `tokio-util` (for graceful shutdown)
- [ ] Module layout:
  - [ ] `main.rs` — wiring, `AppState`, graceful shutdown
  - [ ] `config.rs` — env loading, typed `Config` struct, validation on startup
  - [ ] `db.rs` — pool construction, migration runner
  - [ ] `models/mod.rs` — domain types: `Symbol`, `AssetClass`, `Candle`, `Signal`, `Trade`,
        `Side`, `SignalStatus`, `TradeStatus`, `TradingMode`
  - [ ] `routes/mod.rs`, `routes/health.rs`, `routes/telegram.rs`
  - [ ] `telegram/mod.rs` — bot client, `TelegramClient` trait, message formatting
  - [ ] `market_data/mod.rs` — `MarketDataProvider` trait + provider registry
  - [ ] `ta/mod.rs` — technical analysis engine (empty skeleton)
  - [ ] `scanner/mod.rs` — scanner task (empty skeleton)
  - [ ] `risk/mod.rs` — risk engine (empty skeleton)
  - [ ] `monitor/mod.rs` — trade monitor task (empty skeleton)
  - [ ] `error.rs` — `AppError` with `IntoResponse` impl
- [ ] `.env.example` with every variable documented and grouped by concern
- [ ] `README.md` — one-command local-run instructions
- [ ] Structured logging: `tracing-subscriber` with JSON output, gated by `RUST_LOG`
- [ ] Graceful shutdown: join `ctrl_c` AND `SIGTERM` (Docker sends SIGTERM on `docker stop`)
- [ ] CI workflow (GitHub Actions):
  - [ ] `cargo fmt --check`
  - [ ] `cargo clippy -- -D warnings`
  - [ ] `cargo test`
  - [ ] `cargo deny check` (vulnerability audit, license check, duplicate deps)

### Testing

- [ ] `cargo check`, `cargo clippy`, `cargo test` all pass
- [ ] App boots, `GET /health` returns `200 OK` with `{"status":"ok"}`
- [ ] Ctrl-C (SIGINT) shuts down cleanly within 5s
- [ ] `docker stop` (SIGTERM) shuts down cleanly within 5s
- [ ] CI green on push

---

## M2 — Database + Migrations

### Todos

- [ ] `docker-compose.yml` with Postgres 16 and a named data volume
- [ ] `sqlx-cli` for migrations — committed to repo under `migrations/`
- [ ] `db.rs`: pool construction; run pending migrations on startup
- [ ] Single `PgPool` shared across all handlers and background tasks via `Arc<AppState>`
- [ ] Schema:

```sql
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Asset watchlist
CREATE TABLE watchlist (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    symbol      TEXT NOT NULL UNIQUE,
    asset_class TEXT NOT NULL CHECK (asset_class IN (
                    'stock','etf','crypto','reit','commodity','index')),
    enabled     BOOLEAN NOT NULL DEFAULT true,
    note        TEXT,
    added_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- OHLCV candle storage (one row per symbol + timeframe + timestamp)
CREATE TABLE candles (
    id          BIGSERIAL PRIMARY KEY,
    symbol      TEXT NOT NULL,
    timeframe   TEXT NOT NULL,             -- '1d', '4h', '1h', '15m', '5m'
    ts          TIMESTAMPTZ NOT NULL,
    open        NUMERIC NOT NULL,
    high        NUMERIC NOT NULL,
    low         NUMERIC NOT NULL,
    close       NUMERIC NOT NULL,
    volume      NUMERIC NOT NULL,
    CONSTRAINT candles_uniq UNIQUE (symbol, timeframe, ts)
);
CREATE INDEX candles_symbol_tf_ts_idx ON candles (symbol, timeframe, ts DESC);

-- Signals generated by the scanner
CREATE TABLE signals (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    symbol          TEXT NOT NULL,
    asset_class     TEXT NOT NULL,
    side            TEXT NOT NULL CHECK (side IN ('long','short')),
    entry_price     NUMERIC NOT NULL CHECK (entry_price > 0),
    stop_loss       NUMERIC NOT NULL CHECK (stop_loss > 0),
    take_profit     NUMERIC NOT NULL CHECK (take_profit > 0),
    risk_reward     NUMERIC NOT NULL,
    score           INTEGER NOT NULL CHECK (score BETWEEN 0 AND 100),
    score_breakdown JSONB NOT NULL DEFAULT '{}'::jsonb,
    strategy        TEXT NOT NULL DEFAULT 'default',
    timeframe       TEXT NOT NULL,
    status          TEXT NOT NULL CHECK (status IN (
                        'pending','accepted','rejected','expired')),
    telegram_msg_id BIGINT,
    raw_context     JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at      TIMESTAMPTZ NOT NULL,           -- created_at + SIGNAL_TTL_HOURS
    CONSTRAINT signals_idempotency UNIQUE (symbol, side, strategy, timeframe, created_at)
);
CREATE INDEX signals_status_idx   ON signals (status);
CREATE INDEX signals_created_idx  ON signals (created_at DESC);
CREATE INDEX signals_symbol_idx   ON signals (symbol);

-- Trades
CREATE TABLE trades (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    signal_id           UUID NOT NULL REFERENCES signals(id),
    symbol              TEXT NOT NULL,
    asset_class         TEXT NOT NULL,
    side                TEXT NOT NULL CHECK (side IN ('long','short')),
    planned_entry_price NUMERIC NOT NULL,
    actual_entry_price  NUMERIC,
    quantity            NUMERIC,
    stop_loss           NUMERIC NOT NULL,
    take_profit         NUMERIC NOT NULL,
    currency            TEXT NOT NULL DEFAULT 'EUR',
    fees                NUMERIC NOT NULL DEFAULT 0,
    realized_pnl        NUMERIC,
    status              TEXT NOT NULL CHECK (status IN (
                            'planned','open','closed_target','closed_stop',
                            'closed_manual','invalidated')),
    alerts_sent         JSONB NOT NULL DEFAULT '{}'::jsonb,
    trading_mode        TEXT NOT NULL CHECK (trading_mode IN ('paper','live_manual')),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    opened_at           TIMESTAMPTZ,
    closed_at           TIMESTAMPTZ
);
CREATE INDEX trades_status_idx ON trades (status);
CREATE INDEX trades_symbol_idx ON trades (symbol);
CREATE INDEX trades_open_idx   ON trades (status)
    WHERE status IN ('planned','open');

-- Audit log
CREATE TABLE audit_log (
    id          BIGSERIAL PRIMARY KEY,
    at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    actor       TEXT NOT NULL,      -- 'scanner','telegram','monitor','system'
    event       TEXT NOT NULL,      -- 'signal.generated','trade.accepted', etc.
    entity_id   UUID,
    payload     JSONB
);
CREATE INDEX audit_log_at_idx ON audit_log (at DESC);

-- Backtest results (one row per run; trade-level detail in backtest_trades)
CREATE TABLE backtest_runs (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    strategy            TEXT NOT NULL,
    symbols             TEXT[] NOT NULL,
    timeframe           TEXT NOT NULL,
    train_from          TIMESTAMPTZ NOT NULL,
    train_to            TIMESTAMPTZ NOT NULL,
    oos_from            TIMESTAMPTZ NOT NULL,   -- out-of-sample window start
    oos_to              TIMESTAMPTZ NOT NULL,
    fee_per_trade       NUMERIC NOT NULL,
    slippage_bps        NUMERIC NOT NULL,
    trade_count         INTEGER NOT NULL,
    win_rate            NUMERIC NOT NULL,
    avg_r               NUMERIC NOT NULL,
    expectancy          NUMERIC NOT NULL,
    profit_factor       NUMERIC NOT NULL,
    max_drawdown_pct    NUMERIC NOT NULL,
    sharpe_ratio        NUMERIC NOT NULL,
    win_rate_p_value    NUMERIC NOT NULL,       -- binomial test vs 50%
    monte_carlo_p05_dd  NUMERIC NOT NULL,       -- 5th-percentile max drawdown
    passed_gate         BOOLEAN NOT NULL,       -- true if all thresholds met
    regime_breakdown    JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE backtest_trades (
    id              BIGSERIAL PRIMARY KEY,
    run_id          UUID NOT NULL REFERENCES backtest_runs(id) ON DELETE CASCADE,
    symbol          TEXT NOT NULL,
    side            TEXT NOT NULL,
    entry_at        TIMESTAMPTZ NOT NULL,
    exit_at         TIMESTAMPTZ NOT NULL,
    entry_price     NUMERIC NOT NULL,
    exit_price      NUMERIC NOT NULL,
    stop_loss       NUMERIC NOT NULL,
    take_profit     NUMERIC NOT NULL,
    r_multiple      NUMERIC NOT NULL,   -- actual outcome in R units
    exit_reason     TEXT NOT NULL       -- 'target','stop','end_of_data'
);
CREATE INDEX backtest_trades_run_idx ON backtest_trades (run_id);
```

### Testing

- [ ] `docker compose up -d postgres` succeeds
- [ ] Migrations apply cleanly on app boot
- [ ] Insert/select for all tables works
- [ ] CHECK constraint violations fail as expected
- [ ] UNIQUE constraint on candles deduplication works
- [ ] Integration tests use `sqlx::test` macro (throwaway DB per test)

---

## M3 — Market Data Provider

### Todos

- [ ] Define `MarketDataProvider` trait:
  ```rust
  #[async_trait::async_trait]
  pub trait MarketDataProvider: Send + Sync {
      /// Fetch OHLCV candles for a symbol and timeframe.
      async fn fetch_candles(
          &self,
          symbol: &str,
          timeframe: &str,
          from: DateTime<Utc>,
          to: DateTime<Utc>,
      ) -> anyhow::Result<Vec<Candle>>;

      /// Fetch latest price (for trade monitoring).
      async fn latest_price(&self, symbol: &str) -> anyhow::Result<Decimal>;
  }
  ```
- [ ] Implement `PolygonProvider` (primary for stocks/ETFs):
  - [ ] REST client using `reqwest`
  - [ ] Aggregate candles endpoint (`/v2/aggs/ticker/{ticker}/range/...`)
  - [ ] Latest price via last trade / prev close endpoint
  - [ ] Timeframe mapping: `'1d'` → `day/1`, `'4h'` → `hour/4`, etc.
  - [ ] Parse `NUMERIC` prices — never cast to `f64`
  - [ ] Exponential backoff + retry on 429 / 5xx (max 3 retries)
  - [ ] `tokio::time::timeout` on every request (configurable, default 10s)
  - [ ] API key from env `POLYGON_API_KEY`
- [ ] Implement `CoinGeckoProvider` (crypto supplement):
  - [ ] Map `BTC-USD` → CoinGecko id `bitcoin`
  - [ ] Fetch OHLCV via `/coins/{id}/ohlc`
  - [ ] Same retry and timeout discipline
- [ ] `MockProvider` for tests: returns configurable fixed candle sequences
- [ ] In-memory LRU cache keyed by `(symbol, timeframe)` storing the latest N candles
  (configurable `CACHE_CANDLES_PER_KEY`, default 500) with a configurable TTL per
  timeframe. **Do not key on `(from, to)` — incremental fetches always have a changing
  `from`, making such a key effectively uncacheable.**
- [ ] Provider registry: `MarketDataRegistry` selects provider by `AssetClass`
- [ ] Expose `GET /ready` — returns 200 only when DB and primary provider reachable

### Testing

- [ ] `MockProvider` returns known candles; callers get deterministic results
- [ ] Timeout is enforced (mock slow server)
- [ ] Retry fires on 429, backs off, succeeds on retry
- [ ] Cache: second call within TTL makes zero HTTP requests
- [ ] Symbol routing: crypto goes to `CoinGeckoProvider`, stocks to `PolygonProvider`
- [ ] `/ready` returns 503 when Polygon is unreachable

---

## M4 — Candle Ingestion + Storage

### Todos

- [ ] Background task `CandleIngestor` spawned at startup, sharing `Arc<AppState>`
- [ ] On startup: for each enabled watchlist symbol × configured timeframes, back-fill
  candles from `now() - BACKFILL_DAYS` (env, default 365) if not already stored
- [ ] Incremental ingestion loop: every `INGEST_INTERVAL_SECS` (env, default 60s for
  intraday; daily candles refreshed once per day after market close)
  - [ ] For each symbol × timeframe: fetch candles from `last stored ts` to `now()`
  - [ ] Upsert into `candles` table (rely on UNIQUE constraint)
  - [ ] Emit `tracing::info!` for new candles stored; `tracing::warn!` for gaps
- [ ] Gap detection: after ingest, check for unexpected gaps in the series; log them
- [ ] Respect provider rate limits: configurable concurrency limit per provider
- [ ] Graceful shutdown: ingestor task listens on the shared shutdown channel
- [ ] **Self-monitoring / dead-man heartbeat:**
  - [ ] Each background task (ingestor, scanner) writes a `last_heartbeat_at` timestamp
    to a shared `AtomicI64` in `AppState` after every successful cycle
  - [ ] A lightweight `WatchdogTask` spawned at startup polls all heartbeats every
    `WATCHDOG_CHECK_SECS` (env, default 120); if any task has not reported within
    `WATCHDOG_STALE_SECS` (env, default 300), it sends a Telegram alert:
    `"⚠️ SwingBot: <task> has not run in >5 min — check logs"`
  - [ ] Watchdog itself is guarded by the health endpoint: `GET /health` returns 503 if
    any heartbeat is stale (so an external uptime monitor can also catch it)
- [ ] Telegram command `GET /watchlist` list (later wired in M8); data model ready now
- [ ] Watchlist seeding from `WATCHLIST_SEED` env var (CSV of symbols) on first boot

### Testing

- [ ] Back-fill populates `candles` for a symbol from configured start date
- [ ] Incremental ingest fetches only new candles (no duplicates in DB)
- [ ] Upsert on conflict does not corrupt existing rows
- [ ] Rate-limit concurrency cap is respected (mock provider counts parallel calls)
- [ ] Shutdown signal causes ingestor to finish current batch and exit cleanly
- [ ] Gap in provider data is logged as a warning, does not crash task
- [ ] Watchdog: if ingestor heartbeat goes stale (mock time advance), Telegram alert fires
- [ ] `GET /health` returns 503 when ingestor heartbeat is stale

---

## M5 — Technical Analysis Engine

### Todos

All indicators are pure functions: `fn indicator(candles: &[Candle], ...) -> Vec<Decimal>`
or an appropriate output type. No state, no I/O. Fully unit-testable.

- [ ] **Trend indicators:**
  - [ ] `sma(candles, period)` — simple moving average
  - [ ] `ema(candles, period)` — exponential moving average
  - [ ] `macd(candles, fast, slow, signal)` → `Vec<MacdPoint { macd, signal, histogram }>`
  - [ ] `adx(candles, period)` → `Vec<AdxPoint { adx, di_plus, di_minus }>`
  - [ ] `supertrend(candles, period, multiplier)` → `Vec<SupertrendPoint { value, bullish }>`
  - [ ] `ichimoku(candles)` → `Vec<IchimokuPoint { tenkan, kijun, senkou_a, senkou_b, chikou }>`
  - [ ] `parabolic_sar(candles, step, max)` → `Vec<Decimal>`

- [ ] **Momentum indicators:**
  - [ ] `rsi(candles, period)` → `Vec<Decimal>`
  - [ ] `stochastic(candles, k_period, d_period, smooth)` → `Vec<StochPoint { k, d }>`
  - [ ] `cci(candles, period)` → `Vec<Decimal>`
  - [ ] `williams_r(candles, period)` → `Vec<Decimal>`
  - [ ] `roc(candles, period)` → `Vec<Decimal>`
  - [ ] `awesome_oscillator(candles)` → `Vec<Decimal>`

- [ ] **Volatility indicators:**
  - [ ] `atr(candles, period)` → `Vec<Decimal>`
  - [ ] `bollinger_bands(candles, period, std_dev)` → `Vec<BBPoint { upper, middle, lower }>`
  - [ ] `keltner_channels(candles, ema_period, atr_period, mult)` → `Vec<KCPoint>`
  - [ ] `historical_volatility(candles, period)` → `Vec<Decimal>`

- [ ] **Volume indicators:**
  - [ ] `obv(candles)` → `Vec<Decimal>`
  - [ ] `vwap(candles, session_open: NaiveTime, tz: &Tz)` → `Vec<Decimal>`
        (session-reset at exchange open: 09:30 ET for US stocks/ETFs, 00:00 UTC for
        crypto; the caller passes the correct `session_open` + `tz` per asset class —
        **never reset at midnight UTC for equity instruments**)
  - [ ] `cmf(candles, period)` → `Vec<Decimal>`
  - [ ] `mfi(candles, period)` → `Vec<Decimal>`
  - [ ] `accumulation_distribution(candles)` → `Vec<Decimal>`
  - [ ] `volume_sma(candles, period)` → `Vec<Decimal>`

- [ ] **Support / Resistance:**
  - [ ] `pivot_points_classic(prev_candle)` → `PivotLevels { r3..r1, pivot, s1..s3 }`
  - [ ] `pivot_points_fibonacci(prev_candle)` → `PivotLevels`
  - [ ] `fibonacci_retracement(swing_high, swing_low)` → `FibLevels`
  - [ ] `key_levels(candles, lookback)` → `Vec<Decimal>` — significant high/low clusters

- [ ] **Candlestick pattern detection:**
  - [ ] `detect_hammer(candle)` → `bool`
  - [ ] `detect_hanging_man(candle, trend)` → `bool`
  - [ ] `detect_shooting_star(candle)` → `bool`
  - [ ] `detect_inverted_hammer(candle)` → `bool`
  - [ ] `detect_doji(candle)` → `DojiKind`
  - [ ] `detect_engulfing(prev, curr)` → `Option<BullBear>`
  - [ ] `detect_morning_star(c1, c2, c3)` → `bool`
  - [ ] `detect_evening_star(c1, c2, c3)` → `bool`
  - [ ] `detect_three_white_soldiers(c1, c2, c3)` → `bool`
  - [ ] `detect_three_black_crows(c1, c2, c3)` → `bool`
  - [ ] `detect_pin_bar(candle, context)` → `Option<BullBear>`

- [ ] `AnalysisResult` struct: all indicator values for one symbol + timeframe, ready
  for scoring
- [ ] `analyse(candles: &[Candle]) -> AnalysisResult` — runs all indicators in one call

### Testing

Every indicator has:
- [ ] Known input → known output test (hand-computed or cross-validated against
  a reference implementation)
- [ ] Panics on empty input are replaced with `Result::Err` — no unwrap in lib code
- [ ] Minimum-length candle sequences (e.g. period - 1) return partial/empty vecs,
  not panics
- [ ] Candlestick patterns: explicit tests for both true-positive and true-negative cases

---

## M5.5 — Early Backtest Validation (Quick Go/No-Go Gate)

> **Run this before building the full scanner (M6).** The TA engine now exists as pure
> functions. A simplified backtest on 3 years of daily candles across 15+ symbols takes
> hours to write and minutes to run — and either validates that the strategies have real
> edge, or saves months of building the wrong thing on top of a weak signal.

### Why here

M6.5 (the rigorous gate) re-uses the full scanner's signal generation logic. But that
logic is not complex — M5's TA functions are 90% of it. M5.5 writes a thin simulation
loop directly over the TA engine. If no edge appears here, the strategies need redesign
before adding Telegram, risk engine, and deployment infrastructure on top of them.

### Todos

- [ ] `QuickBacktester::run(symbols, candles_by_symbol, config)`:
  - Walk forward bar-by-bar on 3 years of daily candles — no look-ahead
  - Apply `EmaPullbackStrategy` and `BreakoutStrategy` signal logic directly over M5 TA functions
  - Entry at bar N+1 open + configured slippage; check intra-bar stop/target on each subsequent bar
  - Apply flat €1 fee per trade
  - Track portfolio state: open trades, cumulative P&L
  - Print: trade count, win rate, avg R, profit factor
- [ ] No DB required — load candles from Polygon JSON files downloaded via M3 client
- [ ] Compute binomial p-value on win rate (lenient threshold p < 0.10 at this stage)
- [ ] Emit go/no-go decision to stdout:
  `"✅ Edge detected — proceed to M6"` or
  `"⚠️ No significant edge on daily candles — revise strategy before continuing"`
- [ ] Run across at least 15 symbols spanning stocks, ETFs, and crypto

### Testing

- [ ] Deterministic output for fixed candle input (fixed random seed)
- [ ] Look-ahead bias check: inject a candle value that would only exist in the future;
  assert it is never accessed by the strategy
- [ ] Known candle sequence produces expected trade outcome
- [ ] p-value fires correctly: 200 trades at 50/50 → not significant; 200 trades at
  60% win rate → significant

### Decision gate

| Result | Action |
|---|---|
| p < 0.10, profit factor ≥ 1.2, ≥ 200 simulated trades | Proceed to M6 |
| Otherwise | **Stop.** Revise strategy parameters or signal logic, re-run M5.5 |

---

## M6 — Multi-Timeframe Scanner + Signal Generation

### Todos

- [ ] Background task `Scanner` spawned at startup
- [ ] Scanner updates its `last_heartbeat_at` in `AppState` after every full scan cycle
  (feeds the watchdog introduced in M4)
- [ ] Configurable scan interval per timeframe (env: `SCAN_INTERVAL_1D_SECS`, etc.)
- [ ] For each enabled symbol in watchlist, on each scan tick:
  1. Load candles for configured timeframes from DB (no re-fetch — candle store is source of truth)
  2. Run `analyse()` for each timeframe
  3. Run strategy modules (see below)
  4. Aggregate scores across timeframes
  5. If score ≥ `SCORE_THRESHOLD` and no duplicate pending signal: persist to `signals`, enqueue notification
- [ ] **Earnings calendar ingestion** (prerequisite for event-driven strategies):
  - [ ] Background task polling Alpha Vantage or FMP earnings endpoint every 24h
  - [ ] New migration (M6): `earnings_calendar` table:
    ```sql
    CREATE TABLE earnings_calendar (
        symbol          TEXT NOT NULL,
        report_date     DATE NOT NULL,
        eps_estimate    NUMERIC,
        eps_actual      NUMERIC,
        surprise_pct    NUMERIC,
        fetched_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
        PRIMARY KEY (symbol, report_date)
    );
    ```
  - [ ] Helper `earnings_within(symbol, days)` → `Option<Date>` — used by all strategies
- [ ] **Strategy modules** (each is a struct implementing `Strategy` trait):
  - [ ] `EmaPullbackStrategy` — price pulled back to EMA20/50, trend intact, RSI 40–60,
    MACD favorable, volume confirmation
  - [ ] `BreakoutStrategy` — price breaking above prior resistance / Bollinger upper band
    with volume surge
  - [ ] `MeanReversionStrategy` — RSI oversold/overbought extremes with Bollinger Band touch,
    suitable for ranging markets
  - [ ] `EarningsGapStrategy` — post-earnings gap direction + volume confirmation. On the
    session after an earnings report: if gap ≥ `EARNINGS_GAP_MIN_PCT` (env, default 3%)
    in the trend direction with volume ≥ 1.5× 20-day average, treat as a momentum entry.
    Entry at open, stop at gap midpoint, target at 1.618× gap extension. **Available
    immediately — earnings dates are fetched by the earnings calendar task above.**
  - [ ] `PreEarningsMomentumStrategy` — when a stock has been in a confirmed uptrend
    (EMA stack + ADX > 25) and earnings are 3–7 days away, score a continuation entry.
    The earnings date proximity is surfaced as a risk factor, not a disqualifier. This
    captures pre-earnings run-up, a well-documented seasonal effect.
  - [ ] `ShortSqueezeStrategy` — **first-class strategy, not just a score modifier.**
    Setup: high short interest (available from V2-14 when live) + bullish technical
    structure (price holding above key support, RSI recovering from oversold) + a
    catalyst (earnings surprise, unusual options call sweep, or news event). The
    asymmetry: trapped shorts become forced buyers, amplifying the move to 2–5× typical
    R. In MVP (before short interest data is live), runs in technical-only mode — looks
    for the TA pattern and flags as "potential squeeze" if volume is surging on an
    attempted breakdown failure. Lower confidence until short interest data is wired in.
    A squeeze setup should never be sized at normal risk — cap at 0.5× `RISK_PER_TRADE_PCT`
    due to the binary outcome risk if the squeeze fails.
- [ ] **Earnings proximity modifier applied to all strategies:**
  - [ ] Earnings within 0–2 days: add `"⚠️ Earnings in N days — gap risk"` to
    `SignalContext.risk_factors`; reduce composite score by 15 points (except
    `EarningsGapStrategy` which requires an earnings event)
  - [ ] Earnings within 3–5 days: add warning to risk factors; reduce score by 5 points
- [ ] **Multi-timeframe confluence scoring:**
  - [ ] HTF (weekly/daily): trend direction, EMA stack alignment, ADX strength
  - [ ] MTF (4H/1H): setup quality, key level proximity, momentum alignment
  - [ ] LTF (15M/5M): entry trigger, candlestick pattern, volume confirmation
- [ ] Signal deduplication: reject if an active `pending` or `accepted` signal
  already exists for the same `(symbol, side, strategy, timeframe)`
- [ ] `Signal.expires_at` = `created_at + SIGNAL_TTL_HOURS`
- [ ] Emit `audit_log` `signal.generated` on each new signal
- [ ] Scanner respects shutdown signal

### Testing

- [ ] Known candle sequence triggers expected signal from `EmaPullbackStrategy`
- [ ] Duplicate suppression: second scan with same setup produces no new signal
- [ ] Score below threshold: signal stored only if `STORE_BELOW_THRESHOLD=true` (optional
  debug flag), but no Telegram notification
- [ ] Expired signals (past `expires_at`) are not re-triggered
- [ ] Empty watchlist: scanner runs without panic, logs a warning
- [ ] Multiple symbols: signals for different symbols are independent
- [ ] `EarningsGapStrategy`: known gap candle with high volume triggers signal; gap below
  threshold or low volume does not
- [ ] Earnings within 2 days: composite score reduced by 15, risk factor warning present
  in `SignalContext`
- [ ] Earnings within 3–5 days: score reduced by 5, warning present
- [ ] No earnings data for symbol: strategies run without error (earnings modifier skipped)

---

## M6.5 — Backtesting & Strategy Validation (Pre-Live Gate)

> **This milestone is a hard gate.** `live_manual` mode is disabled in `Config` until
> at least one strategy has a passing `backtest_runs` row in the database. The goal is
> 1000+ simulated trades on out-of-sample data with statistically significant edge —
> verified before a single euro of real money is deployed.

### Why this sits here

After M6 (scanner) the TA engine and signal logic exist in pure, testable form. Running
backtests now — before Telegram, risk engine, or deployment — means every subsequent
milestone is built on a strategy that has already proven its edge on real historical data.

### How to get 1000 trades without waiting a year

Polygon.io provides up to 3 years of intraday OHLCV history on its free tier. M4 already
back-fills this on startup. With 3 years of candles across a 30-symbol watchlist and
multiple timeframes, a strategy firing on 2–3% of bars yields well over 1000 simulated
trades immediately — no waiting required.

### Todos

#### Data preparation
- [ ] Verify M4 back-fill covers at least 3 years for all watchlist symbols
- [ ] Add `sector` and `market` columns to `watchlist` table (needed for regime-split
  reporting and correlation checks)
- [ ] Index symbols for broad market proxies: SPY, QQQ, IWM added to watchlist
  automatically as regime anchors (non-tradeable, `enabled=false` for scanning)

#### Walk-forward methodology
- [ ] `WalkForwardSplitter`: given a full candle range and configurable ratios
  (default: 60% train / 20% validation / 20% out-of-sample), returns three
  non-overlapping date windows
- [ ] **The test (out-of-sample) window is never used during strategy design or
  parameter tuning.** The `BacktestRunner` enforces this: the OOS window dates
  are recorded in `backtest_runs` and the CLI warns if the same OOS window is
  run twice (a sign of overfitting to the test set)
- [ ] Validation split: used to sanity-check after training; any parameter change
  triggered by validation results requires resampling a fresh validation window
  from the training period

#### Bar-by-bar simulation (no look-ahead bias)
- [ ] `BacktestRunner::run(strategy, symbols, timeframes, split, config)`:
  1. Load candles from DB for the full window
  2. Walk forward one bar at a time — at each bar, the runner sees only
     `candles[0..=current]`, never future bars
  3. **Entry:** signal fires at bar `N` close → entry at bar `N+1` open + slippage
  4. **Exit:** on each subsequent bar, check high/low against stop and target
     (not just the close) to detect intra-bar hits; if both hit in the same bar,
     assume stop hit first (conservative)
  5. Apply fee per trade (flat EUR amount configurable, default 1.00 EUR)
  6. Track portfolio state: open trades, available capital, cumulative P&L
  7. Respect `MAX_OPEN_TRADES` and `MAX_TOTAL_OPEN_RISK_PCT` as they would apply live
  8. Record every simulated trade in `backtest_trades`
- [ ] Slippage model: configurable basis points per side (default 10 bps for liquid
  stocks, 30 bps for crypto, 20 bps for small-caps/ETFs)
- [ ] **Survivorship-bias note:** document that the current implementation tests on
  today's watchlist (existing symbols). Flag this clearly in backtest output.
  Full survivorship-bias correction (including delisted symbols) is a V2 enhancement.

#### Statistical significance
- [ ] **Binomial test on win rate:** null hypothesis = 50% (coin flip). Compute exact
  p-value. Require p < 0.05 to pass gate (at 1000 trades, even 53% win rate is
  statistically significant).
- [ ] **t-test on R-multiples:** one-sample t-test, null hypothesis = 0.0. Require
  p < 0.05 and mean R > 0.
- [ ] **Bootstrap confidence interval on expectancy:** resample the `backtest_trades`
  R-multiples with replacement 10,000 times. Report 5th, 50th, and 95th percentile
  expectancy. The 5th percentile must be > 0 to pass.
- [ ] Store `win_rate_p_value` in `backtest_runs`.

#### Monte Carlo simulation
- [ ] `MonteCarlo::simulate(r_multiples, n_simulations, initial_capital)`:
  - Randomly reshuffle the trade sequence 10,000 times
  - Simulate the equity curve for each shuffle
  - Output: distribution of final returns, distribution of max drawdowns
- [ ] Store `monte_carlo_p05_dd` (5th-percentile max drawdown) in `backtest_runs`
- [ ] This is the realistic worst-case drawdown at 95% confidence — the number that
  determines how much capital to deploy initially

#### Regime-split analysis
- [ ] Divide the OOS period into named regime windows (derive from SPY candles):
  - Trend up (SPY above 200-day SMA, ADX > 25)
  - Trend down (SPY below 200-day SMA, ADX > 25)
  - Ranging (ADX < 20)
  - High volatility (rolling 20-day HV of SPY > 1.5× its own 1-year average)
- [ ] Compute win rate and expectancy separately for each regime window
- [ ] Store as `regime_breakdown` JSONB in `backtest_runs`
- [ ] A strategy that passes the aggregate gate but collapses in one regime
  (e.g. expectancy < 0 in "Trend down") should emit a prominent warning —
  the market regime detector (M-V2-07) will use this to suppress signals
  in that regime when live

#### Robustness checks (sensitivity analysis)
- [ ] Parameter sweep: vary the top-3 most sensitive strategy parameters by ±20%
  and re-run. If performance degrades by > 30% on any variation, flag the
  strategy as fragile (warn, do not block)
- [ ] Fee sensitivity: re-run with 2× and 5× fees. If the strategy becomes
  unprofitable at 5× fees, flag (Trade Republic pricing can change)

#### Go / No-Go gate
- [ ] `GateChecker::evaluate(run: &BacktestRun) -> GateResult { passed, failures }`:

  | Metric | Minimum to pass |
  |---|---|
  | Trade count (OOS) | ≥ 500 |
  | Win rate | ≥ 50% |
  | Win rate p-value | < 0.05 |
  | Average R-multiple | > 0.0 |
  | R-multiple t-test p-value | < 0.05 |
  | Expectancy (5th-pct bootstrap) | > 0.0 |
  | Profit factor | ≥ 1.3 |
  | Monte Carlo 5th-pct max drawdown | ≤ 25% |
  | Sharpe ratio | ≥ 0.8 |
  | All regime windows: expectancy | > −0.5R (warn if negative, block if < −0.5R) |

- [ ] `passed_gate` column set to `true` only when all minimums are met
- [ ] `Config::validate()` at startup: if `TRADING_MODE=live_manual` and no
  `backtest_runs` row with `passed_gate=true` exists for the active strategy,
  refuse to start with a clear error message

#### Telegram interface
- [ ] `/backtest <STRATEGY> <DAYS>` — runs a full walk-forward backtest over the last
  `<DAYS>` of candle history for all enabled watchlist symbols; sends a summary
  message on completion
- [ ] Backtest summary Telegram message:
  ```
  Backtest: EMA Pullback — 1000 days, 31 symbols
  OOS window: 2025-01-01 → 2025-10-01

  Trades (OOS):    847
  Win rate:        57.3%  (p=0.0003)
  Avg R:           +0.61R (p=0.0012)
  Expectancy:      +0.38R [0.21–0.56 90% CI]
  Profit factor:   1.71
  Sharpe ratio:    1.14
  Max DD (p05 MC): 14.2%

  Regime breakdown:
  Trend up:    +0.52R ✅
  Trend down:  +0.19R ✅
  Ranging:     +0.08R ⚠️ weak
  High vol:    +0.31R ✅

  Gate: ✅ PASSED — live_manual mode unlocked
  ```
- [ ] `/backtest_compare <STRATEGY_A> <STRATEGY_B>` — side-by-side summary of two runs

### Testing

- [ ] Bar-by-bar runner never accesses a future candle (verified by injecting a candle
  that would only be available in the future and asserting it is never seen)
- [ ] Entry is at bar N+1 open, not bar N close (asserted on known test sequence)
- [ ] Stop hit before target on same bar → stop exit recorded (conservative)
- [ ] Fees and slippage reduce P&L by the correct amounts
- [ ] Walk-forward splitter produces non-overlapping windows of correct proportions
- [ ] Binomial p-value: 500 trades at 50% win rate → p ≈ 0.5 (not significant);
  500 trades at 60% win rate → p < 0.0001 (significant)
- [ ] Bootstrap CI: deterministic for fixed random seed
- [ ] Monte Carlo: 5th-percentile drawdown is reproducible for fixed seed
- [ ] `GateChecker`: known failing run returns `passed=false` with correct failure list
- [ ] `Config::validate()` refuses to start in `live_manual` when no passing run exists
- [ ] Sensitivity sweep: parameter variation produces distinct (not identical) results

---

## M7 — Risk Engine

### Todos

- [ ] Configurable via env (all required, validated at startup):
  - [ ] `ACCOUNT_SIZE` (decimal, account currency)
  - [ ] `ACCOUNT_CURRENCY` (e.g. `EUR`)
  - [ ] `RISK_PER_TRADE_PCT` (e.g. `1.0`)
  - [ ] `MIN_RISK_REWARD` (e.g. `3.0`)
  - [ ] `MAX_OPEN_TRADES` (e.g. `5`)
  - [ ] `MAX_TOTAL_OPEN_RISK_PCT` (e.g. `5.0`)
  - [ ] `FRACTIONAL_SHARES` (bool — Trade Republic supports fractional for many assets)
- [ ] Position sizing:
  - [ ] `quantity = (account_size × risk_pct / 100) / |entry_price − stop_loss|`
  - [ ] Round to 2 decimal places if `FRACTIONAL_SHARES=true`, else floor to integer
- [ ] Return type: `RiskDecision::Accept { quantity: Decimal }` or
  `RiskDecision::Reject { reason: String }`
- [ ] Pre-trade checks:
  - [ ] R/R ≥ `MIN_RISK_REWARD` — hard block
  - [ ] Count of `planned` + `open` trades < `MAX_OPEN_TRADES` — hard block
  - [ ] Sum of all open trade risks + this trade risk ≤ `MAX_TOTAL_OPEN_RISK_PCT` — hard block
  - [ ] No existing open/planned trade for the same symbol — hard block
  - [ ] Warn (not block) if two open trades are in the same sector (sector metadata on watchlist)
- [ ] Currency conversion: for non-`ACCOUNT_CURRENCY` assets, use a fixed exchange rate
  from env (`FX_USD_EUR`, etc.) — live FX in V2
- [ ] `MAX_DAILY_LOSS_PCT` and `MAX_WEEKLY_LOSS_PCT` env vars: schema and env vars exist
  in M7; enforcement is V2 (M-V2-16)
- [ ] **Fundamental quality gate** (stocks only — skipped for ETFs, crypto, commodities):
  - [ ] Fetch quarterly fundamentals via FMP or Alpha Vantage: revenue growth (TTM),
    gross margin, debt-to-equity ratio
  - [ ] Cache in `company_fundamentals` table with 24h TTL:
    ```sql
    CREATE TABLE company_fundamentals (
        symbol          TEXT PRIMARY KEY,
        revenue_growth  NUMERIC,   -- YoY TTM revenue growth as decimal (e.g. -0.12)
        gross_margin    NUMERIC,   -- e.g. 0.42
        debt_to_equity  NUMERIC,
        fetched_at      TIMESTAMPTZ NOT NULL DEFAULT now()
    );
    ```
  - [ ] Reject signal if any of: revenue growth < `FUND_MIN_REVENUE_GROWTH` (env,
    default −0.15), gross margin < `FUND_MIN_GROSS_MARGIN` (env, default 0.0),
    debt-to-equity > `FUND_MAX_DE_RATIO` (env, default 5.0)
  - [ ] `FUNDAMENTAL_GATE_ENABLED` env var (default true); configurable thresholds so
    users can tune aggressively for growth plays or defensively for blue chips
  - [ ] `RiskDecision::Reject` with human-readable reason, e.g. `"Fundamental gate:
    revenue −18% YoY (threshold −15%)"`
  - [ ] Data unavailable (private company, no filing): skip gate, log warning

### Testing

- [ ] Correct quantity for known inputs (long, short, EUR, USD)
- [ ] R/R below minimum → `Reject`
- [ ] `MAX_OPEN_TRADES` reached → `Reject`
- [ ] Duplicate symbol open → `Reject`
- [ ] Total risk would exceed cap → `Reject`
- [ ] Zero entry-stop distance → `Reject` with safe error message, no panic
- [ ] Fractional vs integer rounding is correct
- [ ] Fundamental gate: revenue growth below threshold → `Reject` with reason
- [ ] Fundamental gate: ETF / crypto symbol → gate skipped, signal proceeds
- [ ] Fundamental data unavailable → gate skipped, warning logged, signal proceeds
- [ ] `FUNDAMENTAL_GATE_ENABLED=false` → all fundamental checks bypassed

---

## M8 — Telegram Notification + Paper Mode

### Todos

- [ ] `teloxide` in webhook mode, mounted at `POST /telegram/webhook`
- [ ] Set webhook on bot start: call `set_webhook` with `secret_token` header
- [ ] Validate `X-Telegram-Bot-Api-Secret-Token` on every Telegram request → 401 if wrong
- [ ] Load `TELEGRAM_BOT_TOKEN`, `TELEGRAM_CHAT_ID` from env
- [ ] `TRADING_MODE` env var: `paper` | `live_manual` (default `paper`)
- [ ] `TelegramClient` trait with `send_signal_notification`, `send_message`,
  `edit_message`, `send_detailed_analysis` — allows mocking in tests
- [ ] Enable `message_reaction` in `allowed_updates` when registering the webhook so the
  bot receives emoji reactions from the user
- [ ] **`SignalContext`** struct stored in `signals.raw_context` JSONB — must include:
  - `top_reasons: Vec<String>` — 3–5 human-readable bullets explaining the setup
  - `risk_factors: Vec<String>` — 1–3 things the user should watch out for
  - `score_breakdown: ScoreBreakdown` — per-category scores and max weights
  - Key indicator values at signal time (EMA stack, RSI, ATR, volume ratio, etc.)
- [ ] **Short signal notification** (always sent, kept concise):
  ```
  [PAPER] 📈 AAPL · Long  ·  Score 72/100

  Entry 184.20  ·  Stop 174.99  ·  Target 211.83
  R/R 3.0×  ·  Risk €184  ·  EMA Pullback · 1D → 4H

  ↑ Trend aligned  ·  RSI 52 healthy  ·  Bullish engulfing 4H

  ⏰ Expires in 4h  ·  React: ❓ analysis · 🤔 bear case · ⏰ snooze · 🔕 mute · 👍👎 feedback
  ```
  Rules: no more than 6 lines of content above the buttons; lead with the numbers the
  user needs to judge the trade at a glance; top 2–3 reasons as a single compact line.
- [ ] **Inline buttons** below every signal notification:
  `✅ Accept` (callback `accept:<uuid>`) · `❌ Reject` (callback `reject:<uuid>`)
- [ ] **Detailed analysis message** (sent on ❓ reaction — see M9):
  ```
  📊 Full Analysis — AAPL Long  ·  Score 72/100

  📈 Why this trade?
  • Daily uptrend intact: EMA 20 > 50 > 200 (strong stack)
  • Price pulled back to EMA 50 — historically high-probability support
  • RSI 52: healthy momentum, not overbought, room to run
  • Bullish engulfing on 4H: buyers absorbed the full prior red candle
  • Volume 1.4× 20-day average on the bounce — conviction confirmed

  🌍 Market context
  • Regime: Bull — SPY above 200 SMA, golden cross active
  • No high-impact US events in the next 24h ✓
  • News sentiment: Neutral (no major headlines in last 24h)

  📐 Key levels
  • Entry 184.20 (EMA 50 touch)  ·  Stop 174.99 (below prior week low + 0.5 ATR)
  • Target 211.83 (weekly resistance / fib 1.618)
  • ATR(14): 3.20 — typical daily range

  ⚠️ Watch out for
  • Earnings in 12 days — consider closing before
  • Tech sector: 1 other open trade (NVDA)

  Score breakdown:
    Trend alignment   ████████████████████░░░░░  20/25
    Setup quality     ████████████████░░░░░░░░░  16/20
    Entry confirm     ████████████░░░░░░░░░░░░░  12/15
    Risk/Reward       ██████████████░░░░░░░░░░░  14/15
    News sentiment    █████░░░░░░░░░░░░░░░░░░░░   5/10
    Social sentiment  ███░░░░░░░░░░░░░░░░░░░░░░   3/5
    Macro / regime    ████░░░░░░░░░░░░░░░░░░░░░   4/5
    Prediction mkt    ██░░░░░░░░░░░░░░░░░░░░░░░   2/5
    ─────────────────────────────────────────
    Total                                        72/100
  ```
  The detailed message ends with the same `✅ Accept` / `❌ Reject` buttons so the user
  can act immediately without scrolling back up.
- [ ] Store `telegram_msg_id` on the signal after send
- [ ] Notification gating: `score < SCORE_THRESHOLD` → no send (signal still stored)
- [ ] Telegram send failure: log via `tracing::error!`, do not 500 the scanner task

### Testing

- [ ] `MockTelegramClient` records all sends; scanner → 1 send observed for qualifying signal
- [ ] Short message fits within 6 content lines; contains symbol, side, entry, stop, target, R/R, score, mode label, top reasons, expiry hint
- [ ] Detailed analysis message contains all score breakdown categories, top reasons, risk factors, key levels
- [ ] Below-threshold score: signal stored, no Telegram send
- [ ] Send failure is logged and does not crash the scanner
- [ ] Bad `X-Telegram-Bot-Api-Secret-Token` → 401

---

## M9 — Telegram Callback Handling + Trade Execution

### Todos

- [ ] Parse all incoming update types: callback queries, message reactions, text commands
- [ ] New migration (M9): add `user_feedback TEXT CHECK (user_feedback IN ('good','bad'))`
  nullable column to `signals`; add `symbol_mutes` table:
  ```sql
  CREATE TABLE symbol_mutes (
      symbol      TEXT NOT NULL,
      muted_until TIMESTAMPTZ NOT NULL,
      PRIMARY KEY (symbol)
  );
  ```
- [ ] Pause state: `notifications_paused: AtomicBool` in `AppState` (in-memory, resets
  on restart — acceptable for a personal bot)
- [ ] **Reaction dispatch** (`MessageReactionUpdated`): look up signal by
  `telegram_msg_id`; route by reaction emoji to the appropriate handler below
- [ ] **❓ — Full analysis:**
  1. If signal not found or expired → "Signal no longer available"
  2. Build `DetailedAnalysis` from `signals.raw_context`; send as threaded reply with
     `✅ Accept` / `❌ Reject` buttons
  3. Idempotent (re-sending on repeated reactions is fine — no state mutation)
  4. Emit `audit_log` `signal.detail_requested`
- [ ] **🤔 — Bear case:**
  1. Build a focused counter-argument from `signals.raw_context`: weakest score
     categories, `risk_factors`, upcoming macro events within 48h
  2. Format:
     ```
     🤔 Bear Case — AAPL Long

     Why this might not work:
     • News sentiment only 5/10 — no positive catalyst
     • Earnings in 12 days — gap risk on close
     • Social sentiment weak (StockTwits bullish ratio 0.42)

     Weakest categories:
       News sentiment   █████░░░░░░  5/10
       Prediction mkt   ██░░░░░░░░░  2/5
     ```
  3. Ends with `✅ Accept anyway` / `❌ Reject` buttons
  4. Emit `audit_log` `signal.bear_case_requested`
- [ ] **⏰ — Snooze:**
  1. Extend `signals.expires_at` by `SNOOZE_HOURS` (env, default 2); cap at
     `SIGNAL_MAX_AGE_HOURS` so signals cannot be snoozed indefinitely
  2. Reply: "⏰ Snoozed — signal now expires at <new_time>"
  3. Idempotent: snoozing an already-snoozed signal extends further (up to cap)
  4. Emit `audit_log` `signal.snoozed`
- [ ] **🔕 — Mute symbol:**
  1. Upsert into `symbol_mutes (symbol, muted_until = now() + MUTE_HOURS)` (env,
     default 24h); scanner skips notifications for muted symbols
  2. Reply: "🔕 <SYMBOL> muted for 24h"
  3. Emit `audit_log` `symbol.muted`
- [ ] **👍 — Positive feedback:**
  1. Set `signals.user_feedback = 'good'`
  2. Reply: "👍 Noted"
  3. Emit `audit_log` `signal.feedback_good`
- [ ] **👎 — Negative feedback:**
  1. Set `signals.user_feedback = 'bad'`
  2. Reply: "👎 Noted"
  3. Emit `audit_log` `signal.feedback_bad`
- [ ] Unknown reaction → silent ignore (no audit log, no reply)
- [ ] Route callback queries by prefix (`accept:`, `reject:`, `executed:`,
  `cancel_trade:`, `same_price:`)
- [ ] **Reject path:** signal → `rejected`; edit original message to show rejection;
  emit `audit_log` `signal.rejected`
- [ ] **Accept path:**
  1. **Expiry check:** if `now() > signal.expires_at` → mark `expired`, send
     user-friendly message, audit log, stop — no trade
  2. **Risk engine check:** run `RiskEngine::evaluate(signal)` — if `Reject`, send
     reason, mark signal `rejected`, audit log, stop
  3. Signal → `accepted`; create `trades` row `status='planned'`, `trading_mode` from config
  4. **Paper mode:** immediately set `status='open'`, `actual_entry_price=planned_entry_price`,
     `opened_at=now()`; send confirmation; skip execution buttons
  5. **Live mode:** send follow-up with `executed:<trade_id>` / `cancel_trade:<trade_id>` buttons
- [ ] Idempotency: Accept twice → exactly one trade (transaction + status check)
- [ ] Unknown / malformed callback → silent ignore + audit log
- [ ] **Conversation state** for live execution flow, stored in `DashMap<ChatId, ConversationState>`:
  - `AwaitingFillPrice { trade_id }` → `AwaitingQuantity { trade_id, fill_price }` →
    `Done`
  - State is in-memory only; app restart resets it (user re-presses `executed`)
- [ ] `executed:<trade_id>` callback:
  - [ ] Ask for actual fill price (free text or `same_as_planned` button)
  - [ ] Validate price > 0; on invalid input ask again (max 3 retries then abort)
  - [ ] Ask for quantity (or accept risk-engine default with one-tap confirm)
  - [ ] Set `actual_entry_price`, `quantity`, `opened_at = now()`, `status = 'open'`
- [ ] `cancel_trade:<trade_id>` → `status = 'invalidated'`
- [ ] Telegram commands:
  - [ ] `/open` — list open + planned trades (symbol, side, entry, current P&L if available)
  - [ ] `/close <trade_id> <price>` — set `closed_manual`, compute `realized_pnl`, `closed_at`
  - [ ] `/watchlist` — list enabled symbols
  - [ ] `/add <SYMBOL>` — add symbol to watchlist (triggers back-fill)
  - [ ] `/remove <SYMBOL>` — disable symbol in watchlist
  - [ ] `/score <SYMBOL>` — run analysis on demand, return current score without
    generating a signal
  - [ ] `/why <SYMBOL>` — explain why no signal was generated: list the conditions that
    failed (e.g. "RSI 71 — overbought · ADX 14 — no trend · score 38/100 < threshold").
    If a signal does exist, redirect to it instead
  - [ ] `/risk <entry> <stop> [qty]` — position-size calculator independent of any signal;
    returns quantity, risk amount, and R/R for a user-supplied setup. Useful when the
    user spots a trade manually in their broker
  - [ ] `/pause` — set `notifications_paused = true`; scanner keeps running and storing
    signals but sends no Telegram messages. Reply: "🔇 Notifications paused. Use /resume
    to re-enable. Signals are still being recorded."
  - [ ] `/resume` — set `notifications_paused = false`; immediately send a digest of any
    pending signals that accumulated while paused (up to `RESUME_DIGEST_LIMIT`, default 5,
    oldest first)
  - [ ] `/status` — one-message bot health summary:
    ```
    SwingBot status
    Last scan:    14s ago  ✓
    Last ingest:  47s ago  ✓
    DB:           ok
    Polygon:      ok
    Open trades:  2
    Account risk: 1.8% / 5.0% max
    Mode:         paper
    Paused:       no
    ```
  - [ ] `/mode paper|live` — switch `TRADING_MODE` at runtime (updates in-memory config;
    persisted via env on restart); reply confirms new mode
- [ ] **Forwarded message / article handler:** when the user forwards any text message
  into the chat (e.g. a news headline or article snippet), extract ticker symbols
  (cashtag `$AAPL` pattern and known watchlist symbols mentioned by name); for each
  found symbol reply with its current score and any active pending signal. No LLM
  required — keyword + watchlist lookup only. Unknown tickers get a brief "not on
  watchlist" note with an `/add` prompt
- [ ] `DISPLAY_TZ` env var — convert timestamps for display
- [ ] Emit `audit_log` on every state change

### Testing

- [ ] ❓ reaction → detailed analysis sent as threaded reply with ✅/❌ buttons, audit row written
- [ ] ❓ on unknown message ID → "Signal no longer available", no crash
- [ ] ❓ on expired signal → "Signal no longer available"
- [ ] ❓ twice → two analysis replies, no duplicate state mutation
- [ ] 🤔 reaction → bear case reply with weakest categories and risk factors, ✅/❌ buttons
- [ ] ⏰ reaction → `expires_at` extended by `SNOOZE_HOURS`, confirmation reply
- [ ] ⏰ snooze beyond `SIGNAL_MAX_AGE_HOURS` cap → clamped to cap, not exceeded
- [ ] 🔕 reaction → row upserted in `symbol_mutes`, scanner skips that symbol, confirmation reply
- [ ] 🔕 twice → mute extended (upsert), not duplicated
- [ ] 👍 reaction → `user_feedback = 'good'` set, "👍 Noted" reply, audit row
- [ ] 👎 reaction → `user_feedback = 'bad'` set, "👎 Noted" reply, audit row
- [ ] Unknown reaction → no reply, no crash, no audit row
- [ ] `/why AAPL` with no active signal → human-readable explanation of failing conditions
- [ ] `/why AAPL` with active pending signal → redirect to existing signal
- [ ] `/risk 184.20 174.99` → correct quantity and risk amount returned
- [ ] `/risk` with stop == entry → safe error, no panic
- [ ] `/pause` → `notifications_paused = true`; qualifying signal generated after pause → no Telegram send
- [ ] `/resume` → `notifications_paused = false`; digest of accumulated signals sent (≤ `RESUME_DIGEST_LIMIT`)
- [ ] `/status` → contains last scan time, DB status, open trade count, account risk pct, mode, pause state
- [ ] `/mode live` → trading mode updated in AppState; subsequent accept creates live trade
- [ ] `/mode invalid` → user-friendly error, mode unchanged
- [ ] Forwarded message with `$AAPL` → score and active signal for AAPL returned
- [ ] Forwarded message with unknown ticker → "not on watchlist" + `/add` prompt
- [ ] Forwarded message with no recognisable ticker → no reply (silent)
- [ ] Reject → signal status updated, message edited, audit row written
- [ ] Accept → exactly one trade created
- [ ] Double Accept → still one trade
- [ ] Accept on rejected signal → user-friendly error, no state change
- [ ] Risk-rejected accept → no trade, reason sent
- [ ] Expired signal → `expired` status, no trade, user-friendly message
- [ ] Paper mode → trade immediately `open` at planned price
- [ ] Live mode → `executed` / `cancel_trade` buttons shown
- [ ] `executed` flow: invalid price rejected, conversation recovers
- [ ] `/close` computes correct P&L for long and short
- [ ] `/open` shows only open and planned trades
- [ ] Malformed UUID → no crash, audit row written

---

## M10 — Production Deploy

### Todos

- [ ] Multi-stage `Dockerfile` (builder on `rust:slim`, final on `gcr.io/distroless/cc`)
- [ ] **`sqlx` offline mode required for Docker build:**
  - [ ] Run `cargo sqlx prepare` locally; commit `sqlx-data.json`
  - [ ] Set `SQLX_OFFLINE=true` in Dockerfile
  - [ ] Add `cargo sqlx prepare --check` to CI to keep `sqlx-data.json` in sync
- [ ] `docker-compose.prod.yml` — app + Postgres 16 + Caddy (HTTPS with auto-cert)
- [ ] Domain / subdomain pointed at server
- [ ] `.env.production.example` — committed without secrets
- [ ] Restart policy: `unless-stopped` on all services
- [ ] Postgres volume backed up: nightly `pg_dump` cron + 7-day retention + restore test
- [ ] Structured JSON logs written to file; `logrotate` config
- [ ] `GET /health` — liveness (always 200 if process is up)
- [ ] `GET /ready` — readiness (200 only when DB pool and market data provider reachable)
- [ ] Caddy (or Traefik) configured as reverse proxy — `SmartIpKeyExtractor` in
  `tower_governor` so rate limiting keys on the forwarded client IP, not `127.0.0.1`

### Testing

- [ ] Container builds in CI without live DB (`SQLX_OFFLINE=true`)
- [ ] `docker compose up` on server brings app up cleanly
- [ ] HTTPS endpoint responds with valid cert
- [ ] App and Postgres survive server reboot with data intact
- [ ] `pg_dump` produces a restorable dump (restore tested into scratch DB)
- [ ] Nightly backup cron fires and produces a file

---

## MVP Definition of Done

- [ ] Bot continuously scans configured watchlist across multiple timeframes
- [ ] TA engine scores setups and only notifies on qualifying signals
- [ ] **At least one strategy has a passing backtest run (M6.5 gate) with ≥ 500
  out-of-sample trades, statistically significant edge, and max drawdown ≤ 25%**
- [ ] `live_manual` mode is locked until the above gate is met
- [ ] Telegram notification includes full signal context (score breakdown, key indicators)
- [ ] Accept → risk engine check → trade created (paper or live)
- [ ] Paper mode auto-opens at planned price; live mode walks user through fill entry
- [ ] `/close`, `/open`, `/watchlist`, `/add`, `/remove`, `/score`, `/backtest` all work
- [ ] Audit log captures every state-changing event
- [ ] Runs in Docker on a server with HTTPS, restart policy, nightly DB backup
- [ ] Restart does not corrupt or duplicate any data
- [ ] CI: fmt + clippy + tests + sqlx-check on every push

---
---

# Part 2 — Post-MVP Roadmap

Each milestone below is independent and can be shipped in any order after the MVP is
live. **Suggested order prioritises the data sources with the highest expected alpha
first:** options flow and insider activity (V2-02) and LLM synthesis (V2-05) are
elevated above their original positions because they represent edge that TA alone
cannot capture.

---

## M-V2-01 — News Sentiment Ingestion

### Todos

- [ ] `NewsSentimentProvider` trait:
  ```rust
  async fn fetch_news(&self, symbol: &str, from: DateTime<Utc>) -> anyhow::Result<Vec<NewsItem>>;
  ```
  `NewsItem { headline, source, url, published_at, sentiment_score: Decimal, symbols: Vec<String> }`
- [ ] Implement `NewsApiProvider` (NewsAPI.org)
- [ ] Implement `FmpNewsProvider` (Financial Modeling Prep — includes sentiment tags)
- [ ] Background task: poll news every `NEWS_POLL_INTERVAL_SECS` (default 300)
- [ ] Store in `news_items` table; deduplicate by URL
- [ ] Compute rolling 24h sentiment score per symbol: average of `sentiment_score`
  weighted by recency
- [ ] Feed 24h sentiment into signal scorer (Category: News sentiment, weight 10)
- [ ] Telegram alert for any single article with sentiment score > `NEWS_ALERT_THRESHOLD`
  (configurable), regardless of scanner signal
- [ ] Strip any PII or credentials from stored news payloads

### Testing

- [ ] Mock provider returns known articles; sentiment score matches expected computation
- [ ] Deduplication: same URL ingested twice produces one DB row
- [ ] High-impact article triggers Telegram alert
- [ ] Score contribution: positive sentiment raises composite score, negative lowers it

---

## M-V2-02 — Options Flow + Insider Activity

> **Elevated from original V2-09.** These two data sources are among the most
> reliable alternative signals available to retail traders. Institutions cannot hide
> large options bets or insider filings — this is legally public information that
> systematically precedes large moves.

### Options Flow (Unusual Activity)

- [ ] Polygon.io options endpoint — scan for unusual open interest and large premium orders
  on watchlist symbols relative to 30-day average OI
- [ ] `UnusualOptionsEvent { symbol, expiry, strike, call_put, oi, premium, oi_ratio,
  detected_at }` — store in `unusual_options` table
- [ ] Classify: `call_sweep` (bullish institutional bet) vs. `put_sweep` (bearish hedge)
- [ ] Options flow score contribution: large call sweep on a bullish TA signal → +score;
  large put sweep on a long signal → penalty or suppression
- [ ] Telegram alert when unusual options activity detected for a watchlist symbol,
  independent of any scanner signal (breaking news mode)
- [ ] `UNUSUAL_OI_RATIO_THRESHOLD` env var (default 3.0 — OI must be 3× the 30-day avg)

### Insider Activity (SEC Form 4)

- [ ] SEC EDGAR Form 4 parser: fetch filings for watchlist symbols via EDGAR full-text
  search API (free, no key required)
- [ ] `InsiderFiling { symbol, insider_name, role, transaction_type: Buy|Sell,
  shares, price, filed_at }` — store in `insider_activity` table
- [ ] Threshold: only flag buys > `INSIDER_BUY_MIN_USD` (env, default $50,000)
- [ ] Cluster detection: ≥ 2 insiders buying within 5 trading days → strong bullish modifier
- [ ] Insider buy score boost: +5 to composite; cluster buy: +8
- [ ] Large insider sell: −3 (sells are noisier — insiders sell for many reasons)
- [ ] Telegram alert on any qualifying insider buy, independent of scanner signal
- [ ] Emit `audit_log` on every filing ingested

### IV Rank (Implied Volatility Context)

- [ ] Polygon options data (same paid tier as unusual options flow above)
- [ ] Compute IV rank: `(current_IV − 52w_low_IV) / (52w_high_IV − 52w_low_IV) × 100`
- [ ] Store in `options_iv` table: `symbol, iv_rank, current_iv, iv_52w_high, iv_52w_low,
  fetched_at`
- [ ] **Score modifier:**
  - High IV rank (> 70) on a breakout signal: options market already pricing in the move
    — premium is expensive, risk/reward deteriorates; add −4 to score and a risk factor
    "High IV rank: market already expects a move"
  - Low IV rank (< 30) with bullish setup: options cheap, move not expected by market
    — breakout has more room; add +3 to score
  - IV rank 30–70: neutral, no modifier
- [ ] IV rank shown in detailed Telegram analysis: "IV rank: 23 (low — options cheap)"

### Testing

- [ ] OI ratio below threshold: no event, no alert
- [ ] OI ratio above threshold: event stored, Telegram alert fires
- [ ] Call sweep on long signal: score raised by expected amount
- [ ] Put sweep on long signal: score penalised by expected amount
- [ ] Insider buy above threshold: filing stored, Telegram alert fires
- [ ] Insider buy cluster (≥ 2 within 5 days): stronger modifier applied
- [ ] Insider sell: −3 modifier, no alert
- [ ] EDGAR fetch failure: logged, task does not crash
- [ ] IV rank > 70 on breakout: −4 modifier, risk factor added to SignalContext
- [ ] IV rank < 30 on bullish setup: +3 modifier
- [ ] IV rank 30–70: no modifier

---

## M-V2-03 — Cross-Sectional Momentum + Sector Rotation

> **New milestone.** Entirely absent from the original plan and high impact-to-effort.
> A stock breaking out while its sector is breaking down is a much weaker signal than
> one leading a sector breakout. Cross-sectional ranking is a computation over data
> you already have — no new APIs required.

### Todos

- [ ] **Sector assignment:** add `sector` (GICS sector) and `industry` columns to
  `watchlist` table; seed from FMP or a static CSV; ETFs tagged by benchmark sector
- [ ] Sector ETFs added automatically as non-tradeable watchlist anchors (`enabled=false`):
  XLK (tech), XLF (financials), XLE (energy), XLV (health), XLP (consumer staples),
  XLY (consumer discretionary), XLI (industrials), XLB (materials), XLRE (real estate),
  XLU (utilities), XLC (communication)
- [ ] **Relative strength computation:**
  - [ ] `relative_strength(symbol_candles, benchmark_candles, period)` → `Decimal`
    — rate of change of symbol minus rate of change of benchmark over `period` bars
  - [ ] Run weekly for all watchlist symbols; benchmark = SPY for stocks/ETFs, QQQ for
    tech-heavy names (configurable per symbol via `watchlist.benchmark`)
  - [ ] Store results in `relative_strength` table:
    ```sql
    CREATE TABLE relative_strength (
        symbol              TEXT NOT NULL,
        period_days         INTEGER NOT NULL,
        rs_score            NUMERIC NOT NULL,
        rank_in_sector      NUMERIC NOT NULL,   -- percentile 0–100
        rank_in_watchlist   NUMERIC NOT NULL,
        computed_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
        PRIMARY KEY (symbol, period_days, computed_at)
    );
    ```
- [ ] **Sector momentum filter:**
  - [ ] Long signal passes sector filter if: sector ETF is above its 20-day EMA AND
    `rank_in_sector` ≥ `SECTOR_RANK_MIN_LONG` (env, default 50th percentile)
  - [ ] Short signal passes if: sector ETF is below its 20-day EMA AND
    `rank_in_sector` ≤ `SECTOR_RANK_MAX_SHORT` (env, default 50th percentile)
  - [ ] `SECTOR_FILTER_ENABLED` env var (default true)
- [ ] **Cross-sectional score modifier:**
  - Top quartile (rank ≥ 75) → +5 composite score
  - Bottom quartile (rank ≤ 25) → −8 composite score
  - Sector ETF in downtrend and signal is long → −5 additional
- [ ] Sector context shown in Telegram notification: `"Sector rank 3/12 ✅"` or
  `"⚠️ Sector trending down"`
- [ ] **Weekly RS digest** (every Monday): top-3 and bottom-3 symbols by relative
  strength per sector; helps user spot sector rotation opportunities manually

### Testing

- [ ] RS computation correct against known benchmark candle sequence
- [ ] Top-quartile rank → +5 modifier applied; bottom-quartile → −8 modifier
- [ ] Sector ETF below 20 EMA: long signal suppressed (score below threshold)
- [ ] `SECTOR_FILTER_ENABLED=false`: filter bypassed, modifiers still applied
- [ ] Weekly digest contains correct ranking for each sector
- [ ] Symbol with no sector tag: no crash, sector modifier skipped

---

## M-V2-04 — Strategy Performance Feedback Loop

> **New milestone.** Fixed scoring weights are calibrated once at design time and never
> updated. A strategy that has been wrong five times running in the current regime should
> have its contribution reduced automatically. This is the feedback loop that turns the
> bot from a static ruleset into an adaptive system — without requiring ML.

### Todos

- [ ] `strategy_performance` table:
  ```sql
  CREATE TABLE strategy_performance (
      strategy        TEXT NOT NULL,
      asset_class     TEXT NOT NULL,
      window_days     INTEGER NOT NULL,   -- rolling window: 30, 90, 365
      trade_count     INTEGER NOT NULL,
      win_rate        NUMERIC NOT NULL,
      avg_r           NUMERIC NOT NULL,
      expectancy      NUMERIC NOT NULL,
      suspended       BOOLEAN NOT NULL DEFAULT false,
      last_updated    TIMESTAMPTZ NOT NULL DEFAULT now(),
      PRIMARY KEY (strategy, asset_class, window_days)
  );
  ```
- [ ] Updated on every trade close (paper or live); back-filled from existing `trades`
  table on startup
- [ ] **Dynamic score multiplier** (`DYNAMIC_WEIGHTING_ENABLED` env var, default false
  — enable after accumulating ≥ 30 trades):
  - 30-day expectancy ≥ 0.5R → multiplier 1.2
  - 30-day expectancy 0.0–0.5R → multiplier 1.0
  - 30-day expectancy −0.2–0.0R → multiplier 0.8
  - 30-day expectancy < −0.2R → multiplier 0.5
  - Fall back to 90-day window if < 10 trades in 30-day; fall back to 1.0 if < 5 trades
    in any window
- [ ] Multiplier applied to strategy's score contribution before final composite score
- [ ] **Automatic suspension:** if 30-day expectancy < −0.5R with ≥ 15 trades →
  suspend strategy; send Telegram alert: `"⚠️ EMA Pullback suspended: 30d expectancy
  −0.7R (15 trades). Use /strategy_resume ema_pullback to re-enable."`
  - `suspended=true` in `strategy_performance`; scanner skips suspended strategies
  - `/strategies` Telegram command: list all strategies with trade count, 30d expectancy,
    multiplier, suspended flag
  - `/strategy_resume <name>` — re-enable a suspended strategy (requires manual decision)
- [ ] Emit `audit_log` on multiplier change and suspension

### Testing

- [ ] Expectancy ≥ 0.5R → multiplier 1.2 applied to score contribution
- [ ] Expectancy < −0.2R → multiplier 0.5 applied
- [ ] Expectancy < −0.5R with ≥ 15 trades → suspended, Telegram alert sent
- [ ] < 5 trades in any window → multiplier defaults to 1.0
- [ ] Suspended strategy not run by scanner
- [ ] `DYNAMIC_WEIGHTING_ENABLED=false` → multiplier always 1.0, no suspension
- [ ] `/strategies` output reflects per-strategy metrics and suspended flag

---

## M-V2-05 — Economic Calendar Integration

### Todos

- [ ] `EconomicCalendarProvider` trait returning `Vec<EconomicEvent>`
  `EconomicEvent { name, country, impact: High|Medium|Low, scheduled_at, actual, forecast, previous }`
- [ ] Implement provider (Trading Economics API or Alpha Vantage)
- [ ] Background task: refresh calendar once daily and on startup
- [ ] Store in `economic_events` table
- [ ] **Signal suppression:** if a High-impact event for the asset's primary market
  is scheduled within `EVENT_BLACKOUT_HOURS` (env, default 2) before/after signal
  creation, add warning to notification and optionally suppress (configurable)
- [ ] Telegram digest: every trading day morning, send a list of today's high-impact events
- [ ] Asset-to-market mapping: stocks → US events; crypto → global; metals → US + China

### Testing

- [ ] Event within blackout window: signal notification includes warning banner
- [ ] Suppression mode: no notification sent when `SUPPRESS_ON_HIGH_IMPACT=true`
- [ ] Morning digest sent at configured time with correct events
- [ ] Event outside blackout: no suppression, no warning

---

## M-V2-06 — Social Media Signal Ingestion

This is the highest-impact alternative data source. Implement in layers.

### Layer A — Twitter/X

- [ ] Twitter API v2 developer account + bearer token (`TWITTER_BEARER_TOKEN` env)
- [ ] Monitor a configured list of accounts (`TWITTER_WATCH_ACCOUNTS` — CSV of @handles):
  - [ ] Heads of state (US President, etc.)
  - [ ] Central bank officials (Fed Chair, ECB President)
  - [ ] High-profile financial commentators and fund managers
  - [ ] Company CEOs for watchlist symbols
- [ ] Monitor keywords and cashtags: `$AAPL`, `$BTC`, `tariff`, `rate hike`, etc.
- [ ] Monitor trending topics (US Trending) — flag if finance-adjacent
- [ ] Store tweets in `social_posts` table: `author`, `platform`, `content`, `posted_at`,
  `mentions` (symbols detected), `sentiment_score`, `reach` (follower count × engagement)
- [ ] NLP pipeline: keyword-based sentiment + entity extraction (no LLM in V2 — use
  a curated keyword dictionary; LLM integration is V3)
- [ ] High-reach + high-sentiment tweet → immediate Telegram alert (breaking news mode)
- [ ] Rolling 4h social buzz score per symbol → fed into signal scorer (weight 5)

### Layer B — Reddit

- [ ] Reddit API (`REDDIT_CLIENT_ID`, `REDDIT_CLIENT_SECRET`)
- [ ] Monitor: r/wallstreetbets, r/investing, r/stocks, r/cryptocurrency, r/realestateinvesting
- [ ] Track post and comment velocity per symbol (mentions per hour)
- [ ] WSB "rocket / moon" emoji count as bullish signal; "puts" / "crash" as bearish
- [ ] Store in `social_posts` with `platform='reddit'`
- [ ] Unusual mention velocity spike → Telegram alert

### Layer C — StockTwits

- [ ] StockTwits API (public endpoints, no auth required for read)
- [ ] Fetch `bullish_count` / `bearish_count` per symbol
- [ ] Bullish ratio = `bullish / (bullish + bearish)` → feeds sentiment score

### Testing

- [ ] Tweet by monitored account → stored, sentiment computed, high-reach tweet triggers alert
- [ ] Symbol cashtag detected correctly across platforms
- [ ] Reddit velocity spike detected and alerted
- [ ] StockTwits bullish ratio feeds correctly into composite score

---

## M-V2-07 — LLM-Enhanced Analysis

> **Elevated from original V2-14.** Once news (V2-01) and social (V2-04) data are live,
> Claude can synthesise technical state + news headlines + social sentiment into a verdict
> that no keyword-matching or TA formula can replicate. This is qualitatively different
> alpha — it reasons about *why* a setup matters in the current context.

### Todos

- [ ] Define `LlmAnalysisProvider` trait:
  ```rust
  async fn analyse_signal_context(&self, context: &SignalContext) -> anyhow::Result<LlmVerdict>;
  ```
  `LlmVerdict { confidence: Decimal, summary: String, concerns: Vec<String>, direction_agreement: bool }`
- [ ] Implement `ClaudeProvider` using the Anthropic API (`claude-sonnet-4-6` default)
  with prompt caching for the system preamble and market context blocks (cache saves
  ~80% of tokens on repeated signals for the same symbol within a session)
- [ ] On signal generation: call LLM with a structured prompt containing:
  - Technical indicator summary (EMA stack, RSI, MACD, ATR, volume ratio)
  - Recent news headlines for the symbol from the last 24h (M-V2-01)
  - Recent relevant social posts from the last 4h (M-V2-04)
  - Market regime summary (M-V2-09 when available; otherwise SPY vs. 200 SMA)
  - The signal's entry / stop / target / R/R
- [ ] LLM verdict contributes to signal score (weight 8, as per scoring model)
- [ ] `direction_agreement=false` (LLM disagrees with signal direction) → reduce score
  by 12 additional points regardless of other factors; surface prominently in notification
- [ ] LLM verdict shown in short signal notification (1 line) and full detailed analysis
- [ ] `LLM_ENABLED=false` env var — default off; zero cost and zero latency when disabled
- [ ] Timeout: `LLM_TIMEOUT_SECS` (env, default 15); on timeout, log warning and skip
  LLM contribution (do not block signal delivery)
- [ ] Never include raw API keys, trade IDs, or PII in the LLM prompt

### Testing

- [ ] Mock provider returns fixed verdict; score contribution is correct
- [ ] `direction_agreement=false` applies the additional −12 penalty
- [ ] With `LLM_ENABLED=false`: no API calls made, score contribution is 0
- [ ] LLM timeout: signal is still delivered, LLM score omitted, warning logged
- [ ] Prompt caching: repeated calls with same preamble result in cache hits (verify
  via Anthropic SDK `usage.cache_read_input_tokens` field in response)
- [ ] No API keys or PII appear in the prompt (assert on captured prompt content)

---

## M-V2-08 — Dark Pool / Block Trade Monitoring

> FINRA publishes Alternative Trading System (ATS) volume data weekly — free, no API
> key required. Large institutional block prints on dark pools often precede moves by
> 3–5 days at swing timeframes. This is legally public information that retail can
> actually access and systematically track.

### Todos

- [ ] FINRA ATS data downloader: fetch the weekly CSV report from the FINRA website
  (released each Monday, covers the prior week)
- [ ] Background task: schedule weekly download and parse on Monday after market open
- [ ] `dark_pool_activity` table:
  ```sql
  CREATE TABLE dark_pool_activity (
      symbol          TEXT NOT NULL,
      week_ending     DATE NOT NULL,
      ats_volume      BIGINT NOT NULL,
      total_volume    BIGINT NOT NULL,
      ats_pct         NUMERIC NOT NULL,   -- dark pool % of total volume
      ats_pct_4w_avg  NUMERIC,            -- rolling 4-week average
      fetched_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
      PRIMARY KEY (symbol, week_ending)
  );
  ```
- [ ] Spike detection: `ats_pct > ats_pct_4w_avg × DARK_POOL_SPIKE_RATIO` (env,
  default 1.5) → flag as unusual institutional activity
- [ ] Score modifier (direction-neutral — dark pool signals institutional attention,
  not direction):
  - Spike + bullish TA signal → +5 composite score
  - Spike + short TA signal → +5 for the short (institutions positioning bearish)
  - No spike → no modifier
- [ ] Telegram alert on dark pool spike for any watchlist symbol, independent of scanner
- [ ] Dark pool context shown in Telegram notification when spike present:
  `"Dark pool: 31% vs 4w avg 18% — unusual institutional volume"`

### Testing

- [ ] Weekly CSV parsed correctly; `ats_pct` and `ats_pct_4w_avg` computed correctly
- [ ] Spike fires at correct threshold; no spike below threshold
- [ ] Score modifier applied for bullish and bearish setups
- [ ] Weekly import is idempotent (upsert on PRIMARY KEY)
- [ ] Download failure: logged, last week's data retained, no crash

---

## M-V2-09 — Open Trade Monitor (with restart-safe alert state)

### Todos

- [ ] Background task `TradeMonitor` using the `MarketDataProvider` from M3
- [ ] Poll open trades every `MONITOR_INTERVAL_SECS` (env, default 60)
- [ ] Per-trade checks on each tick:
  - [ ] Target hit → `closed_target`, `realized_pnl`, `closed_at`, Telegram alert
  - [ ] Stop hit → `closed_stop`, `realized_pnl`, `closed_at`, Telegram alert
  - [ ] +1R milestone (price moved 1R in favor) → Telegram alert
  - [ ] +2R milestone → Telegram alert
- [ ] **Restart-safe:** before each Telegram send, write the alert key to
  `trades.alerts_sent` JSONB; on boot, skip already-sent alerts
- [ ] Market-hours awareness: env-configurable trading sessions per asset class
  (`MARKET_HOURS_STOCKS`, `MARKET_HOURS_CRYPTO` = `always`); suppress price checks
  outside hours and on weekends for non-24h markets
- [ ] Provider failure: log error, back off exponentially, do not crash task
- [ ] Graceful shutdown: monitor task obeys shutdown channel

### Testing

- [ ] Target hit → `closed_target`, one alert sent
- [ ] Stop hit → `closed_stop`, one alert sent
- [ ] +1R / +2R fire exactly once each, even after process restart between them
- [ ] Closed trades are not polled
- [ ] Provider error does not kill the task
- [ ] Outside market hours: no price checks, no alerts

---

## M-V2-10 — Polymarket Integration

### Todos

- [ ] Polymarket REST API client (`POLYMARKET_API_KEY` env — public endpoints may not
  require auth)
- [ ] Define a curated list of markets relevant to trading (macro events, elections,
  Fed decisions, trade policy, crypto regulation)
- [ ] Store market snapshots in `prediction_market_events` table: `market_id`, `question`,
  `yes_probability`, `volume`, `fetched_at`
- [ ] Background task: poll relevant markets every `PM_POLL_INTERVAL_SECS` (default 300)
- [ ] Feed yes/no probabilities into signal scorer (Category: Prediction market, weight 5)
  - Example: "US tariff on chips increased?" at 70% probability → bearish modifier for
    NVDA-type signals
- [ ] Significant probability move (> `PM_ALERT_THRESHOLD` change in 1h) → Telegram alert

### Testing

- [ ] Market probability correctly mapped to signal score modifier
- [ ] Large probability shift triggers Telegram alert
- [ ] Provider error handled gracefully (cached last-known value used)

---

## M-V2-11 — Backtesting Enhancements (post-MVP additions)

> The core backtesting engine and validation gate are built in **M6.5** (MVP).
> This post-MVP milestone adds enhancements that become meaningful once real live
> trade data accumulates.

### Todos

- [ ] **Survivorship-bias correction:** ingest a list of historically delisted symbols
  (e.g. from a provider dataset or a manually maintained CSV) and include them in
  backtests to avoid testing only on survivors
- [ ] **Walk-forward optimization:** automated parameter sweep across the training window,
  selecting the parameter set with the best validation-window Sharpe; records the
  chosen parameters in `backtest_runs` for full reproducibility
- [ ] **Live vs. backtest divergence tracking:** after 60+ live trades, compare live
  win rate and avg-R to the OOS backtest values; alert if divergence exceeds 15%
  (possible sign of regime change or implementation bug)
- [ ] **Continuous rolling backtest:** re-run the walk-forward backtest monthly on the
  latest 3 years of data; store each run in `backtest_runs`; alert if a previously
  passing strategy no longer clears the gate

### Testing

- [ ] Delisted symbols included in universe produce lower win rates than survivor-only
  run (verified against a controlled test dataset)
- [ ] Divergence alert fires when synthetic live results differ from OOS by > 15%

---

## M-V2-12 — Market Regime Detection

### Todos

- [ ] Regime indicators computed on broad-market indices (SPY, QQQ, IWM):
  - [ ] SPY vs. 200-day SMA (above = bull, below = bear)
  - [ ] SPY 50-day SMA vs. 200-day SMA (golden cross / death cross)
  - [ ] ADX of SPY (> 25 = trending, < 20 = ranging)
  - [ ] VIX proxy (historical volatility of SPY as substitute if VIX data unavailable)
  - [ ] Breadth: % of S&P 500 symbols above their 50-day SMA (requires watchlist to include
    broad enough symbol set; can approximate with sector ETFs)
- [ ] `MarketRegime` enum: `StrongBull`, `Bull`, `Neutral`, `Bear`, `StrongBear`
- [ ] Regime computed once daily, stored in `market_regime` table
- [ ] Signal scorer: regime modifier applied to all signals
  - `StrongBull` → boost long signals, penalize short
  - `StrongBear` → boost short signals, penalize long
  - `Neutral` → no modifier
- [ ] Regime change → Telegram alert

### Testing

- [ ] SPY above 200 SMA + golden cross + ADX > 25 → `StrongBull`
- [ ] Regime modifier correctly adjusts composite signal scores
- [ ] Regime change triggers alert

---

## M-V2-13 — Reporting + Analytics

### Todos

- [ ] Daily Telegram summary (every trading day at configurable time):
  - [ ] Open trades with unrealized P&L
  - [ ] Today's signals: accepted / rejected counts
  - [ ] Current market regime
  - [ ] Top-scoring pending signal (if any)
- [ ] Weekly Telegram summary (every Monday morning):
  - [ ] Closed trades: win rate, average R, realized P&L
  - [ ] Best and worst trade
  - [ ] Strategy performance breakdown
  - [ ] Most-mentioned symbols in social feeds
- [ ] REST endpoints (all require `ADMIN_TOKEN` bearer auth):
  - [ ] `GET /api/signals/recent?limit=50`
  - [ ] `GET /api/trades/open`
  - [ ] `GET /api/trades/history?from=&to=`
  - [ ] `GET /api/stats` — aggregate win rate, expectancy, P&L
  - [ ] `GET /api/trades/export.csv`
- [ ] Pagination on all list endpoints (`cursor`-based)

### Testing

- [ ] Daily and weekly summaries contain correct data
- [ ] Endpoints reject requests without valid admin token (401)
- [ ] CSV export is parseable and contains all expected columns
- [ ] Pagination: fetching all pages yields same set as unpaginated (small test set)

---

## M-V2-14 — Short Interest + Analyst Ratings

> Options flow and insider activity were elevated to V2-02. This milestone covers the
> remaining institutional signal sources.

### Todos

- [ ] **Short interest:**
  - [ ] FINRA short interest data (released twice monthly via FINRA API or Polygon)
  - [ ] High short interest (> `SHORT_INTEREST_THRESHOLD` %, env default 15%) + bullish
    TA signal → elevated squeeze potential; boost composite score by 5
  - [ ] Store in `short_interest` table: `symbol, reported_date, short_interest_pct,
    days_to_cover, fetched_at`
  - [ ] Telegram alert when short interest crosses threshold for watchlist symbol

- [ ] **Analyst upgrades/downgrades:**
  - [ ] FMP analyst ratings endpoint — poll weekly
  - [ ] Cluster of ≥ 2 upgrades within 5 trading days → bullish modifier +4
  - [ ] Cluster of ≥ 2 downgrades within 5 trading days → bearish modifier −4
  - [ ] Store in `analyst_ratings` table: `symbol, analyst_firm, rating, price_target,
    action: Upgrade|Downgrade|Initiate, rated_at`
  - [ ] Telegram alert on any rating action for watchlist symbol

### Testing

- [ ] Short interest above threshold + bullish signal → score boost applied
- [ ] Analyst upgrade cluster → +4 modifier, Telegram alert fires
- [ ] Analyst downgrade cluster → −4 modifier
- [ ] Mock HTTP responses; all tables have integration tests

---

## M-V2-15 — Chart Pattern Detection

Chart patterns require more candles and are less reliable on short timeframes.
Implement as an add-on scorer, not a gate.

### Todos

- [ ] `detect_head_and_shoulders(candles)` → `Option<ChartPattern { bullish, key_levels }>`
- [ ] `detect_double_top(candles)` → `Option<ChartPattern>`
- [ ] `detect_double_bottom(candles)` → `Option<ChartPattern>`
- [ ] `detect_ascending_triangle(candles)` → `Option<ChartPattern>`
- [ ] `detect_descending_triangle(candles)` → `Option<ChartPattern>`
- [ ] `detect_symmetrical_triangle(candles)` → `Option<ChartPattern>`
- [ ] `detect_flag(candles)` → `Option<ChartPattern>` (bull and bear)
- [ ] `detect_cup_and_handle(candles)` → `Option<ChartPattern>`
- [ ] Pattern detections feed into signal scorer as a bonus category (weight up to 10)
- [ ] Pattern included in Telegram signal notification when detected

### Testing

- [ ] Each detector has hand-crafted candle sequences that trigger and do not trigger it
- [ ] Pattern scoring correctly raises/lowers composite score

---

## M-V2-16 — Advanced Risk Management

### Todos

- [ ] **Daily and weekly loss cap enforcement** (env vars from M7 now enforced):
  - [ ] Query realized P&L for today / this week from closed trades
  - [ ] If cap hit → block all new accepts; send Telegram alert
  - [ ] At 80% of cap → send warning alert
- [ ] **Correlation-aware risk:**
  - [ ] Rolling 30-day return correlation between open positions
  - [ ] If `|correlation| > 0.7`, warn user; do not block (configurable)
- [ ] **Sector concentration:**
  - [ ] Watchlist symbols tagged with GICS sector
  - [ ] Reject new trade if > `MAX_SECTOR_RISK_PCT` already exposed to same sector
- [ ] **Kelly Criterion (optional sizing mode):**
  - [ ] Compute fractional Kelly based on strategy win rate + avg R from backtest data
  - [ ] `SIZING_MODE=kelly` env var; default remains fixed-risk
- [ ] **Trailing stop suggestion:**
  - [ ] At +1R, suggest moving stop to breakeven via Telegram button
  - [ ] At +2R, suggest trail to +1R

### Testing

- [ ] Daily loss cap: trade rejected after cap is breached
- [ ] 80% warning fires correctly
- [ ] High correlation between two open trades → warning in Telegram
- [ ] Sector concentration block works

---

## M-V2-17 — Multi-Strategy Framework

### Todos

- [ ] Strategy configuration in `strategies/*.toml` files — loaded at startup
- [ ] Each strategy file specifies:
  - Indicator parameters
  - Timeframe combination
  - Score weights
  - Asset class applicability (`stocks_only`, `crypto_ok`, etc.)
  - Enabled/disabled flag
- [ ] `StrategyRegistry`: load all strategies, run applicable ones per symbol per tick
- [ ] Per-strategy performance tracking in DB (`strategy_performance` table):
  win rate, expectancy, trade count — updated on every trade close
- [ ] Telegram command `/strategies` — list enabled strategies with performance summary
- [ ] Hot-reload strategies on `SIGHUP` (no restart required)

### Testing

- [ ] Strategy disabled in TOML is not run
- [ ] Asset-class filter: crypto strategy not applied to stock symbols
- [ ] Performance metrics update correctly after trade close
- [ ] Hot-reload: updated TOML reflected without restart

---

## M-V2-18 — ML-Enhanced Signal Weighting

> **Data dependency: requires ≥ 12 months of paper or live trade data.**
> This is deliberately late — not because it's unimportant, but because training on
> < 200 trades produces a model that fits noise. Build it once you have the data.
> The feedback loop in V2-04 bridges the gap: adaptive multipliers without ML,
> using real outcome data, starting from month 1.

### Todos

- [ ] Feature extraction pipeline: for each historical signal in `signals` table,
  compute feature vector: `(score_breakdown fields, sector_rank, iv_rank,
  dark_pool_spike, news_sentiment_24h, social_buzz_4h, asset_class, market_regime,
  day_of_week, days_to_earnings)`
- [ ] Outcome label: `reached_target` boolean — price hit `take_profit` before `stop_loss`
  (computed from `trades` table or from candle history for signals that were rejected)
- [ ] Train a gradient-boosted classifier (logistic regression as baseline):
  `features → P(reach_target)` as the signal score (0–100)
- [ ] **A/B shadow mode:** `ML_SCORING_MODE=shadow` runs both old and new scorer in
  parallel; logs disagreements to `audit_log` tagged `ml.shadow_disagreement`; only
  old score used for actual notifications
- [ ] **Live mode:** `ML_SCORING_MODE=live` — ML confidence score replaces the hand-tuned
  composite score. Requires ≥ 500 OOS trades and validation accuracy ≥ 55%.
- [ ] Retrain weekly on rolling 12-month window; store model version + validation metrics
  in `ml_model_runs` table
- [ ] Alert if validation accuracy on most recent fold drops below 52% (near-random):
  `"⚠️ ML model accuracy degraded to 51.2% — reverting to static scoring"`
- [ ] `ML_SCORING_ENABLED` env var (default false)

### Testing

- [ ] Feature vector shape is deterministic for known signal input
- [ ] Shadow mode: both scores computed, disagreements logged, only old score governs
  notifications
- [ ] Accuracy below 52% threshold → alert fires, model not promoted to live
- [ ] Retraining on 500-trade synthetic dataset completes without error
- [ ] `ML_SCORING_ENABLED=false` → no model inference, static scoring used

---

## M-V2-19 — Security Hardening

### Todos

- [ ] Optional IP allowlist for all external-facing endpoints (`ALLOWED_IPS` env var)
- [ ] Database least-privilege user: separate `migration_user` (owner, DDL) and
  `app_user` (DML only — no DROP, no ALTER)
- [ ] Per-route rate limits tuned with real traffic data from logs
- [ ] `audit_log` retention policy: archive rows older than 90 days to a separate
  `audit_log_archive` table; export and purge older than 1 year
- [ ] Secret rotation runbook: documented procedure for rotating all API keys and tokens
  without downtime
- [ ] Secrets never logged: audit `tracing` callsites — any `?` on a type containing
  API keys must implement `Debug` with redaction

### Testing

- [ ] IP not on allowlist → 403
- [ ] App user cannot execute DDL (test with `DROP TABLE` attempt)
- [ ] Archived audit rows accessible via `audit_log_archive`
- [ ] No API key appears in any log line (capture and scan log output in test)

---

## M-V2-20 — TradingView Webhook (Optional Bonus Input)

TradingView can complement the bot's own scanner as a second opinion.

### Todos

- [ ] `POST /webhooks/tradingview` endpoint
- [ ] Payload validated with `subtle`-crate constant-time secret comparison
- [ ] `RequestBodyLimitLayer` (16 KB max)
- [ ] Per-IP rate limit via `tower_governor` (`SmartIpKeyExtractor`)
- [ ] Parsed signal injected into the same `signals` pipeline as scanner-generated signals,
  tagged `strategy = 'tradingview-<alert-name>'`
- [ ] Secret stripped from all log lines before emission

### Testing

- [ ] Valid webhook → signal stored and scored
- [ ] Wrong secret → 401, no DB write
- [ ] Oversized body → 413
- [ ] Rate limit: 11 fast requests → last one 429; parallel IP still 200
- [ ] Secret does not appear in any log output

---

## M-V2-21 — Polish + Admin Dashboard

### Todos

- [ ] Simple read-only admin web dashboard (`axum` + `askama` HTML templates or a
  minimal SPA) — protected by `ADMIN_TOKEN`
- [ ] Dashboard pages: open trades, signal history, strategy performance, watchlist,
  economic calendar, social sentiment heatmap
- [ ] CSV export for all major entities (signals, trades, audit log)
- [ ] Per-symbol notification muting (`/mute <SYMBOL> <HOURS>`)
- [ ] Snooze all notifications (`/snooze <MINUTES>`)
- [ ] Strategy parameter config UI (read-only view; writes still require TOML edit)

---

# Build Order at a Glance

```
MVP:
  M1   (skeleton)
  → M2   (DB + migrations, incl. backtest + earnings calendar tables)
  → M3   (market data provider)
  → M4   (candle ingest — back-fills 3 years of history on first boot)
  → M5   (TA engine — pure functions, no I/O)
  → M5.5 (early backtest validation — go/no-go gate; stop here if no edge)
  → M6   (scanner + event-driven strategies: EarningsGap, PreEarnings, ShortSqueeze)
  → M6.5 (full backtesting gate ← live_manual locked until this passes)
  → M7   (risk engine + fundamental quality gate)
  → M8   (Telegram notification + paper mode)
  → M9   (Telegram callbacks + trade execution)
  → M10  (production deploy)
  → MVP DONE → paper trade ≥ 3 months before enabling live_manual

V2 (priority-ordered — feedback loops and structural alpha first, then data enrichment):
  M-V2-01 (news sentiment — pre-filters bad-news trades, feeds LLM)
  → M-V2-02 (options flow + insider activity + IV rank ← leading indicators, high signal)
  → M-V2-03 (cross-sectional momentum + sector rotation ← new; computes over data you have)
  → M-V2-04 (strategy feedback loop ← new; dynamic weight suppression from live outcomes)
  → M-V2-05 (economic calendar)
  → M-V2-06 (social media signals — Reddit, StockTwits, Twitter/X)
  → M-V2-07 (LLM-enhanced analysis — synthesises all V2-01–06 data)
  → M-V2-08 (dark pool / FINRA ATS monitoring ← new; free, directionally useful)
  → M-V2-09 (open trade monitor)
  → M-V2-10 (Polymarket integration)
  → M-V2-11 (backtesting enhancements: survivorship bias, live divergence tracking)
  → M-V2-12 (market regime detection)
  → M-V2-13 (reporting + analytics)
  → M-V2-14 (short interest + analyst ratings)
  → M-V2-15 (chart pattern detection)
  → M-V2-16 (advanced risk management: daily loss caps, Kelly sizing, correlation)
  → M-V2-17 (multi-strategy framework: TOML-configured strategies, hot-reload)
  → M-V2-18 (ML-enhanced signal weighting ← new; requires ≥ 12 months data)
  → M-V2-19 (security hardening)
  → M-V2-20 (TradingView webhook — optional bonus input)
  → M-V2-21 (polish + admin dashboard)
```

After every MVP milestone: commit, run the full testing checklist, deploy to the
production stack from M10, verify end-to-end with a real watchlist symbol.
