use axum::{
    extract::Form,
    response::Html,
    routing::{get, post},
    Router,
    Extension,
};
use mongodb::{options::ClientOptions, Client, Collection};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use handlebars::Handlebars;
use dotenv::dotenv;
use std::env;

#[derive(Debug, Deserialize, Serialize)]
struct Order {
    name: String,
    phone: String,
    address: String,
    delivery_time: String,
}

struct AppState {
    db: Collection<Order>,
    handlebars: Handlebars<'static>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let mongo_uri = env::var("MONGO_URI").expect("MONGO_URI must be set");

    // Initialize MongoDB client
    let client_options = ClientOptions::parse(&mongo_uri).await.unwrap();
    let client = Client::with_options(client_options).unwrap();
    let db = client.database("Order_locator").collection::<Order>("orders");

    // Initialize Handlebars
    let mut handlebars = Handlebars::new();
    handlebars.register_template_file("form", "templates/form.hbs").unwrap();

    // Shared state
    let state = Arc::new(AppState {
        db,
        handlebars,
    });

    // Build our application with a route
    let app = Router::new()
        .route("/", get(show_form).post(submit_form))
        .layer(Extension(state));

    // Run our app with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn show_form(state: Extension<Arc<AppState>>) -> Html<String> {
    let html = state.handlebars.render("form", &()).unwrap();
    Html(html)
}

async fn submit_form(
    state: Extension<Arc<AppState>>,
    Form(order): Form<Order>,
) -> Html<String> {
    state.db.insert_one(order, None).await.unwrap();
    Html("<h1>Order submitted successfully!</h1>".to_string())
}
