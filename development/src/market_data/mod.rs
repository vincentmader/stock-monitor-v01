use async_trait::async_trait;
use chrono::NaiveDate;

use crate::models::{AssetClass, Candle};

/// Common interface for all market-data backends (Polygon, CoinGecko, …).
/// Implemented per-provider in M3; used by the candle ingestor in M4.
#[async_trait]
pub trait MarketDataProvider: Send + Sync + 'static {
    /// Fetch OHLCV candles for `symbol` between `from` and `to` (inclusive).
    /// Prices must be split/dividend-adjusted.
    async fn candles(
        &self,
        symbol: &str,
        asset_class: AssetClass,
        timeframe: &str,
        from: NaiveDate,
        to: NaiveDate,
    ) -> anyhow::Result<Vec<Candle>>;

    /// Return the current ask price for a symbol (used for live mode slippage estimates).
    async fn latest_price(&self, symbol: &str, asset_class: AssetClass) -> anyhow::Result<rust_decimal::Decimal>;
}
