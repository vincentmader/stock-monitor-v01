Here’s a Claude-ready implementation plan with milestone checklists and testing after each milestone.
# SwingBot Implementation Plan

## Goal

Build a Rust/Rocket-based swing trading assistant.

The system should:

- Receive TradingView webhook alerts
- Store signals in PostgreSQL
- Notify the user via Telegram
- Allow Accept/Reject via Telegram buttons
- Track manually executed trades
- Monitor open trades
- Send Telegram status updates
- Keep Trade Republic execution manual

No automatic Trade Republic order execution.

---

# Milestone 1 — Project Skeleton

## Todos

- [ ] Create Rust project `swingbot`
- [ ] Add Rocket
- [ ] Add `rocket_db_pools`
- [ ] Add SQLx PostgreSQL support
- [ ] Add `teloxide`
- [ ] Add `serde`, `serde_json`, `uuid`, `chrono`
- [ ] Add `rust_decimal`
- [ ] Create module layout:
  - [ ] `main.rs`
  - [ ] `db.rs`
  - [ ] `models.rs`
  - [ ] `routes.rs`
  - [ ] `telegram.rs`
  - [ ] `risk.rs`
  - [ ] `monitor.rs`
- [ ] Add `Rocket.toml`
- [ ] Add `.env.example`

## Testing

- [ ] `cargo check` passes
- [ ] `cargo clippy` passes
- [ ] `cargo test` passes
- [ ] Rocket starts locally
- [ ] `/health` returns `200 OK`

---

# Milestone 2 — PostgreSQL + Migrations

## Todos

- [ ] Add Docker Compose with PostgreSQL
- [ ] Add persistent PostgreSQL volume
- [ ] Add initial SQL migration
- [ ] Create `signals` table
- [ ] Create `trades` table
- [ ] Add indexes for:
  - [ ] signal status
  - [ ] trade status
  - [ ] trade symbol
- [ ] Use `rust_decimal`/SQL `NUMERIC` for prices
- [ ] Add database connection pool via `rocket_db_pools`

## Suggested tables

```sql
CREATE TABLE signals (
    id UUID PRIMARY KEY,
    symbol TEXT NOT NULL,
    side TEXT NOT NULL CHECK (side IN ('long', 'short')),
    price NUMERIC NOT NULL,
    stop_loss NUMERIC NOT NULL,
    take_profit NUMERIC NOT NULL,
    risk_reward NUMERIC NOT NULL,
    score INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL CHECK (
        status IN ('pending', 'accepted', 'rejected', 'expired')
    ),
    tradingview_time TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE trades (
    id UUID PRIMARY KEY,
    signal_id UUID REFERENCES signals(id),
    symbol TEXT NOT NULL,
    side TEXT NOT NULL CHECK (side IN ('long', 'short')),
    planned_entry_price NUMERIC NOT NULL,
    actual_entry_price NUMERIC,
    stop_loss NUMERIC NOT NULL,
    take_profit NUMERIC NOT NULL,
    status TEXT NOT NULL CHECK (
        status IN (
            'planned',
            'open',
            'closed_target',
            'closed_stop',
            'closed_manual',
            'invalidated'
        )
    ),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    opened_at TIMESTAMPTZ,
    closed_at TIMESTAMPTZ
);
Testing
 docker compose up postgres works
 migrations apply cleanly
 app connects to PostgreSQL
 test insert/select signal
 test insert/select trade
 invalid status fails because of SQL check constraint
Milestone 3 — TradingView Webhook Ingestion
Todos
 Add route POST /webhooks/tradingview
 Define payload model:
 symbol
 side
 price
 time
 optional strategy name
 optional timeframe
 Validate payload
 Reject invalid side
 Reject invalid price
 Reject missing symbol
 Add webhook secret validation
 Add duplicate signal prevention
 Calculate default:
 stop loss
 take profit
 risk/reward
 Store signal as pending
 Return JSON response
Testing
 valid webhook creates signal
 invalid JSON returns 400
 missing secret returns 401
 invalid side returns 400
 duplicate signal is ignored or marked duplicate
 DB row contains expected stop/target values
Milestone 4 — Telegram Notification
Todos
 Create Telegram bot client
 Load bot token from env
 Load Telegram chat id from env
 Implement send_text
 Implement send_signal
 Add inline buttons:
 Accept
 Reject
 Send Telegram message after valid TradingView signal
 Store Telegram message id if useful
Telegram message format
📈 Swing setup detected

Symbol: AAPL
Side: Long
Entry: 184.20
Stop: 174.99
Target: 211.83
R/R: 3.0
Score: 7/10

Execute manually in Trade Republic?
Testing
 mock Telegram client in unit tests
 webhook triggers Telegram send
 Telegram message contains symbol, entry, stop, target
 buttons contain correct callback payload
 failure to send Telegram message is logged
Milestone 5 — Telegram Callback Handling
Todos
 Add route POST /telegram/webhook
 Parse Telegram callback updates
 Parse callback data:
 accept:<signal_id>
 reject:<signal_id>
 On reject:
 mark signal as rejected
 send confirmation message
 On accept:
 mark signal as accepted
 create trade with status planned
 send follow-up asking whether trade was executed
 Add second-level callback:
 executed:<trade_id>
 cancel_trade:<trade_id>
Testing
 reject callback updates signal
 accept callback updates signal
 accept callback creates planned trade
 duplicate accept does not create duplicate trades
 unknown callback is safely ignored
 malformed UUID returns safe response
Milestone 6 — Manual Trade Execution Flow
Todos
 Add command/callback for “I executed the trade”
 Store actual entry price
 Support defaulting actual entry to planned entry
 Add endpoint or Telegram flow for manual price input
 Change trade status:
 planned → open
 Add manual close endpoint
 Add manual invalidate endpoint
Recommended flow
Signal detected
→ Accept
→ Planned trade created
→ User manually buys in Trade Republic
→ User confirms execution
→ User enters actual fill price
→ Trade becomes open
Testing
 planned trade can become open
 open trade stores actual entry price
 invalid price is rejected
 manual close sets closed_manual
 invalidated trade no longer appears in open trades
Milestone 7 — Risk Engine
Todos
 Add configurable account size
 Add risk percentage per trade
 Calculate position size
 Enforce minimum risk/reward
 Enforce maximum open trades
 Enforce maximum risk across open trades
 Prevent duplicate symbol exposure
 Optional: prevent correlated/sector exposure
 Add max daily loss placeholder
 Add max weekly loss placeholder
 Include risk result in Telegram signal message
Risk rules
max risk per trade: 0.5–1%
minimum R/R: 3.0
max open trades: 5
max total open risk: 5%
no duplicate open symbol
Testing
 position size is calculated correctly
 trade is rejected if R/R too low
 trade is rejected if max open trades reached
 duplicate symbol is rejected
 total open risk limit is enforced
 edge cases with zero/negative prices fail safely
Milestone 8 — Signal Scoring
Todos
 Add scoring model
 Score trend alignment
 Score EMA pullback
 Score RSI zone
 Score volume confirmation if provided
 Penalize extended price
 Penalize choppy market if provided
 Store score in DB
 Only notify if score >= threshold
Example score
+2 price above EMA200
+2 EMA20 above EMA50
+2 pullback to EMA20/50
+1 RSI between 40 and 65
+1 volume above average
-2 price too extended
Testing
 high-quality setup passes
 low-score setup is stored but not notified
 score is deterministic
 missing optional fields do not crash scoring
Milestone 9 — Market Data Provider
Todos
 Define MarketDataProvider trait
 Implement one provider initially
 Fetch latest price by symbol
 Handle provider errors
 Add timeout handling
 Add rate-limit handling
 Cache recent prices briefly
 Store latest checked price optionally
Trait sketch
#[async_trait::async_trait]
pub trait MarketDataProvider {
    async fn latest_price(&self, symbol: &str) -> anyhow::Result<rust_decimal::Decimal>;
}
Testing
 mock provider returns fixed prices
 timeout is handled
 invalid symbol is handled
 monitor can use mock provider
Milestone 10 — Trade Monitoring
Todos
 Add background monitoring task
 Poll open trades every N minutes
 Fetch latest price
 Detect target hit
 Detect stop hit
 Detect +1R
 Detect +2R
 Send Telegram updates
 Mark closed trades in DB
 Avoid duplicate alerts
Testing
 target hit closes trade as closed_target
 stop hit closes trade as closed_stop
 +1R alert sent once
 +2R alert sent once
 closed trades are not monitored
 provider failure does not crash task
Milestone 11 — Backtesting
Todos
 Extract strategy rules into pure functions
 Add candle model
 Add CSV input
 Simulate EMA20/EMA50/EMA200
 Simulate RSI
 Simulate entries
 Simulate stop/target exits
 Include fees
 Include slippage
 Output metrics:
 win rate
 average R
 profit factor
 max drawdown
 expectancy
 longest losing streak
Testing
 indicator calculations are tested
 known candle sequence triggers expected signal
 stop hit before target is handled correctly
 target hit before stop is handled correctly
 fees/slippage affect result
 metrics are calculated correctly
Milestone 12 — Paper Mode / Live Manual Mode
Todos
 Add config value TRADING_MODE
 Support paper
 Support live_manual
 In paper mode:
 auto-open accepted paper trades
 simulate closes
 In live manual mode:
 require user execution confirmation
 Add Telegram label showing mode
Testing
 paper mode does not require manual execution
 live manual mode requires confirmation
 mode is shown in Telegram messages
 changing mode does not require code change
Milestone 13 — Deployment
Todos
 Add production Dockerfile
 Add Docker Compose for app + PostgreSQL
 Add .env.production
 Add HTTPS reverse proxy
 Add domain/subdomain
 Add Telegram webhook setup script
 Add TradingView webhook URL
 Add database backup script
 Add restart policy
 Add logs
 Add /health
 Add /ready
Testing
 container builds successfully
 app starts on server
 PostgreSQL persists after restart
 HTTPS endpoint works
 TradingView test alert reaches app
 Telegram message arrives
 app survives restart
 backup script produces usable dump
Milestone 14 — Security Hardening
Todos
 Require webhook secret
 Rate-limit webhook endpoint
 Validate request size
 Reject unknown symbols if symbol allowlist is enabled
 Add IP allowlist for TradingView webhook IPs if possible
 Do not log secrets
 Use environment variables for all secrets
 Add database least-privilege user
 Add structured error logging
 Add audit log table
Testing
 missing secret rejected
 wrong secret rejected
 oversized body rejected
 invalid payload rejected
 secrets do not appear in logs
 audit log receives critical events
Milestone 15 — Polish
Todos
 Add daily Telegram summary
 Add weekly performance summary
 Add /signals/recent
 Add /trades/open
 Add /trades/history
 Add simple admin dashboard later
 Add CSV export
 Add config file for strategy parameters
 Add symbol watchlist
 Add strategy versioning
Testing
 summaries include correct open/closed trades
 recent signals endpoint works
 open trades endpoint works
 CSV export works
 strategy config reload works or fails safely
Definition of Done
The prototype is usable when:
 TradingView alert reaches Rocket
 Signal is validated and stored
 Telegram notification is sent
 User can accept/reject
 Accepted signal creates planned trade
 User can mark trade as executed
 Open trade is monitored
 Stop/target updates are sent
 Trade history is persisted
 System runs in Docker on remote server
 Basic tests pass
 Trade Republic execution remains manual
