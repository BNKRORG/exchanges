pub(super) enum BinanceApi {
    Spot(Spot),
}

impl BinanceApi {
    pub(super) fn http_path(&self) -> &str {
        match self {
            Self::Spot(spot) => spot.http_path(),
        }
    }

    pub(super) fn request_weight(&self) -> u32 {
        match self {
            Self::Spot(spot) => spot.request_weight(),
        }
    }
}

pub(super) enum Spot {
    // Ping,
    // Time,
    ExchangeInfo,
    // Depth,
    // Trades,
    // HistoricalTrades,
    // AggTrades,
    // Klines,
    // AvgPrice,
    // Ticker24hr,
    // Price,
    // BookTicker,
    // Order,
    // OrderTest,
    // OpenOrders,
    // AllOrders,
    // Oco,
    // OrderList,
    // AllOrderList,
    // OpenOrderList,
    Account,
    MyTrades,
    // UserDataStream,
}

impl Spot {
    pub(super) fn http_path(&self) -> &str {
        match self {
            // Self::Ping => "/api/v3/ping",
            // Self::Time => "/api/v3/time",
            Self::ExchangeInfo => "/api/v3/exchangeInfo",
            // Self::Depth => "/api/v3/depth",
            // Self::Trades => "/api/v3/trades",
            // Self::HistoricalTrades => "/api/v3/historicalTrades",
            // Self::AggTrades => "/api/v3/aggTrades",
            // Self::Klines => "/api/v3/klines",
            // Self::AvgPrice => "/api/v3/avgPrice",
            // Self::Ticker24hr => "/api/v3/ticker/24hr",
            // Self::Price => "/api/v3/ticker/price",
            // Self::BookTicker => "/api/v3/ticker/bookTicker",
            // Self::Order => "/api/v3/order",
            // Self::OrderTest => "/api/v3/order/test",
            // Self::OpenOrders => "/api/v3/openOrders",
            // Self::AllOrders => "/api/v3/allOrders",
            // Self::Oco => "/api/v3/order/oco",
            // Self::OrderList => "/api/v3/orderList",
            // Self::AllOrderList => "/api/v3/allOrderList",
            // Self::OpenOrderList => "/api/v3/openOrderList",
            Self::Account => "/api/v3/account",
            Self::MyTrades => "/api/v3/myTrades",
            // Self::UserDataStream => "/api/v3/userDataStream",
        }
    }

    pub(super) fn request_weight(&self) -> u32 {
        match self {
            Self::ExchangeInfo | Self::Account | Self::MyTrades => 20,
        }
    }
}
