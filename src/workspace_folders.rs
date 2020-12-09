use serde::{Deserialize, Serialize};
use url::Url;

use crate::OneOf;

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct WorkspaceFileOperationsClientCapabilities {
    /// Whether the client supports dynamic registration for file
    /// requests/notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /// The client has support for sending didCreateFiles notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_create: Option<bool>,

    /// The server is interested in receiving willCreateFiles requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_create: Option<bool>,

    /// The server is interested in receiving didRenameFiles requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_rename: Option<bool>,

    /// The server is interested in receiving willRenameFiles requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_rename: Option<bool>,

    /// The server is interested in receiving didDeleteFiles requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_delete: Option<bool>,

    /// The server is interested in receiving willDeleteFiles requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_delete: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceServerCapabilities {
    /// The server supports workspace folder.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_folders: Option<WorkspaceFoldersServerCapabilities>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(feature = "proposed")]
    pub file_operations: Option<WorkspaceFileOperationsServerCapabilities>,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFoldersServerCapabilities {
    /// The server has support for workspace folders
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported: Option<bool>,

    /// Whether the server wants to receive workspace folder
    /// change notifications.
    ///
    /// If a string is provided, the string is treated as an ID
    /// under which the notification is registered on the client
    /// side. The ID can be used to unregister for these events
    /// using the `client/unregisterCapability` request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_notifications: Option<OneOf<bool, String>>,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct WorkspaceFileOperationsServerCapabilities {
    /// The server is interested in receiving didCreateFiles
    /// notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_create: Option<FileOperationRegistrationOptions>,

    /// The server is interested in receiving willCreateFiles requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_create: Option<FileOperationRegistrationOptions>,

    /// The server is interested in receiving didRenameFiles
    /// notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_rename: Option<FileOperationRegistrationOptions>,

    /// The server is interested in receiving willRenameFiles requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_rename: Option<FileOperationRegistrationOptions>,

    /// The server is interested in receiving didDeleteFiles file
    /// notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_delete: Option<FileOperationRegistrationOptions>,

    /// The server is interested in receiving willDeleteFiles file
    /// requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_delete: Option<FileOperationRegistrationOptions>,
}

/// The options to register for file operations.
///
/// @since 3.16.0 - proposed state
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct FileOperationRegistrationOptions {
    pub patterns: Vec<FileOperationPattern>,
}

/// A pattern kind describing if a glob pattern matches a file a folder or
///  both.
///
///  @since 3.16.0 - proposed state
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
#[cfg(feature = "proposed")]
pub enum FileOperationPatternKind {
    /// The pattern matches a file only.
    File,

    /// The pattern matches a folder only.
    Folder,
}

/// Matching options for the file operation pattern.
///
/// @since 3.16.0 - proposed state
///
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct FileOperationPatternOptions {
    /// The pattern should be matched ignoring casing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_case: Option<bool>,
}

/// A pattern to describe in which file operation requests or notifications
/// the server is interested in.
///
/// @since 3.16.0 - proposed state
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct FileOperationPattern {
    /// The glob pattern to match. Glob patterns can have the following syntax:
    /// - `*` to match one or more characters in a path segment
    /// - `?` to match on one character in a path segment
    /// - `**` to match any number of path segments, including none
    /// - `{}` to group conditions (e.g. `**​/*.{ts,js}` matches all TypeScript
    ///   and JavaScript files)
    /// - `[]` to declare a range of characters to match in a path segment
    ///   (e.g., `example.[0-9]` to match on `example.0`, `example.1`, …)
    /// - `[!...]` to negate a range of characters to match in a path segment
    ///   (e.g., `example.[!0-9]` to match on `example.a`, `example.b`, but
    ///   not `example.0`)
    pub glob: String,

    /// Whether to match files or folders with this pattern.
    ///
    /// Matches both if undefined.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matches: Option<FileOperationPatternKind>,

    /// Additional options used during matching.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<FileOperationPatternOptions>,
}

/// The parameters sent in notifications/requests for user-initiated creation
///  of files.
///
///  @since 3.16.0 - proposed state
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct CreateFilesParams {
    /// An array of all files/folders created in this operation.
    pub files: Vec<FileCreate>,
}
/// Represents information on a file/folder create.
///
/// @since 3.16.0 - proposed state
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct FileCreate {
    /// A file:// URI for the location of the file/folder being created.
    pub uri: String,
}

/// The parameters sent in notifications/requests for user-initiated renames
/// of files.
///
/// @since 3.16.0 - proposed state
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct RenameFilesParams {
    /// An array of all files/folders renamed in this operation. When a folder
    /// is renamed, only the folder will be included, and not its children.
    pub files: Vec<FileRename>,
}

/// Represents information on a file/folder rename.
///
/// @since 3.16.0 - proposed state
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct FileRename {
    /// A file:// URI for the original location of the file/folder being renamed.
    pub old_uri: String,

    /// A file:// URI for the new location of the file/folder being renamed.
    pub new_uri: String,
}

/// The parameters sent in notifications/requests for user-initiated deletes
/// of files.
///
/// @since 3.16.0 - proposed state
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct DeleteFilesParams {
    /// An array of all files/folders deleted in this operation.
    pub files: Vec<FileDelete>,
}
/// Represents information on a file/folder delete.
///
/// @since 3.16.0 - proposed state
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct FileDelete {
    /// A file:// URI for the location of the file/folder being deleted.
    pub uri: String,
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
