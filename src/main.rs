use std::fmt::format;
use axum::{
    Json, 
    Router, 
    http::{StatusCode}, 
    response::{IntoResponse, Response}, 
    routing::{get, post}
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

#[derive(Deserialize)]
struct AuthRequest{
    email: String,
    password: String
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub message: String,
    #[serde(skip_serializing)]
    pub status: StatusCode
}

impl IntoResponse for AuthResponse { //This is to implement IntoResponse functionality so that axum can use it in http body
    fn into_response(self) -> Response {
        let body = Json(serde_json::json!({"Message": self.message}));
        (self.status, body).into_response()
    }
}

impl AuthResponse{ //we are implementing factory methods on this Struct.

    pub fn created(msg: impl Into<String>) -> Self {
        Self { 
            message: msg.into(), 
            status: StatusCode::CREATED
        }
    }

    pub fn ok(msg: impl Into<String>) -> Self{
        Self {
            message: msg.into(),
            status: StatusCode::OK,
        }
    }

    pub fn unauthorised(msg: impl Into<String>) -> Self{
        Self {
             message: msg.into(), 
             status: StatusCode::UNAUTHORIZED }
    }

}

async fn signup_function(Json(payload): Json<AuthRequest>) -> AuthResponse{
    println!("Sign Up Requset {} / {}", payload.email, payload.password);

    // let response = AuthResponse{
    //     message: format!("User {} created Successfully !", payload.email),
    // };
    // (StatusCode::CREATED, Json(response))
    AuthResponse::created(format!("User {} Created Successfull", payload.email))
}


#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route("/", post(|| async{"Hello World!"}))
    .route("/signup", post(signup_function));
    // .route("/signin", post(signin_function))
    // .route("/onramp", post(onramp_function))
    // .route("/create_limit_order", post(create_limit_order_function))
    // .route("/create_market_order", post(create_market_order_function))
    // .route("/get_orderbook", get(get_orderbook_function));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap(); //use anyhow
}

// async function signin_function(){
    

// }

// async function onramp_function(){
    

// }

// async function create_limit_order_function(){
    

// }

// async function create_market_order_function(){
    

// }

// async function get_orderbook_function(){
    

// }
