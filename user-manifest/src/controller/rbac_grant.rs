use std::fmt;
use std::fmt::{Formatter};
use std::hash::{Hash, Hasher};

/// Generic form of an identifier for an RBAC resource (role/cluster role). Does not contain rules
/// To avoid re-storing rules in memory
#[derive(Eq, PartialEq, Hash)]
pub struct RBACId{
    /// type of resource which holds permissions - e.x. role or cluster_role
    pub(crate) rbac_type: IDType,
    /// namespace which this RBAC resource lives in - may be none if source resource is cluster-wide
    pub(crate) namespace: Option<String>,
    /// name of the rbac resource
    pub(crate) name: String,
}

/// Object which grants RBAC permissions. Generic form of role_binding/cluster_role_binding
#[derive(Eq, PartialEq, Hash)]
pub struct RBACGrant {
    //TODO: Custom hash (and maybe eq?) function which ignores permissions_id.
    /// type of resource which grants RBAC permissions - e.x. role_binding or cluster_role_binding
    pub(crate) grant_type: GrantType,
    /// namespace which the permission grant occurs in - may be none if the grant is cluster-wide
    pub(crate) namespace: Option<String>,
    /// name of the grant - unique within the grant_type for this namespace
    pub(crate) name: String,
    /// the id of the permissions granted by this permissions grant
    pub(crate) permissions_id: RBACId,
}

/// Enum for the Types of Grants - Can be expanded to support other sources of permissions
#[derive(Eq, PartialEq, Hash)]
pub enum GrantType{
    RoleBinding,
    ClusterRoleBinding,
}

impl fmt::Display for GrantType{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self{
            GrantType::RoleBinding => {
                write!(f, "RoleBinding")
            }
            GrantType::ClusterRoleBinding => {
                write!(f, "ClusterRoleBinding")
            },
        }
    }
}

/// Enum for the Type of RBAC resources - Can be expanded to other resources which hold RBAC rules
#[derive(Eq, PartialEq, Hash)]
pub enum IDType{
    Role,
    ClusterRole,
}

impl fmt::Display for IDType{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self{
            IDType::Role => {
                write!(f, "Role")
            }
            IDType::ClusterRole => {
                write!(f, "ClusterRole")
            },
        }
    }
}