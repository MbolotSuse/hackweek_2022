use futures::{pin_mut, TryStreamExt};
use std::collections::{HashMap, HashSet};
use kube::{api::{Api, ListParams}, runtime::{watcher, WatchStreamExt}, Client};
use crate::controller::rbac_grant::{GrantType, IDType, RBACGrant, RBACId};
use k8s_openapi::api::rbac::v1::{PolicyRule, Role, ClusterRole, RoleBinding, ClusterRoleBinding};
use std::error::Error;

pub struct RBACController{
    user_to_grant: HashMap<String, HashSet<RBACGrant>>,
    grant_to_permissions: HashMap<RBACId, Vec<PolicyRule>>
}

pub async fn run_controllers(client: Client, controller: RBACController){
    let role_binding_api = Api::<RoleBinding>::all(client.clone());
    let cluster_role_binding_api = Api::<ClusterRoleBinding>::all(client.clone());
    let role_api = Api::<Role>::all(client.clone());
    let cluster_role_api = Api::<ClusterRole>::all(client.clone());
    let role_watcher = watcher(role_binding_api, ListParams::default()).applied_objects();
    pin_mut!(role_watcher);
    while let Ok(Some(event)) = role_watcher.try_next().await{
        let grant_result = convert_role_binding_to_grant(&event).await;
        let grant = match grant_result{
            Ok(result) => result,
            Err(result) => {
                eprintln!("unable to convert role_binding {:?} to RBACGrant with error {:?}, will skip", event, result);
                continue;
            },
        };
        let role_ns = match grant.permissions_id.namespace.clone(){
            Some(ns) => ns,
            None => {
                eprintln!("Unable to use get role {} due to Optional/None namespace in grant", grant.permissions_id.name);
                continue;
            },
        };
        let ns_role_api = Api::<Role>::namespaced(client.clone(), role_ns.as_str());
        let rules_result = get_rules(&grant, ns_role_api, cluster_role_api.clone());
        match rules_result{
            Ok(rules) => {
                //TODO: store the rules in the controller
            },
            None => {
                eprintln!("Unable to get rules for grant {}, skipping", grant.name);
                continue;
            }
        }
    };
}

pub async fn get_rules(grant: &RBACGrant, role_api: Api::<Role>, cluster_role_api: Api::<ClusterRole>) -> Result<Vec<PolicyRule>, Box<dyn Error>>{
    if grant.permissions_id.rbac_type == IDType::Role {
        let role_result = role_api.get(grant.permissions_id.name.as_str()).await;
        match role_result{
            Ok(role_result) => {
                match role_result.rules {
                    Some(role_rules) => Result::Ok(role_rules),
                    None => Result::Ok(Vec::new()),
                }
            },
            Err(role_result) => {
                return Result::Err(format!("Unable to retrieve details for role {} due to error {:?}", grant.permissions_id.name, role_result).into());
            }
        }
    }else if grant.permissions_id.rbac_type == IDType::ClusterRole {
        let cluster_role_result = cluster_role_api.get(grant.permissions_id.name.as_str()).await;
        // TODO: Technically, these are identical. However, the return type on cluster_role_api and role_api's get methods are different
        // This could probably be solved with a macro
        match cluster_role_result {
            Ok(role_result) => {
                match role_result.rules {
                    Some(role_rules) => Result::Ok(role_rules),
                    None => Result::Ok(Vec::new()),
                }
            },
            Err(role_result) => {
                return Result::Err(format!("Unable to retrieve details for role {} due to error {:?}", grant.permissions_id.name, role_result).into());
            }
        }
    }else{
        return Result::Err(format!("Invalid rbac type {} on grant {}", grant.permissions_id.rbac_type, grant.name).into())
    }
}

async fn convert_role_binding_to_grant(role_binding: &RoleBinding) -> Result<RBACGrant, Box<dyn Error>>{
    let binding_name = match role_binding.metadata.name.clone(){
        Some(name) => name,
        None => return Result::Err("role binding was missing name".into())
    };
    let binding_namespace = match role_binding.metadata.namespace.clone(){
        Some(namespace) => namespace,
        None => return Result::Err("role binding namespace was missing".into())
    };
    let rbac_type;
    let mut id_namespace: Option<String> = None;
    match role_binding.role_ref.kind.as_str(){
        "Role" => {
            rbac_type = IDType::Role;
            id_namespace = Some(binding_namespace.clone());
        },
        "ClusterRole" => {
            rbac_type = IDType::ClusterRole;
        },
        _ =>{
            return Result::Err(format!("role ref was for a {}, not a ClusterRole or Role", role_binding.role_ref.kind).into())
        }
    };


    return Result::Ok(RBACGrant{
        grant_type: GrantType::RoleBinding,
        namespace: Some(binding_namespace),
        name: binding_name,
        permissions_id: RBACId{
            rbac_type,
            namespace: id_namespace,
            name: role_binding.role_ref.name.clone()
        }
    })
}

pub fn new() -> RBACController{
    let default_user_grant: HashMap<String, HashSet<RBACGrant>> = HashMap::new();
    let default_grant_to_perms: HashMap<RBACId, Vec<PolicyRule>> = HashMap::new();
    return RBACController{
        user_to_grant: default_user_grant,
        grant_to_permissions: default_grant_to_perms
    }
}