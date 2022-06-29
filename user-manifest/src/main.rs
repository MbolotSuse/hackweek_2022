mod controller;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use kube::Client;
use crate::controller::rbac_controller::{RBACController, run_controllers};

#[get("/health")]
async fn health() -> impl Responder{
    HttpResponse::Ok().body("ok")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client_result = Client::try_default().await;
    let rbac_controller = crate::controller::rbac_controller::new();
    let &controller_ref = &rbac_controller;
    let client = match client_result{
        Ok(new_client) => new_client,
        Err(result) => return std::io::Result::Err(std::io::Error::new(std::io::ErrorKind::Other,result.to_string())),
    };
    run_controllers(client, rbac_controller);
    HttpServer::new( move || {
        App::new().
            app_data(web::Data::new(controller_ref)).
            service(health)
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}