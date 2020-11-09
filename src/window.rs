#[cfg(feature = "proposed")]
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[cfg(feature = "proposed")]
use serde_json::Value;

use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Eq, PartialEq, Clone, Copy, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum MessageType {
    /// An error message.
    Error = 1,
    /// A warning message.
    Warning = 2,
    /// An information message.
    Info = 3,
    /// A log message.
    Log = 4,
}

/// Window specific client capabilities.
#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowClientCapabilities {
    /// Whether client supports handling progress notifications. If set
    /// servers are allowed to report in `workDoneProgress` property in the
    /// request specific server capabilities.
    ///
    /// @since 3.15.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_done_progress: Option<bool>,

    /// Capabilities specific to the showMessage request
    ///
    /// @since 3.16.0 - proposed state
    ///
    #[cfg(feature = "proposed")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_message: Option<ShowMessageRequestClientCapabilities>,
}

/// Show message request client capabilities
#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
#[cfg(feature = "proposed")]
#[serde(rename_all = "camelCase")]
pub struct ShowMessageRequestClientCapabilities {
    /// Capabilities specific to the `MessageActionItem` type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_action_item: Option<MessageActionItemCapabilities>,
}

#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
#[cfg(feature = "proposed")]
#[serde(rename_all = "camelCase")]
pub struct MessageActionItemCapabilities {
    /// Whether the client supports additional attribues which
    /// are preserved and send back to the server in the
    /// request's response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_properties_support: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageActionItem {
    /// A short title like 'Retry', 'Open Log' etc.
    pub title: String,

    /// Additional attributes that the client preserves and
    /// sends back to the server. This depends on the client
    /// capability window.messageActionItem.additionalPropertiesSupport
    #[cfg(feature = "proposed")]
    #[serde(flatten)]
    pub properties: HashMap<String, MessageActionItemProperty>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[cfg(feature = "proposed")]
#[serde(untagged)]
pub enum MessageActionItemProperty {
    String(String),
    Boolean(bool),
    Integer(i32),
    Object(Value),
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct LogMessageParams {
    /// The message type. See {@link MessageType}
    #[serde(rename = "type")]
    pub typ: MessageType,

    /// The actual message
    pub message: String,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct ShowMessageParams {
    /// The message type. See {@link MessageType}.
    #[serde(rename = "type")]
    pub typ: MessageType,

    /// The actual message.
    pub message: String,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct ShowMessageRequestParams {
    /// The message type. See {@link MessageType}
    #[serde(rename = "type")]
    pub typ: MessageType,

    /// The actual message
    pub message: String,

    /// The message action items to present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<MessageActionItem>>,
}
