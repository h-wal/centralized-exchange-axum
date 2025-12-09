# Order Books (Rust)

An Axum-based demo of a central limit order book service with simple user auth, in-memory balances, and an actor-style orderbook engine. Built for portfolio/showcase purposes (not production-ready).

## Features
- HTTP API for signup/signin, mock onramp, market creation/listing, limit & market orders, cancel, and order book snapshot.
- In-memory matching engine with price/qty checks and partial fill handling; FIFO within price levels via `VecDeque`.
- Actor-style separation: orderbook actor for matching; DB actor for users/balances and reconciliation.
- Lightweight DTOs with JSON responses using Axum.

## Architecture
- `src/main.rs` boots Axum and wires the shared `AppState` with channels to the actors.
- `actors/orderbook.rs` keeps `MarketBook` state per market, processes order commands, calls DB reconciliation.
- `actors/db.rs` mocks a user store (signup/signin, balance/holdings, reconciliation).
- `domain/*` models: `Order`, `Trade`, `MarketBook`, `User`.
- `handlers/*` map HTTP routes to actor commands.

## API (paths relative to `http://0.0.0.0:4000`)
- `POST /signup` – `{ email, password }`
- `POST /signin` – `{ email, password }`
- `POST /onramp` – `{ user_email, balance, holding }` (adds to in-memory balances)
- `POST /createmarket` – `{ market_id }`
- `POST /listmarkets` – no body
- `POST /createLimitOrder` – `{ market_id, user_email, order: { qty, price, side } }`
- `POST /createMarketOrder` – `{ market_id, user_email, order: { qty, price, side } }`
- `POST /getorderbook` – `{ user_email, market_id }`
- `POST /cancelorder` – `{ market_id, side, order_id }`

`side` is `"Bid"` or `"Ask"`.

## Quick start
```bash
cargo run
# Server runs on http://0.0.0.0:4000
```

### Minimal demo script (example)
```bash
# 1) Sign up and sign in
curl -X POST localhost:4000/signup -d '{"email":"alice@test.com","password":"pw"}' -H "Content-Type: application/json"
curl -X POST localhost:4000/signin -d '{"email":"alice@test.com","password":"pw"}' -H "Content-Type: application/json"

# 2) Fund account
curl -X POST localhost:4000/onramp -d '{"user_email":"alice@test.com","balance":10000,"holding":0}' -H "Content-Type: application/json"

# 3) Create market + place limit order
curl -X POST localhost:4000/createmarket -d '{"market_id":1}' -H "Content-Type: application/json"
curl -X POST localhost:4000/createLimitOrder -d '{"market_id":1,"user_email":"alice@test.com","order":{"qty":5,"price":100,"side":"Bid"}}' -H "Content-Type: application/json"

# 4) View book
curl -X POST localhost:4000/getorderbook -d '{"user_email":"alice@test.com","market_id":1}' -H "Content-Type: application/json"
```

## Notes
- State is in-memory;
- Matching is best-effort with price/qty checks; per-price FIFO.

