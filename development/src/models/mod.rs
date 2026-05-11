use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── core value types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol(pub String);

impl Symbol {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into().to_uppercase())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AssetClass {
    Stock,
    Etf,
    Crypto,
    Reit,
    Commodity,
    Index,
}

impl std::fmt::Display for AssetClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AssetClass::Stock => "stock",
            AssetClass::Etf => "etf",
            AssetClass::Crypto => "crypto",
            AssetClass::Reit => "reit",
            AssetClass::Commodity => "commodity",
            AssetClass::Index => "index",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Long,
    Short,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SignalStatus {
    Pending,
    Accepted,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TradeStatus {
    Planned,
    Open,
    ClosedTarget,
    ClosedStop,
    ClosedManual,
    Invalidated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TradingMode {
    Paper,
    LiveManual,
}

// ── market data ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub symbol: String,
    pub timeframe: String,
    pub ts: DateTime<Utc>,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
}

// ── signals & trades ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub id: Uuid,
    pub symbol: String,
    pub asset_class: AssetClass,
    pub side: Side,
    pub entry_price: Decimal,
    pub stop_loss: Decimal,
    pub take_profit: Decimal,
    pub risk_reward: Decimal,
    pub score: i32,
    pub score_breakdown: serde_json::Value,
    pub strategy: String,
    pub timeframe: String,
    pub status: SignalStatus,
    pub telegram_msg_id: Option<i64>,
    pub raw_context: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: Uuid,
    pub signal_id: Uuid,
    pub symbol: String,
    pub asset_class: AssetClass,
    pub side: Side,
    pub planned_entry_price: Decimal,
    pub actual_entry_price: Option<Decimal>,
    pub quantity: Option<Decimal>,
    pub stop_loss: Decimal,
    pub take_profit: Decimal,
    pub currency: String,
    pub fees: Decimal,
    pub realized_pnl: Option<Decimal>,
    pub scale_out_price: Option<Decimal>,
    pub scale_out_qty: Option<Decimal>,
    pub scale_out_at: Option<DateTime<Utc>>,
    pub scale_out_elected: bool,
    pub status: TradeStatus,
    pub alerts_sent: serde_json::Value,
    pub trading_mode: TradingMode,
    pub created_at: DateTime<Utc>,
    pub opened_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
}

// ── timeframe constants ───────────────────────────────────────────────────────

pub mod timeframe {
    pub const DAILY: &str = "1d";
    pub const FOUR_HOUR: &str = "4h";
    pub const ONE_HOUR: &str = "1h";
    pub const FIFTEEN_MIN: &str = "15m";
    pub const FIVE_MIN: &str = "5m";
}
