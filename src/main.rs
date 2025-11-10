use std::{collections::{BTreeMap, HashMap, VecDeque}, hash::Hash, os::macos::raw::stat, ptr::null, vec}; //used to store the users array in memory
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
    pub new_holdings: u64,
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
     
     pub fn ok(msg: impl Into<String>, new_balance: u64, new_holdings: u64) -> Self{
        Self { message: msg.into() , new_balance: new_balance, new_holdings: new_holdings, status: StatusCode::ACCEPTED }
     }

     pub fn err(msg: impl Into<String>, new_balance: u64, new_holdings: u64) -> Self{
        Self { message: msg.into(), new_balance, new_holdings: new_holdings ,status: status::StatusCode::INTERNAL_SERVER_ERROR }
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
        user_email: String,
        delta_balance: u64,
        delta_holdings: u64,
        response_status: oneshot::Sender<OnRampDbResponseType>
    },
    CheckUser{
        user_email: String,
        response_status: oneshot::Sender<CheckUserDbResponseType>
    }
    // UpdateBalance { email: String, amount: u32 },
}

#[derive(Deserialize)]
enum Side {
    Bid,
    Ask,
}

struct Trade {
    buyer: String,
    seller: String,
    qty: u64,
    price : u64
}
struct OrderSummary {
    owner: String,
    qty: u64,
    price: u64,
    side: Side,      // Reuse Side::Bid / Side::Ask
}
struct OrderbookResponse {
    status: String,
    fills: Vec<Trade> , //change to trade later,
    remaining_qty: u64,
    bids: Option<Vec<OrderSummary>>,
    asks: Option<Vec<OrderSummary>>
}
enum OrderbookCommand {
    NewLimitOrder{
        market_id: String,
        user_id: String,
        side: Side,
        qty: u64,
        price: u64,
        resp: oneshot::Sender<OrderbookResponse> 
    },
    NewMarketOrder{
        market_id: String,
        user_id: String,
        side: Side,
        qty: u64,
        resp: oneshot::Sender<OrderbookResponse>
    },
    GetBook{
        market_id: String,
        resp: oneshot::Sender<OrderbookResponse>
    }
}
#[derive(Deserialize)]
struct Order{
    qty: u64,
    price: u64,
    side: Side
}
struct User{
    email: String,
    password: String,
    balance: u64,
    holdings: u64
}

struct MarketBook {
    bids: BTreeMap<u64, VecDeque<Order>>,
    asks: BTreeMap<u64, VecDeque<Order>>,
}

impl MarketBook {
    fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }
}

// impl MarketBook {

//     fn new() -> Self{

//     }

//     fn inser_order(&mut self, order: Order) -> {

//     }

//     async fn match_order(&mut self, order: Order, db_tx: &DbSender) -> OrderbookResponse{
//         trades = [];
//         if order.side == "Bid" {

//         }

//     }


//     fn snapshot(&self) -> (Vec<OrderSummary>, Vec<OrderSummary>);
// }

struct OnRampDbResponseType{
    status: String,
    balance: u64,
    holdings: u64
}

struct CheckUserDbResponseType{
    user_exists: bool
}

#[derive(Deserialize)]
pub struct OnRampHttpRequest{
    pub user_email: String,
    pub balance: u64,
    pub holding: u64
}

#[derive(Deserialize)]
pub struct CreateMarketOrderRequest{
    pub market_id: String,
    pub user_email: String,
    pub order: Order,
}

pub struct CreateMarketOrderResponse{
    pub message: String,
    pub trades: Vec<Trade>,
    pub status: StatusCode
}

impl IntoResponse for CreateMarketOrderResponse{
    fn into_response(self) -> Response {
                let body = Json(serde_json::json!({"Message": self.message}));
                (self.status, body).into_response()
            }
}


async fn user_db_actor(mut rx: mpsc::Receiver<DbCommand>){
    let mut users: HashMap<String, User> = HashMap::new();

    println!("UserDBActor started");

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
                        holdings: 0
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
            DbCommand::OnRamp { user_email , delta_balance, delta_holdings,response_status} => {
                let status: OnRampDbResponseType = if let Some(user) = users.get_mut(&user_email){ //users.get_mut --> returns a mutable reference to the users hashmap
                    user.balance += delta_balance;
                    user.holdings += delta_holdings;
                    OnRampDbResponseType { status: format!("Successfull! User {} now has balance : {} , holding: {} ", user.email, user.balance, user.holdings), balance: user.balance, holdings: user.holdings }
                } else {
                    OnRampDbResponseType { status: format!("User not found! User: {} found", user_email).to_string(), balance: 0, holdings: 0}
                };
                let _ = response_status.send(status);
            }
            DbCommand::CheckUser { user_email, response_status } => {
                let response = CheckUserDbResponseType { user_exists: users.contains_key(&user_email) };
                let _ = response_status.send(response);
            }
        }
    }
}

async fn orderbook_actor(mut rx: mpsc::Receiver<OrderbookCommand>, db_tx: DbSender){
    let mut order_book: HashMap<String, MarketBook>= HashMap::new();

    let order_book_1 = MarketBook::new();
    order_book.insert("1".to_string(), order_book_1);

    println!("MarketBookDbActor Started");
    println!("Initialized first market with id = 1");

    while let Some(cmd) = rx.recv().await{
        match cmd {
            OrderbookCommand::NewLimitOrder { market_id, user_id, side, qty, price, resp } => {
                let response = if order_book.contains_key(&market_id){
                    let (oneshot_tx, oneshot_rx) = oneshot::channel();
                    let check_user = db_tx.send(DbCommand::CheckUser { user_email: user_id, response_status: oneshot_tx }).await;
                    match oneshot_rx.await {
                        Ok(response) => {
                            if (response.user_exists) {
                                println!("User exits");
                                OrderbookResponse {
                                    status: "User Does Exist".to_string(),
                                    fills: vec![],
                                    remaining_qty: 0,
                                    bids: None,
                                    asks: None
                                }
                            } else {
                                println!("User does not exists.");
                                OrderbookResponse{
                                    status: "User Does Not Exists".to_string(),
                                    fills: vec![],
                                    remaining_qty: 0,
                                    bids: None,
                                    asks: None
                                }
                            }
                        }
                        Err(response) => {
                            print("Error finding User in the database");
                                OrderbookResponse{
                                    status: "Error finding User in the data base".to_string(),
                                    fills: vec![],
                                    remaining_qty: 0,
                                    bids: None,
                                    asks: None
                                }
                        }
                    }
                    

                    println!("Market {} exists, inserting order...", market_id);
                    OrderbookResponse{
                        status: "Order added Successfull".to_string(),
                        fills: vec![],
                        remaining_qty: 0,
                        bids: None,
                        asks: None
                    }

                } else {
                    println!("Market with Market id = {}, does not exist", market_id);
                    OrderbookResponse {
                        status: "Market does not exist".to_string(),
                        fills: vec![] ,
                        remaining_qty: 0,
                        bids: None,
                        asks: None
                    }
                };
                let _ = resp.send(response);
            }
            OrderbookCommand::NewMarketOrder { market_id, user_id, side, qty, resp } => {
                let response = if order_book.contains_key(&market_id){
                    
                    //Todo Create Market Order

                    println!("Market {} exists, inserting order...", market_id);
                    OrderbookResponse{
                        status: "Order added Successfull".to_string(),
                        fills: vec![],
                        remaining_qty: 0,
                        bids: None,
                        asks: None
                    }

                } else {
                    println!("Market with Market id = {}, does not exist", market_id);
                    OrderbookResponse {
                        status: "Market does not exist".to_string(),
                        fills: vec![] ,
                        remaining_qty: 0,
                        bids: None,
                        asks: None
                    }
                };
                let _ = resp.send(response);
            }
            OrderbookCommand::GetBook { market_id, resp } => {
                let response = if order_book.contains_key(&market_id){
                        println!("Market Exists , id = {}", market_id);
                        OrderbookResponse{
                            status: "This is the current status of the orderBook".to_string(),
                            fills: vec![],
                            remaining_qty: 0,
                            bids: None,
                            asks: None
                        } 
                    } else{
                        println!("Market does not exists, id = {}", market_id);
                        OrderbookResponse { 
                            status: "Market Does not exists".to_string(), 
                            fills: vec![], 
                            remaining_qty: 0, 
                            bids: None,
                            asks: None
                         }
                    };
            }
        }
    }
}



async fn signup_function(
    State(state): State<AppState>,
    Json(payload): Json<AuthRequest>
) -> AuthResponse{
    let db_tx = state.db_tx;
    let (oneshot_tx, oneshot_rx) = oneshot::channel();
    println!("{}", String::from("thread reached here"));
    let _ = db_tx.send(DbCommand::Signup{email: payload.email.clone(), password: payload.password, response_status:oneshot_tx}).await;
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
    State(state): State<AppState>,
    Json(payload): Json<AuthRequest>
) -> AuthResponse{
    let db_tx = state.db_tx;
    let (oneshot_tx, oneshot_rx) = oneshot::channel(); //we declared a oneshot to send it to the db_actor to receive back response
    let _ = db_tx.send(DbCommand::Signin{
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
    State(state):State<AppState>,
    Json(payload):Json<OnRampHttpRequest>
) -> OnRampResponse {
    let db_tx = state.db_tx;
    let (oneshot_tx, oneshot_rx) = oneshot::channel();
    let _ = db_tx.send(DbCommand::OnRamp{
        user_email: payload.user_email.clone(),
        delta_balance: payload.balance,
        delta_holdings: payload.holding,
        response_status: oneshot_tx
    }).await;
    
    match oneshot_rx.await {
        Ok(response) => {
            if response.status.contains("Successfull"){
                OnRampResponse::ok(response.status, response.balance, response.holdings)
            } else {
                OnRampResponse::err(response.status, response.balance, response.holdings)
            }
        },
        Err(response) => {
            OnRampResponse::err("Internal server Error", 0, 0)
        } 
    }
}

async fn create_limit_order_function(
    State(state): State<AppState>,
    Json(payload): Json<CreateMarketOrderRequest>
) -> CreateMarketOrderResponse{
    let ob_tx = state.ob_tx;
    let (oneshot_tx, oneshot_rx) = oneshot::channel();
    let _ = ob_tx.send(OrderbookCommand::NewLimitOrder { 
        market_id: payload.market_id, 
        user_id: payload.user_email,
        side: payload.order.side, 
        qty: payload.order.qty, 
        price: payload.order.price, 
        resp: oneshot_tx 
    }).await;

    match oneshot_rx.await {
        Ok(response) => {
            if response.status.contains("Successfull"){
                CreateMarketOrderResponse{
                    message: "Market Order Created".to_string(),
                    trades: response.fills,
                    status: StatusCode::OK
                }
            } else {
                CreateMarketOrderResponse { 
                    message: response.status.to_string(), 
                    trades: vec![], 
                    status: StatusCode::EXPECTATION_FAILED}
            }
        },
        Err(response) => {
            CreateMarketOrderResponse{
                message:"Error Creating Market Order".to_string(),
                trades: vec![],
                status: StatusCode::INTERNAL_SERVER_ERROR
            }
        }

    }

}

#[derive(Clone)]
struct AppState {
    db_tx: mpsc::Sender<DbCommand>,
    ob_tx: mpsc::Sender<OrderbookCommand>,
}

#[tokio::main]
async fn main() {

    let (tx, rx) = mpsc::channel::<DbCommand>(32); // defining an mpsc channel
    tokio::spawn(user_db_actor(rx));
    let (tx2, rx2 ) = mpsc::channel::<OrderbookCommand>(32);
    tokio::spawn(orderbook_actor(rx2, tx.clone()));
    // let ob_tx = tx2.clone();
    // let db_tx = tx.clone();
    // build our application with a single route

    let state = AppState {
        db_tx: tx.clone(),
        ob_tx: tx2.clone(),
    };

    let app = Router::new().route("/", post(|| async{"Hello World!"}))
    .route("/signup", post(signup_function))
    .route("/signin", post(signin_function))
    .route("/onramp", post(onramp_function)) //this route will return OnRampResponse type of its own which tells back the request handler the updated balance
    .route("/createLimitOrder", post(create_limit_order_function))
    .with_state(state);

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
