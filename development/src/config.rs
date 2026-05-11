use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    // ── server ────────────────────────────────────────────────────────────────
    pub host: String,
    pub port: u16,
    pub rust_log: String,

    // ── database ──────────────────────────────────────────────────────────────
    pub database_url: String,

    // ── telegram (used from M8) ───────────────────────────────────────────────
    pub telegram_bot_token: Option<String>,
    pub telegram_chat_id: Option<i64>,
    pub telegram_webhook_secret: Option<String>,

    // ── market data ───────────────────────────────────────────────────────────
    pub polygon_api_key: Option<String>,
    pub coingecko_api_key: Option<String>,
    pub alpha_vantage_api_key: Option<String>,

    // ── trading ───────────────────────────────────────────────────────────────
    pub trading_mode: TradingMode,
    pub allow_short_signals: bool,
    pub account_size: Option<rust_decimal::Decimal>,
    pub account_currency: String,
    pub risk_per_trade_pct: rust_decimal::Decimal,
    pub min_risk_reward: rust_decimal::Decimal,
    pub max_open_trades: u32,
    pub max_total_open_risk_pct: rust_decimal::Decimal,
    pub fractional_shares: bool,
    pub score_threshold: i32,

    // ── scanner ───────────────────────────────────────────────────────────────
    pub scan_blackout_open_mins: u32,
    pub scan_blackout_close_mins: u32,
    pub min_adv_usd: rust_decimal::Decimal,
    pub min_symbol_price: rust_decimal::Decimal,

    // ── scale-out ─────────────────────────────────────────────────────────────
    pub scale_out_enabled: bool,

    // ── watchdog ──────────────────────────────────────────────────────────────
    pub watchdog_check_secs: u64,
    pub watchdog_stale_secs: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradingMode {
    Paper,
    LiveManual,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            host: var_or("HOST", "0.0.0.0"),
            port: var_or("PORT", "3000")
                .parse()
                .context("PORT must be a valid port number (0–65535)")?,
            rust_log: var_or("RUST_LOG", "swingbot=info"),

            database_url: require("DATABASE_URL")?,

            telegram_bot_token: opt("TELEGRAM_BOT_TOKEN"),
            telegram_chat_id: opt("TELEGRAM_CHAT_ID")
                .map(|s| s.parse().context("TELEGRAM_CHAT_ID must be an integer"))
                .transpose()?,
            telegram_webhook_secret: opt("TELEGRAM_WEBHOOK_SECRET"),

            polygon_api_key: opt("POLYGON_API_KEY"),
            coingecko_api_key: opt("COINGECKO_API_KEY"),
            alpha_vantage_api_key: opt("ALPHA_VANTAGE_API_KEY"),

            trading_mode: match var_or("TRADING_MODE", "paper").as_str() {
                "live_manual" => TradingMode::LiveManual,
                _ => TradingMode::Paper,
            },
            allow_short_signals: var_bool("ALLOW_SHORT_SIGNALS", false),
            account_size: opt("ACCOUNT_SIZE")
                .map(|s| s.parse().context("ACCOUNT_SIZE must be a decimal number"))
                .transpose()?,
            account_currency: var_or("ACCOUNT_CURRENCY", "EUR"),
            risk_per_trade_pct: var_decimal("RISK_PER_TRADE_PCT", "1.0")?,
            min_risk_reward: var_decimal("MIN_RISK_REWARD", "3.0")?,
            max_open_trades: var_or("MAX_OPEN_TRADES", "5")
                .parse()
                .context("MAX_OPEN_TRADES must be a positive integer")?,
            max_total_open_risk_pct: var_decimal("MAX_TOTAL_OPEN_RISK_PCT", "5.0")?,
            fractional_shares: var_bool("FRACTIONAL_SHARES", true),
            score_threshold: var_or("SCORE_THRESHOLD", "60")
                .parse()
                .context("SCORE_THRESHOLD must be an integer 0–100")?,

            scan_blackout_open_mins: var_or("SCAN_BLACKOUT_OPEN_MINS", "30")
                .parse()
                .context("SCAN_BLACKOUT_OPEN_MINS must be a non-negative integer")?,
            scan_blackout_close_mins: var_or("SCAN_BLACKOUT_CLOSE_MINS", "30")
                .parse()
                .context("SCAN_BLACKOUT_CLOSE_MINS must be a non-negative integer")?,
            min_adv_usd: var_decimal("MIN_ADV_USD", "1000000")?,
            min_symbol_price: var_decimal("MIN_SYMBOL_PRICE", "1.0")?,

            scale_out_enabled: var_bool("SCALE_OUT_ENABLED", true),

            watchdog_check_secs: var_or("WATCHDOG_CHECK_SECS", "120")
                .parse()
                .context("WATCHDOG_CHECK_SECS must be a positive integer")?,
            watchdog_stale_secs: var_or("WATCHDOG_STALE_SECS", "300")
                .parse()
                .context("WATCHDOG_STALE_SECS must be a positive integer")?,
        })
    }
}

fn require(key: &str) -> Result<String> {
    std::env::var(key).with_context(|| format!("missing required env var: {key}"))
}

fn opt(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|s| !s.is_empty())
}

fn var_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn var_bool(key: &str, default: bool) -> bool {
    std::env::var(key)
        .map(|v| matches!(v.to_lowercase().as_str(), "true" | "1" | "yes"))
        .unwrap_or(default)
}

fn var_decimal(key: &str, default: &str) -> Result<rust_decimal::Decimal> {
    var_or(key, default)
        .parse()
        .with_context(|| format!("{key} must be a decimal number"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_parse_without_env() {
        // DATABASE_URL is required; set a dummy for the parse test
        std::env::set_var("DATABASE_URL", "postgres://localhost/test");
        let cfg = Config::from_env().unwrap();
        assert_eq!(cfg.port, 3000);
        assert_eq!(cfg.account_currency, "EUR");
        assert!(!cfg.allow_short_signals);
        assert!(cfg.scale_out_enabled);
        std::env::remove_var("DATABASE_URL");
    }
}
