use std::collections::{HashMap, LinkedList};
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use kube::{api::{Api, ListParams}, core::ObjectList, Client, Error};
use k8s_openapi::api::core::v1::{Pod, Namespace};

#[derive(Clone)]
struct KubeInfoServer{
    client: Client
}

/// Validates that a namespace exists. Returns a bool (does the namespace exist) and an error
/// (if we can't determine if the namespace exists)
async fn validate_namespace(namespace: &String, client: &Client) -> Result<bool, Error>{
    let namespace_api: Api<Namespace> = Api::all(client.clone());
    let namespace_result = namespace_api.get(namespace.as_str()).await;
    match namespace_result {
        Ok(_result) => Result::Ok(true),
        Err(result) => {
           match result {
               kube::Error::Api(ref api_error) => {
                   if api_error.code == 404{
                       Result::Ok(false)
                   }else{
                       Result::Err(result)
                   }
               },
               _ => {
                   Result::Err(result)
               }

           }
        },
    }
}

/// Gets all of the service accounts and which pods that they are bound to for a given namespace
/// Must be routed with a path variable {namespace} and a data source of KubeInfoServer
async fn get_all(namespace: web::Path<String>, info_server: web::Data<KubeInfoServer>) -> impl Responder{
    let resource_namespace = namespace.into_inner();
    let server = info_server.into_inner();
    let namespace_result = validate_namespace(&resource_namespace, &server.client.clone()).await;

    match namespace_result{
      Ok(result) =>{
          if !result{
              // if the namespace doesn't exist, return a 404
              return HttpResponse::NotFound().body("namespace not found");
          }
      },
      Err(result) => {
          // TODO: change this to log library
          // if we can't validate the namespace's existence, return an error and proceed no further
          println!("Error when validating namespace {:?}", result);
          return HttpResponse::InternalServerError().body("internal server error");
      }
    };

    let pods_api: Api<Pod> = Api::namespaced(server.client.clone(), resource_namespace.as_str());

    let pod_list_result = pods_api.list(&ListParams::default()).await;
    let pods:ObjectList<Pod> = match pod_list_result{
        Ok(result) => result,
        Err(result) => {
            // TODO: change this to log library
            // all the information we need to determine sa -> pod mapping is on the output list.
            println!("Error when retrieving pods {:?}", result);
            return HttpResponse::InternalServerError().body("internal server error");
        }
    };
    let mut service_accounts_to_pods: HashMap<String, LinkedList<String>> = HashMap::new();
    for pod in &pods{
        // Next section is a bit unwieldy due to forced optional handling
        let pod_name= match &pod.metadata.name{
            Some(name) => name,
            None => continue,
        };
        match &pod.spec{
            Some(spec) => {
                match &spec.service_account_name{
                    Some(sa_name) => {
                        let entry = service_accounts_to_pods.entry(
                                            sa_name.clone()).or_insert(LinkedList::new());
                        entry.push_back(pod_name.clone().to_string());
                    },
                    None => continue,
                }
            },
            None => continue,
        };
    }
    let output_result = serde_json::to_string(&service_accounts_to_pods);
    match output_result{
        Ok(result) => {
            HttpResponse::Ok().body(result)
        }
        Err(result) => {
            // TODO: change this to log library
            println!("Error when retrieving service accounts {:?}", result);
            HttpResponse::InternalServerError().body("internal server error")
        }
    }
}


#[get("/health")]
async fn health() -> impl Responder{
    HttpResponse::Ok().body("ok")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client_result = Client::try_default().await;
    let client = match client_result{
        Ok(new_client) => new_client,
        Err(result) => return std::io::Result::Err(std::io::Error::new(std::io::ErrorKind::Other,result.to_string())),
    };
    let info_server = KubeInfoServer{client};
    HttpServer::new( move || {
        App::new().
            app_data(web::Data::new(info_server.clone())).
            service(health).
            service(web::resource("/{namespace}").route(web::get().to(get_all)))
    })
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
