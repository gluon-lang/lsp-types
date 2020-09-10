use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceCapability {
    /// The server supports workspace folder.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_folders: Option<WorkspaceFolderCapability>,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFolderCapability {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_notifications: Option<WorkspaceFolderCapabilityChangeNotifications>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum WorkspaceFolderCapabilityChangeNotifications {
    Bool(bool),
    Id(String),
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFolder {
    /// The associated URI for this workspace folder.
    pub uri: Url,
    /// The name of the workspace folder. Defaults to the uri's basename.
    pub name: String,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DidChangeWorkspaceFoldersParams {
    /// The actual workspace folder change event.
    pub event: WorkspaceFoldersChangeEvent,
}

/// The workspace folder change event.
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFoldersChangeEvent {
    /// The array of added workspace folders
    pub added: Vec<WorkspaceFolder>,

    /// The array of the removed workspace folders
    pub removed: Vec<WorkspaceFolder>,
}
