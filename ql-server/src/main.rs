extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate actix;
extern crate actix_web;
extern crate env_logger;
extern crate futures;
extern crate juniper;
extern crate ql_server;

use actix::prelude::*;
use actix_web::middleware::cors::Cors;
use actix_web::{
    http, middleware, server, App, AsyncResponder, Error, FutureResponse, HttpRequest,
    HttpResponse, Json, State,
};
use futures::future::Future;
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;

use ql_server::schema::{self, create_schema, Schema};

struct AppState {
    executor: Addr<GraphQLExecutor>,
}

#[derive(Serialize, Deserialize)]
pub struct GraphQLData(GraphQLRequest);

impl Message for GraphQLData {
    type Result = Result<String, Error>;
}

pub struct GraphQLExecutor {
    schema: std::sync::Arc<Schema>,
    context: schema::Context,
}

impl GraphQLExecutor {
    fn new(schema: std::sync::Arc<Schema>, context: schema::Context) -> GraphQLExecutor {
        GraphQLExecutor {
            schema,
            context,
        }
    }
}

impl Actor for GraphQLExecutor {
    type Context = SyncContext<Self>;
}

impl Handler<GraphQLData> for GraphQLExecutor {
    type Result = Result<String, Error>;

    fn handle(&mut self, msg: GraphQLData, _: &mut Self::Context) -> Self::Result {
        let res = msg.0.execute(&self.schema, &self.context);
        let res_text = serde_json::to_string(&res)?;
        Ok(res_text)
    }
}

fn graphiql(_req: &HttpRequest<AppState>) -> Result<HttpResponse, Error> {
    let html = graphiql_source("http://127.0.0.1:9029/graphql");
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

fn graphql((st, data): (State<AppState>, Json<GraphQLData>)) -> FutureResponse<HttpResponse> {
    st.executor
        .send(data.0)
        .from_err()
        .and_then(|res| match res {
            Ok(user) => Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body(user)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let sys = actix::System::new("juniper-example");

    let schema = std::sync::Arc::new(create_schema());
    let addr = SyncArbiter::start(3, move || {
        GraphQLExecutor::new(schema.clone(), schema::Context::new())
    });

    server::new(move || {
        App::with_state(AppState {
            executor: addr.clone(),
        })
        .middleware(middleware::Logger::default())
        .resource("/graphiql", |r| r.method(http::Method::GET).h(graphiql))
        .configure(|app| {
            Cors::for_app(app)
                .supports_credentials()
                .allowed_origin("http://localhost:9029")
                .allowed_headers(vec![
                    http::header::AUTHORIZATION,
                    http::header::ACCEPT,
                    http::header::ORIGIN,
                    http::header::CONTENT_TYPE,
                ])
                .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                .resource("/graphql", |r| r.method(http::Method::POST).with(graphql))
                .register()
        })
    })
    .bind("127.0.0.1:9029")
    .unwrap()
    .start();

    println!("Started http server: 127.0.0.1:9029");
    let _ = sys.run();
}
