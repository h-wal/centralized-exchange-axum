use std::collections::{BTreeMap, VecDeque};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::json;
use crate::domain::{Order, Trade};

/// Used by `/signup` and `/signin` routes
#[derive(Serialize)]
pub struct AuthResponse {
    pub message: String,
    #[serde(skip_serializing)]
    pub status: StatusCode,
}

impl IntoResponse for AuthResponse {
    fn into_response(self) -> Response {
        let body = Json(json!({ "message": self.message }));
        (self.status, body).into_response()
    }
}

impl AuthResponse {
    pub fn created(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            status: StatusCode::CREATED,
        }
    }

    pub fn ok(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            status: StatusCode::OK,
        }
    }

    pub fn unauthorised(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            status: StatusCode::UNAUTHORIZED,
        }
    }

    pub fn internal_server_error(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Used by `/onramp` route
#[derive(Serialize)]
pub struct OnRampResponse {
    pub message: String,
    pub new_balance: u64,
    pub new_holdings: u64,
    #[serde(skip_serializing)]
    pub status: StatusCode,
}

impl IntoResponse for OnRampResponse {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "message": self.message,
            "new_balance": self.new_balance,
            "new_holdings": self.new_holdings
        }));
        (self.status, body).into_response()
    }
}

impl OnRampResponse {
    pub fn ok(msg: impl Into<String>, new_balance: u64, new_holdings: u64) -> Self {
        Self {
            message: msg.into(),
            new_balance,
            new_holdings,
            status: StatusCode::ACCEPTED,
        }
    }

    pub fn err(msg: impl Into<String>, new_balance: u64, new_holdings: u64) -> Self {
        Self {
            message: msg.into(),
            new_balance,
            new_holdings,
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Serialize)]
pub struct CreateMarketOrderResponse {
    pub message: String,
    pub trades: Vec<Trade>,
    #[serde(skip_serializing)]
    pub status: StatusCode,
}

impl IntoResponse for CreateMarketOrderResponse {
    fn into_response(self) -> Response {
        let body = Json(serde_json::json!({
            "message": self.message,
            "trades": self.trades
        }));
        (self.status, body).into_response()
    }
}

#[derive(Serialize)]
pub struct GetOrderBookResponse {
    pub message: String,
    pub bids: Option<BTreeMap<u64, VecDeque<Order>>>,
    pub asks: Option<BTreeMap<u64, VecDeque<Order>>>,
    #[serde(skip_serializing)]
    pub status: StatusCode,
}

impl IntoResponse for GetOrderBookResponse {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "message": self.message,
            "bids": self.bids,
            "asks": self.asks
        }));
        (self.status, body).into_response()
    }
}

