use std::collections::HashMap; //used to store the users array in memory
use axum::{
    Json, 
    Router, 
    http::{StatusCode, response, status}, 
    response::{IntoResponse, Response}, 
    routing::post
};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};
use axum::extract::State;

type DbSender = mpsc::Sender<DbCommand>;


#[derive(Deserialize)]
struct AuthRequest{     //namefeild struct  -   "semantic name - this means every feild of data has some meaning
    email: String,
    password: String
}

struct Wrapper<T>(T); // This is called a tuple struct - 

#[derive(Serialize)]
pub struct AuthResponse {
    pub message: String,
    #[serde(skip_serializing)]
    pub status: StatusCode
}

impl IntoResponse for AuthResponse {        //This is to implement IntoResponse functionality so that axum can use it in http body
    fn into_response(self) -> Response {
        let body = Json(serde_json::json!({"Message": self.message}));
        (self.status, body).into_response()
    }
}

impl AuthResponse{                          //we are implementing factory methods on this Struct.

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

    pub fn internal_server_error(msg: impl Into<String>) -> Self{
        Self {
             message: msg.into(), 
             status: StatusCode::INTERNAL_SERVER_ERROR }
    }

}

#[derive(Serialize)]
pub struct OnRampResponse {
    pub message: String,
    pub new_balance: u64,
    #[serde(skip_serializing)] //we are skipping serializing this status as it can be directly infered by the tcp 
    pub status: StatusCode
}

impl IntoResponse for OnRampResponse {        //This is to implement IntoResponse functionality so that axum can use it in http body
    fn into_response(self) -> Response {
        let body = Json(serde_json::json!({"Message": self.message}));
        (self.status, body).into_response()
    }
}

impl OnRampResponse{
     
     pub fn ok(msg: impl Into<String>, new_balance: u64) -> Self{
        Self { message: msg.into() , new_balance: new_balance, status:StatusCode::ACCEPTED }
     }

     pub fn err(msg: impl Into<String>, new_balance: u64) -> Self{
        Self { message: msg.into(), new_balance, status: status::StatusCode::INTERNAL_SERVER_ERROR }
     }
}
struct SignupResponseType {
    status: String
}

struct SigninResponseType {
    status: String,
    // token: Option<String>
}


enum DbCommand {
    Signup { 
        email: String, 
        password: String, 
        response_status: oneshot::Sender<SignupResponseType> //used to get the status of the request which was sent via mpsc
    },
    Signin { 
        email: String, 
        password: String,
        response_status: oneshot::Sender<SigninResponseType>
    },
    OnRamp {
        delta_balance: u64,
        user_email: String,
        response_status: oneshot::Sender<OnRampDbResponseType>
    }
    // UpdateBalance { email: String, amount: u32 },
}

struct Order{
    qty: u32,
    price: u32
}
struct User{
    email: String,
    password: String,
    balance: u64,
    holding: u32
}

struct OnRampDbResponseType{
    status: String,
    balance: u64
}

#[derive(Deserialize)]
pub struct OnRampHttpRequest{
    pub user_email: String,
    pub balance: u64
}

// impl IntoResponse for OnRampHttpRequest {        //This is to implement IntoResponse functionality so that axum can use it in http body
//     fn into_response(self) -> Response {
//         let body = Json(serde_json::json!({"Message": self.message}));
//         (self.status, body).into_response()
//     }
// }


async fn user_db_actor(mut rx: mpsc::Receiver<DbCommand>){
    let mut users: HashMap<String, User> = HashMap::new();

    println!("UserDBActor started...");

    // Infinite loop — actor waits for incoming messages
    while let Some(cmd) = rx.recv().await {
        match cmd {
            DbCommand::Signup { email, password, response_status } => {
                if users.contains_key(&email) {
                    println!("User '{}' already exists!", email);
                    let response = SignupResponseType {
                        status: "User already exists".to_string(),
                    };
                    let _ = response_status.send(response);
                } else {
                    let user = User {
                        email: email.clone(),
                        password,
                        balance: 0,
                        holding: 0
                    };
                    users.insert(email.clone(), user);
                    let _ = response_status.send(SignupResponseType {
                        status: "User Created Successfully ".to_string(),
                    });
                    println!(" User '{}' added successfully!", email);
                }
            },
            DbCommand::Signin { email, password, response_status } => {
                let response= if let Some(user) = users.get(&email) {
                    if user.password == password {
                        println!("User '{}' authenticated successfully!", email);
                        SigninResponseType {
                            status: "User Authenticated".to_string()
                        }
                    } else {
                        println!(" Incorrect password for '{}'", email);
                        SigninResponseType{
                            status: "Incorrect Password".to_string()
                        }
                    }
                } else {
                    println!(" User '{}' not found, please sign up!", email);
                    SigninResponseType{
                        status: "Kindly SignUp!".to_string()
                    }
                };

                let _ = response_status.send(response);
            },
            DbCommand::OnRamp { delta_balance, user_email , response_status} => {
                let status: OnRampDbResponseType = if let Some(user) = users.get_mut(&user_email){ //users.get_mut --> returns a mutable reference to the users hashmap
                    user.balance += delta_balance;
                    OnRampDbResponseType { status: format!("Successfull! User {} now has {} ", user.email, user.balance), balance: user.balance }
                } else {
                    OnRampDbResponseType { status: format!("User not found! User: {} found", user_email).to_string(), balance: 0}
                };
                let _ = response_status.send(status);
            }
        }
    }
}

async fn signup_function(
    State(db_tx): State<DbSender>,
    Json(payload): Json<AuthRequest>
) -> AuthResponse{
    let (oneshot_tx, oneshot_rx) = oneshot::channel();
    println!("{}", String::from("thread reached here"));
    let create_user = db_tx.send(DbCommand::Signup{email: payload.email.clone(), password: payload.password, response_status:oneshot_tx}).await;
    match oneshot_rx.await {
        Ok(response) => {
            if response.status.contains("already exists") {
                AuthResponse::unauthorised(response.status)
            } else {
                AuthResponse::created(response.status)
            }
        }
        Err(e) => {
            AuthResponse::unauthorised(format!("Actor failed to respond: {}", e))
        }
    }
}

async fn signin_function(
    State(db_tx): State<DbSender>,
    Json(payload): Json<AuthRequest>
) -> AuthResponse{
    let (oneshot_tx, oneshot_rx) = oneshot::channel(); //we declared a oneshot to send it to the db_actor to receive back response
    let check_user = db_tx.send(DbCommand::Signin{
        email: payload.email.clone(),
        password: payload.password,
        response_status:oneshot_tx}
    ).await;

    match oneshot_rx.await {
        Ok(response) => {
            if response.status.contains("Incorrect Password") {
                AuthResponse::unauthorised(response.status)
            } else {
                AuthResponse::created(response.status)
            }
        }
        Err(e) => {
            AuthResponse::unauthorised(format!("Actor failed to respond: {}", e))
        }
    }
}

async fn onramp_function(
    State(db_tx):State<DbSender>,
    Json(payload):Json<OnRampHttpRequest>
) -> OnRampResponse {
    let (oneshot_tx, oneshot_rx) = oneshot::channel();
    let update_userbalance = db_tx.send(DbCommand::OnRamp{
        user_email: payload.user_email.clone(),
        delta_balance: payload.balance,
        response_status: oneshot_tx
    }).await;
    
    match oneshot_rx.await {
        Ok(response) => {
            if response.status.contains("Successfull"){
                OnRampResponse::ok(response.status, response.balance)
            } else {
                OnRampResponse::err(response.status, response.balance)
            }
        },
        Err(response) => {
            OnRampResponse::err("Internal server Error", 0)
        } 
    }
}

#[tokio::main]
async fn main() {

    let (tx, rx) = mpsc::channel::<DbCommand>(32); // defining an mpsc channel
    tokio::spawn(user_db_actor(rx));
    let db_tx = tx.clone();
    // build our application with a single route
    let app = Router::new().route("/", post(|| async{"Hello World!"}))
    .route("/signup", post(signup_function))
    .route("/signin", post(signin_function))
    .route("/onramp", post(onramp_function)) //this route will return OnRampResponse type of its own which tells back the request handler the updated balance
    .with_state(db_tx);

    // .route("/create_limit_order", post(create_limit_order_function))
    // .route("/create_market_order", post(create_market_order_function))
    // .route("/get_orderbook", get(get_orderbook_function));    

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();

    axum::serve(listener, app).await.unwrap(); //use anyhow
}


// async function create_limit_order_function(){
    

// }

// async function create_market_order_function(){
    

// }

// async function get_orderbook_function(){
    

// }
