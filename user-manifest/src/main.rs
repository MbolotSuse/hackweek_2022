mod controller;
mod endpoints;

use std::sync::{Arc, RwLock};
use actix_web::{web, App, HttpServer, rt};
use kube::Client;
use crate::controller::rbac_controller::{RBACController, run_controllers};
use crate::endpoints::health::health;
use crate::endpoints::grants::{get_grants, get_grant_counts};
use crate::endpoints::permissions::get_permissions;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client_result = Client::try_default().await;
    let controller = crate::controller::rbac_controller::new();
    let rbac_controller = Arc::new(RwLock::new(controller));
    let client = match client_result{
        Ok(new_client) => new_client,
        Err(result) => return std::io::Result::Err(std::io::Error::new(std::io::ErrorKind::Other,result.to_string())),
    };
    let thread_controller = Arc::clone(&rbac_controller);
    rt::spawn(run_controllers(client, thread_controller));
    HttpServer::new( move || {
        App::new().
            app_data(web::Data::new(Arc::clone(&rbac_controller))).
            route("/health", web::get().to(health)).
            route("/grants/subject", web::post().to(get_grants)).
            route("/grants/count", web::get().to(get_grant_counts)).
            route("/permissions/subject", web::post().to(get_permissions))
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}