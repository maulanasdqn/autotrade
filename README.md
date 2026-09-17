# AutoTrade IDX

Automated stock trading platform for the Indonesian Stock Exchange (IDX), connected to a real Stockbit Sekuritas brokerage account. Built with Rust.

## Features

- **Portfolio Dashboard** — Real-time portfolio overview with total equity, cash balance, positions, unrealized P&L, and open orders from your Stockbit account
- **Stock Analysis** — AI-powered stock analysis using Yahoo Finance market data and DeepSeek AI, with buy/sell/hold signals, entry points, and risk factors
- **Auto Trading** — Rule-based automated trading that monitors prices and places real buy/sell orders on Stockbit when conditions are met
- **Auto-Run** — Continuously checks your rules against live prices every 5 minutes (toggleable from the Trades page)
- **Trade Rules** — Configure per-stock rules with buy-below, sell-above (take-profit), stop-loss, and max lot parameters
- **AI Suggestions** — AI-generated stock recommendations with confidence scores and reasoning
- **Token Management** — Automatic Stockbit JWT token refresh with expiry tracking

## Architecture

Rust workspace with two crates following clean architecture:

```
autotrade/
├── autotrade-api/          # Backend (Axum)
│   └── src/
│       ├── domain/         # Entities, ports, value objects, errors
│       ├── application/    # Use cases (analyze, autotrade, suggest)
│       ├── infrastructure/ # External adapters
│       │   ├── stockbit/   # Stockbit Sekuritas broker API (trading, portfolio)
│       │   ├── yahoo/      # Yahoo Finance market data (prices, history)
│       │   ├── ai/         # DeepSeek AI integration (analysis, suggestions)
│       │   ├── idx/        # IDX market data (legacy, Cloudflare-blocked)
│       │   ├── broker/     # Simulated broker for testing
│       │   └── repository/ # In-memory repositories
│       └── presentation/   # HTTP handlers, router, app state
│
├── autotrade-web/          # Frontend (Leptos 0.7 WASM)
│   └── src/
│       ├── pages/          # Dashboard, Analysis, Trades, Rules
│       ├── components/     # Reusable UI components
│       ├── api.rs          # HTTP client for backend API
│       └── dto.rs          # Response types
│
├── Cargo.toml              # Workspace config
└── rust-toolchain.toml     # Stable Rust + wasm32-unknown-unknown target
```

### Key Design Decisions

- **Domain ports** define interfaces (`MarketDataPort`, `BrokerPort`, `AiPort`, `TradeRuleRepository`) that infrastructure adapters implement
- **Yahoo Finance** for market data — IDX website blocks automated requests with Cloudflare challenges; Yahoo Finance v8 chart API works without auth for `.JK`-suffixed Indonesian stocks
- **Stockbit v2 API** at `carina.stockbit.com` for brokerage operations — uses nested JSON objects (e.g., `qty.balance.lot`, `price.average.price`) with flat field fallbacks for compatibility
- **Client-side routing** with History API (`pushState`/`popState`) for clean URLs (`/trades`, `/analysis`, `/rules`)

## Prerequisites

- Rust stable toolchain with `wasm32-unknown-unknown` target
- `wasm-bindgen-cli` (`cargo install wasm-bindgen-cli`)
- A Stockbit Sekuritas account with an active trading session

## Setup

1. Clone the repository:
   ```bash
   git clone git@github.com:maulanasdqn/autotrade.git
   cd autotrade
   ```

2. Create a `.env` file in the project root:
   ```env
   STOCKBIT_TOKEN=<your-stockbit-jwt-token>
   AI_API_URL=https://api.deepseek.com
   AI_API_KEY=<your-deepseek-api-key>
   AI_MODEL=deepseek-chat
   PORT=3000
   ```

   **Getting your Stockbit token:**
   1. Log in to [Stockbit](https://stockbit.com) in your browser
   2. Open DevTools → Network tab
   3. Find any request to `carina.stockbit.com`
   4. Copy the `Authorization: Bearer <token>` value

3. Build the frontend:
   ```bash
   cargo build --target wasm32-unknown-unknown -p autotrade-web --release
   wasm-bindgen --out-dir autotrade-web/dist --target web \
     target/wasm32-unknown-unknown/release/autotrade-web.wasm
   cp autotrade-web/style.css autotrade-web/dist/style.css
   ```

4. Run the backend:
   ```bash
   cargo run -p autotrade-api
   ```

5. Open `http://localhost:3000` in your browser.

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/health` | Health check |
| `GET` | `/api/v1/portfolio` | Get portfolio (positions, equity, open orders) |
| `GET` | `/api/v1/suggest` | Get AI stock suggestions |
| `POST` | `/api/v1/analyze/{symbol}` | Run AI analysis on a stock |
| `POST` | `/api/v1/autotrade` | Execute auto-trade rules against live prices |
| `GET` | `/api/v1/rules` | List active trade rules |
| `POST` | `/api/v1/rules` | Create a trade rule |
| `DELETE` | `/api/v1/rules/{symbol}` | Delete a trade rule |
| `GET` | `/api/v1/token/status` | Check Stockbit token status |
| `PUT` | `/api/v1/token` | Update Stockbit token |

## Auto-Trade Logic

The auto-trade engine evaluates each rule against live market data:

1. **Buy signal** — If you have no position in the stock and the current price is at or below `buy_below`, it places a buy order for `max_lot` lots
2. **Take-profit** — If you hold the stock and the price reaches `sell_above`, it sells your entire position
3. **Stop-loss** — If you hold the stock and the price drops to `stop_loss`, it sells to limit downside

**Important:** This places real orders on your Stockbit account. Use with caution and always verify your rules before enabling auto-run.

## Tech Stack

- **Backend:** Rust, Axum, Tokio, Reqwest, Serde
- **Frontend:** Leptos 0.7 (CSR), WASM, gloo-net
- **Market Data:** Yahoo Finance v8 Chart API
- **Broker:** Stockbit Sekuritas v2 API
- **AI:** DeepSeek Chat API

## License

MIT
