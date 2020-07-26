/*!

Language Server Protocol types for Rust.

Based on: <https://microsoft.github.io/language-server-protocol/specification>

This library uses the URL crate for parsing URIs.  Note that there is
some confusion on the meaning of URLs vs URIs:
<http://stackoverflow.com/a/28865728/393898>.  According to that
information, on the classical sense of "URLs", "URLs" are a subset of
URIs, But on the modern/new meaning of URLs, they are the same as
URIs.  The important take-away aspect is that the URL crate should be
able to parse any URI, such as `urn:isbn:0451450523`.


*/
#![allow(non_upper_case_globals)]
#![forbid(unsafe_code)]

#[macro_use]
extern crate bitflags;

use serde::{Deserialize, Serialize};
use serde_json;
use serde_repr::{Deserialize_repr, Serialize_repr};

pub use url::Url;

use std::borrow::Cow;

#[cfg(feature = "proposed")]
use std::convert::TryFrom;

use std::collections::HashMap;

#[cfg(feature = "proposed")]
use base64;
use serde::de;
use serde::de::Error as Error_;
use serde_json::Value;

#[cfg(feature = "proposed")]
use serde::ser::SerializeSeq;

pub mod notification;
pub mod request;

/* ----------------- Auxiliary types ----------------- */

#[derive(Debug, Eq, Hash, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum NumberOrString {
    Number(u64),
    String(String),
}

/* ----------------- Cancel support ----------------- */

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct CancelParams {
    /// The request id to cancel.
    pub id: NumberOrString,
}

/* ----------------- Basic JSON Structures ----------------- */

/// Position in a text document expressed as zero-based line and character offset.
/// A position is between two characters like an 'insert' cursor in a editor.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Default, Deserialize, Serialize)]
pub struct Position {
    /// Line position in a document (zero-based).
    pub line: u64,
    /// Character offset on a line in a document (zero-based).
    pub character: u64,
}

impl Position {
    pub fn new(line: u64, character: u64) -> Position {
        Position { line, character }
    }
}

/// A range in a text document expressed as (zero-based) start and end positions.
/// A range is comparable to a selection in an editor. Therefore the end position is exclusive.
#[derive(Debug, Eq, PartialEq, Copy, Clone, Default, Deserialize, Serialize)]
pub struct Range {
    /// The range's start position.
    pub start: Position,
    /// The range's end position.
    pub end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Range {
        Range { start, end }
    }
}

/// Represents a location inside a resource, such as a line inside a text file.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct Location {
    pub uri: Url,
    pub range: Range,
}

impl Location {
    pub fn new(uri: Url, range: Range) -> Location {
        Location { uri, range }
    }
}

/// Represents a link between a source and a target location.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationLink {
    /// Span of the origin of this link.
    ///
    ///  Used as the underlined span for mouse interaction. Defaults to the word range at
    ///  the mouse position.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_selection_range: Option<Range>,

    /// The target resource identifier of this link.
    pub target_uri: Url,

    /// The full target range of this link.
    pub target_range: Range,

    /// The span of this link.
    pub target_selection_range: Range,
}

/// Represents a diagnostic, such as a compiler error or warning.
/// Diagnostic objects are only valid in the scope of a resource.
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    /// The range at which the message applies.
    pub range: Range,

    /// The diagnostic's severity. Can be omitted. If omitted it is up to the
    /// client to interpret diagnostics as error, warning, info or hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<DiagnosticSeverity>,

    /// The diagnostic's code. Can be omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<NumberOrString>,
    //    code?: number | string;
    /// A human-readable string describing the source of this
    /// diagnostic, e.g. 'typescript' or 'super lint'.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// The diagnostic's message.
    pub message: String,

    /// An array of related diagnostic information, e.g. when symbol-names within
    /// a scope collide all definitions can be marked via this property.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_information: Option<Vec<DiagnosticRelatedInformation>>,

    /// Additional metadata about the diagnostic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<DiagnosticTag>>,
}

impl Diagnostic {
    pub fn new(
        range: Range,
        severity: Option<DiagnosticSeverity>,
        code: Option<NumberOrString>,
        source: Option<String>,
        message: String,
        related_information: Option<Vec<DiagnosticRelatedInformation>>,
        tags: Option<Vec<DiagnosticTag>>,
    ) -> Diagnostic {
        Diagnostic {
            range,
            severity,
            code,
            source,
            message,
            related_information,
            tags,
        }
    }

    pub fn new_simple(range: Range, message: String) -> Diagnostic {
        Self::new(range, None, None, None, message, None, None)
    }

    pub fn new_with_code_number(
        range: Range,
        severity: DiagnosticSeverity,
        code_number: u64,
        source: Option<String>,
        message: String,
    ) -> Diagnostic {
        let code = Some(NumberOrString::Number(code_number));
        Self::new(range, Some(severity), code, source, message, None, None)
    }
}

/// The protocol currently supports the following diagnostic severities:
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum DiagnosticSeverity {
    /// Reports an error.
    Error = 1,
    /// Reports a warning.
    Warning = 2,
    /// Reports an information.
    Information = 3,
    /// Reports a hint.
    Hint = 4,
}

/// Represents a related message and source code location for a diagnostic. This
/// should be used to point to code locations that cause or related to a
/// diagnostics, e.g when duplicating a symbol in a scope.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct DiagnosticRelatedInformation {
    /// The location of this related diagnostic information.
    pub location: Location,

    /// The message of this related diagnostic information.
    pub message: String,
}

/// The diagnostic tags.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum DiagnosticTag {
    /// Unused or unnecessary code.
    /// Clients are allowed to render diagnostics with this tag faded out instead of having
    /// an error squiggle.
    Unnecessary = 1,

    /// Deprecated or obsolete code.
    /// Clients are allowed to rendered diagnostics with this tag strike through.
    Deprecated = 2,
}

/// Represents a reference to a command. Provides a title which will be used to represent a command in the UI.
/// Commands are identitifed using a string identifier and the protocol currently doesn't specify a set of
/// well known commands. So executing a command requires some tool extension code.
#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
pub struct Command {
    /// Title of the command, like `save`.
    pub title: String,
    /// The identifier of the actual command handler.
    pub command: String,
    /// Arguments that the command handler should be
    /// invoked with.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<Value>>,
}

impl Command {
    pub fn new(title: String, command: String, arguments: Option<Vec<Value>>) -> Command {
        Command {
            title,
            command,
            arguments,
        }
    }
}

/// A textual edit applicable to a text document.
///
/// If n `TextEdit`s are applied to a text document all text edits describe changes to the initial document version.
/// Execution wise text edits should applied from the bottom to the top of the text document. Overlapping text edits
/// are not supported.
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextEdit {
    /// The range of the text document to be manipulated. To insert
    /// text into a document create a range where start === end.
    pub range: Range,
    /// The string to be inserted. For delete operations use an
    /// empty string.
    pub new_text: String,
}

impl TextEdit {
    pub fn new(range: Range, new_text: String) -> TextEdit {
        TextEdit { range, new_text }
    }
}

/// Describes textual changes on a single text document. The text document is referred to as a
/// `VersionedTextDocumentIdentifier` to allow clients to check the text document version before an
/// edit is applied. A `TextDocumentEdit` describes all changes on a version Si and after they are
/// applied move the document to version Si+1. So the creator of a `TextDocumentEdit` doesn't need to
/// sort the array or do any kind of ordering. However the edits must be non overlapping.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentEdit {
    /// The text document to change.
    pub text_document: VersionedTextDocumentIdentifier,

    /// The edits to be applied.
    pub edits: Vec<TextEdit>,
}

/// A special text edit to provide an insert and a replace operation.
///
/// @since 3.16.0 - Proposed state
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct InsertReplaceEdit {
    /// The string to be inserted.
    pub new_text: String,

    /// The range if the insert is requested
    pub insert: Range,

    /// The range if the replace is requested.
    pub replace: Range,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CompletionTextEdit {
    Edit(TextEdit),
    #[cfg(feature = "proposed")]
    InsertAndReplace(InsertReplaceEdit),
}

impl From<TextEdit> for CompletionTextEdit {
    fn from(edit: TextEdit) -> Self {
        CompletionTextEdit::Edit(edit)
    }
}

#[cfg(feature = "proposed")]
impl From<InsertReplaceEdit> for CompletionTextEdit {
    fn from(edit: InsertReplaceEdit) -> Self {
        CompletionTextEdit::InsertAndReplace(edit)
    }
}

/// Options to create a file.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFileOptions {
    /// Overwrite existing file. Overwrite wins over `ignoreIfExists`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overwrite: Option<bool>,
    /// Ignore if exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_if_exists: Option<bool>,
}

/// Create file operation
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFile {
    /// The resource to create.
    pub uri: Url,
    /// Additional options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<CreateFileOptions>,
}

/// Rename file options
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameFileOptions {
    /// Overwrite target if existing. Overwrite wins over `ignoreIfExists`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overwrite: Option<bool>,
    /// Ignores if target exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_if_exists: Option<bool>,
}

/// Rename file operation
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameFile {
    /// The old (existing) location.
    pub old_uri: Url,
    /// The new location.
    pub new_uri: Url,
    /// Rename options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<RenameFileOptions>,
}

/// Delete file options
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteFileOptions {
    /// Delete the content recursively if a folder is denoted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recursive: Option<bool>,
    /// Ignore the operation if the file doesn't exist.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_if_not_exists: Option<bool>,
}

/// Delete file operation
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteFile {
    /// The file to delete.
    pub uri: Url,
    /// Delete options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<DeleteFileOptions>,
}

/// A workspace edit represents changes to many resources managed in the workspace.
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceEdit {
    /// Holds changes to existing resources.
    #[serde(with = "url_map")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub changes: Option<HashMap<Url, Vec<TextEdit>>>, //    changes?: { [uri: string]: TextEdit[]; };

    /// Depending on the client capability `workspace.workspaceEdit.resourceOperations` document changes
    /// are either an array of `TextDocumentEdit`s to express changes to n different text documents
    /// where each text document edit addresses a specific version of a text document. Or it can contain
    /// above `TextDocumentEdit`s mixed with create, rename and delete file / folder operations.
    ///
    /// Whether a client supports versioned document edits is expressed via
    /// `workspace.workspaceEdit.documentChanges` client capability.
    ///
    /// If a client neither supports `documentChanges` nor `workspace.workspaceEdit.resourceOperations` then
    /// only plain `TextEdit`s using the `changes` property are supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_changes: Option<DocumentChanges>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum DocumentChanges {
    Edits(Vec<TextDocumentEdit>),
    Operations(Vec<DocumentChangeOperation>),
}

// TODO: Once https://github.com/serde-rs/serde/issues/912 is solved
// we can remove ResourceOp and switch to the following implementation
// of DocumentChangeOperation:
//
// #[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
// #[serde(tag = "kind", rename_all="lowercase" )]
// pub enum DocumentChangeOperation {
//     Create(CreateFile),
//     Rename(RenameFile),
//     Delete(DeleteFile),
//
//     #[serde(other)]
//     Edit(TextDocumentEdit),
// }

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged, rename_all = "lowercase")]
pub enum DocumentChangeOperation {
    Op(ResourceOp),
    Edit(TextDocumentEdit),
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ResourceOp {
    Create(CreateFile),
    Rename(RenameFile),
    Delete(DeleteFile),
}

#[derive(Debug, Default, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurationParams {
    pub items: Vec<ConfigurationItem>,
}

#[derive(Debug, Default, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurationItem {
    /// The scope to get the configuration section for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_uri: Option<String>,

    ///The configuration section asked for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
}

mod url_map {
    use super::*;

    use std::fmt;

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Option<HashMap<Url, Vec<TextEdit>>>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct UrlMapVisitor;
        impl<'de> de::Visitor<'de> for UrlMapVisitor {
            type Value = HashMap<Url, Vec<TextEdit>>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("map")
            }

            fn visit_map<M>(self, mut visitor: M) -> Result<Self::Value, M::Error>
            where
                M: de::MapAccess<'de>,
            {
                let mut values = HashMap::with_capacity(visitor.size_hint().unwrap_or(0));

                // While there are entries remaining in the input, add them
                // into our map.
                while let Some((key, value)) = visitor.next_entry::<Url, _>()? {
                    values.insert(key, value);
                }

                Ok(values)
            }
        }

        struct OptionUrlMapVisitor;
        impl<'de> de::Visitor<'de> for OptionUrlMapVisitor {
            type Value = Option<HashMap<Url, Vec<TextEdit>>>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("option")
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(None)
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(None)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_map(UrlMapVisitor).map(Some)
            }
        }

        // Instantiate our Visitor and ask the Deserializer to drive
        // it over the input data, resulting in an instance of MyMap.
        deserializer.deserialize_option(OptionUrlMapVisitor)
    }

    pub fn serialize<S>(
        changes: &Option<HashMap<Url, Vec<TextEdit>>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        match *changes {
            Some(ref changes) => {
                let mut map = serializer.serialize_map(Some(changes.len()))?;
                for (k, v) in changes {
                    map.serialize_entry(k.as_str(), v)?;
                }
                map.end()
            }
            None => serializer.serialize_none(),
        }
    }
}

impl WorkspaceEdit {
    pub fn new(changes: HashMap<Url, Vec<TextEdit>>) -> WorkspaceEdit {
        WorkspaceEdit {
            changes: Some(changes),
            document_changes: None,
        }
    }
}

/// Text documents are identified using a URI. On the protocol level, URIs are passed as strings.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct TextDocumentIdentifier {
    // !!!!!! Note:
    // In the spec VersionedTextDocumentIdentifier extends TextDocumentIdentifier
    // This modelled by "mixing-in" TextDocumentIdentifier in VersionedTextDocumentIdentifier,
    // so any changes to this type must be effected in the sub-type as well.
    /// The text document's URI.
    pub uri: Url,
}

impl TextDocumentIdentifier {
    pub fn new(uri: Url) -> TextDocumentIdentifier {
        TextDocumentIdentifier { uri }
    }
}

/// An item to transfer a text document from the client to the server.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentItem {
    /// The text document's URI.
    pub uri: Url,

    /// The text document's language identifier.
    pub language_id: String,

    /// The version number of this document (it will strictly increase after each
    /// change, including undo/redo).
    pub version: i64,

    /// The content of the opened text document.
    pub text: String,
}

impl TextDocumentItem {
    pub fn new(uri: Url, language_id: String, version: i64, text: String) -> TextDocumentItem {
        TextDocumentItem {
            uri,
            language_id,
            version,
            text,
        }
    }
}

/// An identifier to denote a specific version of a text document.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct VersionedTextDocumentIdentifier {
    // This field was "mixed-in" from TextDocumentIdentifier
    /// The text document's URI.
    pub uri: Url,

    /// The version number of this document.
    pub version: Option<i64>,
}

impl VersionedTextDocumentIdentifier {
    pub fn new(uri: Url, version: i64) -> VersionedTextDocumentIdentifier {
        VersionedTextDocumentIdentifier {
            uri,
            version: Some(version),
        }
    }
}

/// A parameter literal used in requests to pass a text document and a position inside that document.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentPositionParams {
    // !!!!!! Note:
    // In the spec ReferenceParams extends TextDocumentPositionParams
    // This modelled by "mixing-in" TextDocumentPositionParams in ReferenceParams,
    // so any changes to this type must be effected in sub-type as well.
    /// The text document.
    pub text_document: TextDocumentIdentifier,

    /// The position inside the text document.
    pub position: Position,
}

impl TextDocumentPositionParams {
    pub fn new(
        text_document: TextDocumentIdentifier,
        position: Position,
    ) -> TextDocumentPositionParams {
        TextDocumentPositionParams {
            text_document,
            position,
        }
    }
}

/// A document filter denotes a document through properties like language, schema or pattern.
/// Examples are a filter that applies to TypeScript files on disk or a filter the applies to JSON
/// files with name package.json:
///
/// { language: 'typescript', scheme: 'file' }
/// { language: 'json', pattern: '**/package.json' }
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct DocumentFilter {
    /// A language id, like `typescript`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// A Uri [scheme](#Uri.scheme), like `file` or `untitled`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheme: Option<String>,

    /// A glob pattern, like `*.{ts,js}`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

/// A document selector is the combination of one or many document filters.
pub type DocumentSelector = Vec<DocumentFilter>;

// ========================= Actual Protocol =========================

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    /// The process Id of the parent process that started
    /// the server. Is null if the process has not been started by another process.
    /// If the parent process is not alive then the server should exit (see exit notification) its process.
    pub process_id: Option<u64>,

    /// The rootPath of the workspace. Is null
    /// if no folder is open.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[deprecated(note = "Use `root_uri` instead when possible")]
    pub root_path: Option<String>,

    /// The rootUri of the workspace. Is null if no
    /// folder is open. If both `rootPath` and `rootUri` are set
    /// `rootUri` wins.
    #[serde(default)]
    pub root_uri: Option<Url>,

    /// User provided initialization options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initialization_options: Option<Value>,

    /// The capabilities provided by the client (editor)
    pub capabilities: ClientCapabilities,

    /// The initial trace setting. If omitted trace is disabled ('off').
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<TraceOption>,

    /// The workspace folders configured in the client when the server starts.
    /// This property is only available if the client supports workspace folders.
    /// It can be `null` if the client supports workspace folders but none are
    /// configured.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_folders: Option<Vec<WorkspaceFolder>>,

    /// Information about the client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_info: Option<ClientInfo>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct ClientInfo {
    /// The name of the client as defined by the client.
    pub name: String,
    /// The client's version as defined by the client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Serialize)]
pub struct InitializedParams {}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Deserialize, Serialize)]
pub enum TraceOption {
    #[serde(rename = "off")]
    Off,
    #[serde(rename = "messages")]
    Messages,
    #[serde(rename = "verbose")]
    Verbose,
}

impl Default for TraceOption {
    fn default() -> TraceOption {
        TraceOption::Off
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct GenericRegistrationOptions {
    #[serde(flatten)]
    pub text_document_registration_options: TextDocumentRegistrationOptions,

    #[serde(flatten)]
    pub options: GenericOptions,

    #[serde(flatten)]
    pub static_registration_options: StaticRegistrationOptions,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct GenericOptions {
    #[serde(flatten)]
    pub work_done_progress_options: WorkDoneProgressOptions,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct GenericParams {
    #[serde(flatten)]
    pub text_document_position_params: TextDocumentPositionParams,

    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,

    #[serde(flatten)]
    pub partial_result_params: PartialResultParams,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenericCapability {
    /// This capability supports dynamic registration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GotoCapability {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /// The client supports additional metadata in the form of definition links.
    pub link_support: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceEditCapability {
    /// The client supports versioned document changes in `WorkspaceEdit`s
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_changes: Option<bool>,

    /// The resource operations the client supports. Clients should at least
    /// support 'create', 'rename' and 'delete' files and folders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_operations: Option<Vec<ResourceOperationKind>>,

    /// The failure handling strategy of a client if applying the workspace edit
    /// failes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_handling: Option<FailureHandlingKind>,
}

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

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ResourceOperationKind {
    Create,
    Rename,
    Delete,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub enum FailureHandlingKind {
    Abort,
    Transactional,
    TextOnlyTransactional,
    Undo,
}

/// Specific capabilities for the `SymbolKind` in the `workspace/symbol` request.
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolKindCapability {
    /// The symbol kind values the client supports. When this
    /// property exists the client also guarantees that it will
    /// handle values outside its set gracefully and falls back
    /// to a default value when unknown.
    ///
    /// If this property is not present the client only supports
    /// the symbol kinds from `File` to `Array` as defined in
    /// the initial version of the protocol.
    pub value_set: Option<Vec<SymbolKind>>,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSymbolClientCapabilities {
    /// This capability supports dynamic registration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /// Specific capabilities for the `SymbolKind` in the `workspace/symbol` request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol_kind: Option<SymbolKindCapability>,

    /// The client supports tags on `SymbolInformation`.
    /// Clients supporting tags have to handle unknown tags gracefully.
    ///
    /// @since 3.16.0
    ///
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "TagSupport::deserialize_compat"
    )]
    #[cfg(feature = "proposed")]
    pub tag_support: Option<TagSupport<SymbolTag>>,
}

/// Workspace specific client capabilities.
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceClientCapabilities {
    /// The client supports applying batch edits to the workspace by supporting
    /// the request 'workspace/applyEdit'
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply_edit: Option<bool>,

    /// Capabilities specific to `WorkspaceEdit`s
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_edit: Option<WorkspaceEditCapability>,

    /// Capabilities specific to the `workspace/didChangeConfiguration` notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_change_configuration: Option<GenericCapability>,

    /// Capabilities specific to the `workspace/didChangeWatchedFiles` notification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_change_watched_files: Option<GenericCapability>,

    /// Capabilities specific to the `workspace/symbol` request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<WorkspaceSymbolClientCapabilities>,

    /// Capabilities specific to the `workspace/executeCommand` request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execute_command: Option<GenericCapability>,

    /// The client has support for workspace folders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_folders: Option<bool>,

    /// The client supports `workspace/configuration` requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SynchronizationCapability {
    /// Whether text document synchronization supports dynamic registration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /// The client supports sending will save notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_save: Option<bool>,

    /// The client supports sending a will save request and
    /// waits for a response providing text edits which will
    /// be applied to the document before it is saved.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_save_wait_until: Option<bool>,

    /// The client supports did save notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_save: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionItemCapability {
    /// Client supports snippets as insert text.
    ///
    /// A snippet can define tab stops and placeholders with `$1`, `$2`
    /// and `${3:foo}`. `$0` defines the final tab stop, it defaults to
    /// the end of the snippet. Placeholders with equal identifiers are linked,
    /// that is typing in one will update others too.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet_support: Option<bool>,

    /// Client supports commit characters on a completion item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_characters_support: Option<bool>,

    /// Client supports the follow content formats for the documentation
    /// property. The order describes the preferred format of the client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_format: Option<Vec<MarkupKind>>,

    /// Client supports the deprecated property on a completion item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated_support: Option<bool>,

    /// Client supports the preselect property on a completion item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preselect_support: Option<bool>,

    /// Client supports the tag property on a completion item. Clients supporting
    /// tags have to handle unknown tags gracefully. Clients especially need to
    /// preserve unknown tags when sending a completion item back to the server in
    /// a resolve call.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "TagSupport::deserialize_compat"
    )]
    pub tag_support: Option<TagSupport<CompletionItemTag>>,

    /// Client support insert replace edit to control different behavior if a
    /// completion item is inserted in the text or should replace text.
    ///
    /// @since 3.16.0 - Proposed state
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(feature = "proposed")]
    pub insert_replace_support: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum CompletionItemTag {
    Deprecated = 1,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionItemKindCapability {
    /// The completion item kind values the client supports. When this
    /// property exists the client also guarantees that it will
    /// handle values outside its set gracefully and falls back
    /// to a default value when unknown.
    ///
    /// If this property is not present the client only supports
    /// the completion items kinds from `Text` to `Reference` as defined in
    /// the initial version of the protocol.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_set: Option<Vec<CompletionItemKind>>,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HoverCapability {
    /// Whether completion supports dynamic registration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /// Client supports the follow content formats for the content
    /// property. The order describes the preferred format of the client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_format: Option<Vec<MarkupKind>>,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionCapability {
    /// Whether completion supports dynamic registration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /// The client supports the following `CompletionItem` specific
    /// capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_item: Option<CompletionItemCapability>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_item_kind: Option<CompletionItemKindCapability>,

    /// The client supports to send additional context information for a
    /// `textDocument/completion` requestion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_support: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureInformationSettings {
    /// Client supports the follow content formats for the documentation
    /// property. The order describes the preferred format of the client.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_format: Option<Vec<MarkupKind>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_information: Option<ParameterInformationSettings>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParameterInformationSettings {
    /// The client supports processing label offsets instead of a
    /// simple label string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_offset_support: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureHelpCapability {
    /// Whether completion supports dynamic registration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /// The client supports the following `SignatureInformation`
    /// specific properties.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_information: Option<SignatureInformationSettings>,

    /// The client supports to send additional context information for a
    /// `textDocument/signatureHelp` request. A client that opts into
    /// contextSupport will also support the `retriggerCharacters` on
    /// `SignatureHelpOptions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_support: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishDiagnosticsCapability {
    /// Whether the clients accepts diagnostics with related information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_information: Option<bool>,

    /// Client supports the tag property to provide meta data about a diagnostic.
    /// Clients supporting tags have to handle unknown tags gracefully.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "TagSupport::deserialize_compat"
    )]
    pub tag_support: Option<TagSupport<DiagnosticTag>>,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagSupport<T> {
    /// The tags supported by the client.
    pub value_set: Vec<T>,
}

impl<T> TagSupport<T> {
    /// Support for deserializing a boolean tag Support, in case it's present.
    ///
    /// This is currently the case for vscode 1.41.1
    fn deserialize_compat<'de, S>(serializer: S) -> Result<Option<TagSupport<T>>, S::Error>
    where
        S: serde::Deserializer<'de>,
        T: serde::Deserialize<'de>,
    {
        Ok(
            match Option::<Value>::deserialize(serializer).map_err(serde::de::Error::custom)? {
                Some(Value::Bool(false)) => None,
                Some(Value::Bool(true)) => Some(TagSupport { value_set: vec![] }),
                Some(other) => {
                    Some(TagSupport::<T>::deserialize(other).map_err(serde::de::Error::custom)?)
                }
                None => None,
            },
        )
    }
}

/// Text document specific client capabilities.
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentClientCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synchronization: Option<SynchronizationCapability>,
    /// Capabilities specific to the `textDocument/completion`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion: Option<CompletionCapability>,

    /// Capabilities specific to the `textDocument/hover`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hover: Option<HoverCapability>,

    /// Capabilities specific to the `textDocument/signatureHelp`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_help: Option<SignatureHelpCapability>,

    /// Capabilities specific to the `textDocument/references`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<GenericCapability>,

    /// Capabilities specific to the `textDocument/documentHighlight`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_highlight: Option<GenericCapability>,

    /// Capabilities specific to the `textDocument/documentSymbol`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_symbol: Option<DocumentSymbolClientCapabilities>,
    /// Capabilities specific to the `textDocument/formatting`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatting: Option<GenericCapability>,

    /// Capabilities specific to the `textDocument/rangeFormatting`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_formatting: Option<GenericCapability>,

    /// Capabilities specific to the `textDocument/onTypeFormatting`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_type_formatting: Option<GenericCapability>,

    /// Capabilities specific to the `textDocument/declaration`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaration: Option<GotoCapability>,

    /// Capabilities specific to the `textDocument/definition`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<GotoCapability>,

    /// Capabilities specific to the `textDocument/typeDefinition`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_definition: Option<GotoCapability>,

    /// Capabilities specific to the `textDocument/implementation`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation: Option<GotoCapability>,

    /// Capabilities specific to the `textDocument/codeAction`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_action: Option<CodeActionCapability>,

    /// Capabilities specific to the `textDocument/codeLens`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_lens: Option<GenericCapability>,

    /// Capabilities specific to the `textDocument/documentLink`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_link: Option<DocumentLinkCapabilities>,

    /// Capabilities specific to the `textDocument/documentColor` and the
    /// `textDocument/colorPresentation` request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_provider: Option<GenericCapability>,

    /// Capabilities specific to the `textDocument/rename`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rename: Option<RenameCapability>,

    /// Capabilities specific to `textDocument/publishDiagnostics`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publish_diagnostics: Option<PublishDiagnosticsCapability>,

    /// Capabilities specific to `textDocument/foldingRange` requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folding_range: Option<FoldingRangeCapability>,

    /// The client's semantic highlighting capability.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(feature = "proposed")]
    pub semantic_highlighting_capabilities: Option<SemanticHighlightingClientCapability>,

    /// Capabilities specific to the `textDocument/semanticTokens`
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(feature = "proposed")]
    pub semantic_tokens: Option<SemanticTokensClientCapabilities>,
}

/// Window specific client capabilities.
#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowClientCapabilities {
    /// Whether client supports create a work done progress UI from the server side.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_done_progress: Option<bool>,
}

/// Where ClientCapabilities are currently empty:
#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    /// Workspace specific client capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<WorkspaceClientCapabilities>,

    /// Text document specific client capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_document: Option<TextDocumentClientCapabilities>,

    /// Window specific client capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<WindowClientCapabilities>,

    /// Experimental client capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<Value>,
}

#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    /// The capabilities the language server provides.
    pub capabilities: ServerCapabilities,

    /// The capabilities the language server provides.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_info: Option<ServerInfo>,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
pub struct ServerInfo {
    /// The name of the server as defined by the server.
    pub name: String,
    /// The servers's version as defined by the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
pub struct InitializeError {
    /// Indicates whether the client should retry to send the
    /// initilize request after showing the message provided
    /// in the ResponseError.
    pub retry: bool,
}

// The server can signal the following capabilities:

/// Defines how the host (editor) should sync document changes to the language server.
#[derive(Debug, Eq, PartialEq, Clone, Copy, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum TextDocumentSyncKind {
    /// Documents should not be synced at all.
    None = 0,

    /// Documents are synced by always sending the full content of the document.
    Full = 1,

    /// Documents are synced by sending the full content on open. After that only
    /// incremental updates to the document are sent.
    Incremental = 2,
}

/// Completion options.
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionOptions {
    /// The server provides support to resolve additional information for a completion item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolve_provider: Option<bool>,

    /// The characters that trigger completion automatically.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_characters: Option<Vec<String>>,

    #[serde(flatten)]
    pub work_done_progress_options: WorkDoneProgressOptions,
}

/// Hover options.
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HoverOptions {
    #[serde(flatten)]
    pub work_done_progress_options: WorkDoneProgressOptions,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HoverRegistrationOptions {
    #[serde(flatten)]
    pub text_document_registration_options: TextDocumentRegistrationOptions,

    #[serde(flatten)]
    pub hover_options: HoverOptions,
}

/// Signature help options.
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureHelpOptions {
    /// The characters that trigger signature help automatically.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_characters: Option<Vec<String>>,

    ///  List of characters that re-trigger signature help.
    /// These trigger characters are only active when signature help is already showing. All trigger characters
    /// are also counted as re-trigger characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrigger_characters: Option<Vec<String>>,

    #[serde(flatten)]
    pub work_done_progress_options: WorkDoneProgressOptions,
}

/// Signature help options.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct SignatureHelpRegistrationOptions {
    #[serde(flatten)]
    pub text_document_registration_options: TextDocumentRegistrationOptions,
}
/// Signature help options.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum SignatureHelpTriggerKind {
    /// Signature help was invoked manually by the user or by a command.
    Invoked = 1,
    ///  Signature help was triggered by a trigger character.
    TriggerCharacter = 2,
    /// Signature help was triggered by the cursor moving or by the document content changing.
    ContentChange = 3,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureHelpParams {
    /// The signature help context. This is only available if the client specifies
    /// to send this using the client capability  `textDocument.signatureHelp.contextSupport === true`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<SignatureHelpContext>,

    #[serde(flatten)]
    pub text_document_position_params: TextDocumentPositionParams,

    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureHelpContext {
    ///  Action that caused signature help to be triggered.
    pub trigger_kind: SignatureHelpTriggerKind,

    /// Character that caused signature help to be triggered.
    /// This is undefined when `triggerKind !== SignatureHelpTriggerKind.TriggerCharacter`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_character: Option<String>,

    /// `true` if signature help was already showing when it was triggered.
    /// Retriggers occur when the signature help is already active and can be caused by actions such as
    /// typing a trigger character, a cursor move, or document content changes.
    pub is_retrigger: bool,

    /// The currently active `SignatureHelp`.
    /// The `activeSignatureHelp` has its `SignatureHelp.activeSignature` field updated based on
    /// the user navigating through available signatures.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_signature_help: Option<SignatureHelp>,
}

/// Code Lens options.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeLensOptions {
    /// Code lens has a resolve provider as well.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolve_provider: Option<bool>,
}

/// Format document on type options
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentOnTypeFormattingOptions {
    /// A character on which formatting should be triggered, like `}`.
    pub first_trigger_character: String,

    /// More trigger characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub more_trigger_character: Option<Vec<String>>,
}

/// Execute command options.
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
pub struct ExecuteCommandOptions {
    /// The commands to be executed on the server
    pub commands: Vec<String>,

    #[serde(flatten)]
    pub work_done_progress_options: WorkDoneProgressOptions,
}

/// Save options.
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveOptions {
    /// The client is supposed to include the content on save.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_text: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TextDocumentSyncSaveOptions {
    Supported(bool),
    SaveOptions(SaveOptions),
}

impl From<SaveOptions> for TextDocumentSyncSaveOptions {
    fn from(from: SaveOptions) -> Self {
        Self::SaveOptions(from)
    }
}

impl From<bool> for TextDocumentSyncSaveOptions {
    fn from(from: bool) -> Self {
        Self::Supported(from)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentSyncOptions {
    /// Open and close notifications are sent to the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_close: Option<bool>,

    /// Change notifications are sent to the server. See TextDocumentSyncKind.None, TextDocumentSyncKind.Full
    /// and TextDocumentSyncKindIncremental.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change: Option<TextDocumentSyncKind>,

    /// Will save notifications are sent to the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_save: Option<bool>,

    /// Will save wait until requests are sent to the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_save_wait_until: Option<bool>,

    /// Save notifications are sent to the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save: Option<TextDocumentSyncSaveOptions>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TextDocumentSyncCapability {
    Kind(TextDocumentSyncKind),
    Options(TextDocumentSyncOptions),
}

impl From<TextDocumentSyncOptions> for TextDocumentSyncCapability {
    fn from(from: TextDocumentSyncOptions) -> Self {
        Self::Options(from)
    }
}

impl From<TextDocumentSyncKind> for TextDocumentSyncCapability {
    fn from(from: TextDocumentSyncKind) -> Self {
        Self::Kind(from)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ImplementationProviderCapability {
    Simple(bool),
    Options(StaticTextDocumentRegistrationOptions),
}

impl From<StaticTextDocumentRegistrationOptions> for ImplementationProviderCapability {
    fn from(from: StaticTextDocumentRegistrationOptions) -> Self {
        Self::Options(from)
    }
}

impl From<bool> for ImplementationProviderCapability {
    fn from(from: bool) -> Self {
        Self::Simple(from)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TypeDefinitionProviderCapability {
    Simple(bool),
    Options(StaticTextDocumentRegistrationOptions),
}

impl From<StaticTextDocumentRegistrationOptions> for TypeDefinitionProviderCapability {
    fn from(from: StaticTextDocumentRegistrationOptions) -> Self {
        Self::Options(from)
    }
}

impl From<bool> for TypeDefinitionProviderCapability {
    fn from(from: bool) -> Self {
        Self::Simple(from)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum HoverProviderCapability {
    Simple(bool),
    Options(HoverOptions),
}

impl From<HoverOptions> for HoverProviderCapability {
    fn from(from: HoverOptions) -> Self {
        Self::Options(from)
    }
}

impl From<bool> for HoverProviderCapability {
    fn from(from: bool) -> Self {
        Self::Simple(from)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ColorProviderCapability {
    Simple(bool),
    ColorProvider(ColorProviderOptions),
    Options(StaticTextDocumentColorProviderOptions),
}

impl From<ColorProviderOptions> for ColorProviderCapability {
    fn from(from: ColorProviderOptions) -> Self {
        Self::ColorProvider(from)
    }
}

impl From<StaticTextDocumentColorProviderOptions> for ColorProviderCapability {
    fn from(from: StaticTextDocumentColorProviderOptions) -> Self {
        Self::Options(from)
    }
}

impl From<bool> for ColorProviderCapability {
    fn from(from: bool) -> Self {
        Self::Simple(from)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CodeActionProviderCapability {
    Simple(bool),
    Options(CodeActionOptions),
}

impl From<CodeActionOptions> for CodeActionProviderCapability {
    fn from(from: CodeActionOptions) -> Self {
        Self::Options(from)
    }
}

impl From<bool> for CodeActionProviderCapability {
    fn from(from: bool) -> Self {
        Self::Simple(from)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeActionCapability {
    ///
    /// This capability supports dynamic registration.
    ///
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /// The client support code action literals as a valid
    /// response of the `textDocument/codeAction` request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_action_literal_support: Option<CodeActionLiteralSupport>,

    /// Whether code action supports the `isPreferred` property.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_preferred_support: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeActionLiteralSupport {
    /// The code action kind is support with the following value set.
    pub code_action_kind: CodeActionKindLiteralSupport,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeActionKindLiteralSupport {
    /// The code action kind values the client supports. When this
    /// property exists the client also guarantees that it will
    /// handle values outside its set gracefully and falls back
    /// to a default value when unknown.
    pub value_set: Vec<String>,
}

#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    /// Defines how text documents are synced.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_document_sync: Option<TextDocumentSyncCapability>,

    /// Capabilities specific to `textDocument/selectionRange` requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection_range_provider: Option<SelectionRangeProviderCapability>,

    /// The server provides hover support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hover_provider: Option<HoverProviderCapability>,

    /// The server provides completion support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_provider: Option<CompletionOptions>,

    /// The server provides signature help support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_help_provider: Option<SignatureHelpOptions>,

    /// The server provides goto definition support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition_provider: Option<bool>,

    /// The server provides goto type definition support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_definition_provider: Option<TypeDefinitionProviderCapability>,

    /// the server provides goto implementation support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation_provider: Option<ImplementationProviderCapability>,

    /// The server provides find references support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references_provider: Option<bool>,

    /// The server provides document highlight support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_highlight_provider: Option<bool>,

    /// The server provides document symbol support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_symbol_provider: Option<bool>,

    /// The server provides workspace symbol support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_symbol_provider: Option<bool>,

    /// The server provides code actions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_action_provider: Option<CodeActionProviderCapability>,

    /// The server provides code lens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_lens_provider: Option<CodeLensOptions>,

    /// The server provides document formatting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_formatting_provider: Option<bool>,

    /// The server provides document range formatting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_range_formatting_provider: Option<bool>,

    /// The server provides document formatting on typing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_on_type_formatting_provider: Option<DocumentOnTypeFormattingOptions>,

    /// The server provides rename support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rename_provider: Option<RenameProviderCapability>,

    /// The server provides document link support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_link_provider: Option<DocumentLinkOptions>,

    /// The server provides color provider support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_provider: Option<ColorProviderCapability>,

    /// The server provides folding provider support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folding_range_provider: Option<FoldingRangeProviderCapability>,

    /// The server provides go to declaration support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaration_provider: Option<bool>,

    /// The server provides execute command support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execute_command_provider: Option<ExecuteCommandOptions>,

    /// Workspace specific server capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<WorkspaceCapability>,

    /// Semantic highlighting server capabilities.
    #[cfg(feature = "proposed")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_highlighting: Option<SemanticHighlightingServerCapability>,

    /// Call hierarchy provider capabilities.
    #[cfg(feature = "proposed")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_hierarchy_provider: Option<CallHierarchyServerCapability>,

    /// Semantic tokens server capabilities.
    #[cfg(feature = "proposed")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_tokens_provider: Option<SemanticTokensServerCapabilities>,

    /// Experimental server capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<Value>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentLinkCapabilities {
    /// Whether document link supports dynamic registration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /// Whether the client support the `tooltip` property on `DocumentLink`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tooltip_support: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct ShowMessageParams {
    /// The message type. See {@link MessageType}.
    #[serde(rename = "type")]
    pub typ: MessageType,

    /// The actual message.
    pub message: String,
}

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

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct MessageActionItem {
    /// A short title like 'Retry', 'Open Log' etc.
    pub title: String,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct LogMessageParams {
    /// The message type. See {@link MessageType}
    #[serde(rename = "type")]
    pub typ: MessageType,

    /// The actual message
    pub message: String,
}

/// General parameters to to register for a capability.
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Registration {
    /// The id used to register the request. The id can be used to deregister
    /// the request again.
    pub id: String,

    /// The method / capability to register for.
    pub method: String,

    /// Options necessary for the registration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub register_options: Option<Value>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct RegistrationParams {
    pub registrations: Vec<Registration>,
}

/// Since most of the registration options require to specify a document selector there is a base
/// interface that can be used.
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentRegistrationOptions {
    /// A document selector to identify the scope of the registration. If set to null
    /// the document selector provided on the client side will be used.
    pub document_selector: Option<DocumentSelector>,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticRegistrationOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticTextDocumentRegistrationOptions {
    /// A document selector to identify the scope of the registration. If set to null
    /// the document selector provided on the client side will be used.
    pub document_selector: Option<DocumentSelector>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorProviderOptions {}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticTextDocumentColorProviderOptions {
    /// A document selector to identify the scope of the registration. If set to null
    /// the document selector provided on the client side will be used.
    pub document_selector: Option<DocumentSelector>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// General parameters to unregister a capability.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct Unregistration {
    /// The id used to unregister the request or notification. Usually an id
    /// provided during the register request.
    pub id: String,

    /// The method / capability to unregister for.
    pub method: String,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct UnregistrationParams {
    pub unregisterations: Vec<Unregistration>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct DidChangeConfigurationParams {
    /// The actual changed settings
    pub settings: Value,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DidOpenTextDocumentParams {
    /// The document that was opened.
    pub text_document: TextDocumentItem,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DidChangeTextDocumentParams {
    /// The document that did change. The version number points
    /// to the version after all provided content changes have
    /// been applied.
    pub text_document: VersionedTextDocumentIdentifier,
    /// The actual content changes.
    pub content_changes: Vec<TextDocumentContentChangeEvent>,
}

/// An event describing a change to a text document. If range and rangeLength are omitted
/// the new text is considered to be the full content of the document.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentContentChangeEvent {
    /// The range of the document that changed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<Range>,

    /// The length of the range that got replaced.
    /// NOTE: seems redundant, see: <https://github.com/Microsoft/language-server-protocol/issues/9>
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_length: Option<u64>,

    /// The new text of the document.
    pub text: String,
}

/// Descibe options to be used when registered for text document change events.
///
/// Extends TextDocumentRegistrationOptions
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentChangeRegistrationOptions {
    /// A document selector to identify the scope of the registration. If set to null
    /// the document selector provided on the client side will be used.
    pub document_selector: Option<DocumentSelector>,

    /// How documents are synced to the server. See TextDocumentSyncKind.Full
    /// and TextDocumentSyncKindIncremental.
    pub sync_kind: i32,
}

/// The parameters send in a will save text document notification.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WillSaveTextDocumentParams {
    /// The document that will be saved.
    pub text_document: TextDocumentIdentifier,

    /// The 'TextDocumentSaveReason'.
    pub reason: TextDocumentSaveReason,
}

/// Represents reasons why a text document is saved.
#[derive(Copy, Debug, Eq, PartialEq, Clone, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum TextDocumentSaveReason {
    /// Manually triggered, e.g. by the user pressing save, by starting debugging,
    /// or by an API call.
    Manual = 1,

    /// Automatic after a delay.
    AfterDelay = 2,

    /// When the editor lost focus.
    FocusOut = 3,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DidCloseTextDocumentParams {
    /// The document that was closed.
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DidSaveTextDocumentParams {
    /// The document that was saved.
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentSaveRegistrationOptions {
    /// The client is supposed to include the content on save.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_text: Option<bool>,

    #[serde(flatten)]
    pub text_document_registration_options: TextDocumentRegistrationOptions,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct DidChangeWatchedFilesParams {
    /// The actual file events.
    pub changes: Vec<FileEvent>,
}

/// The file event type.
#[derive(Debug, Eq, PartialEq, Copy, Clone, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum FileChangeType {
    /// The file got created.
    Created = 1,

    /// The file got changed.
    Changed = 2,

    /// The file got deleted.
    Deleted = 3,
}

/// An event describing a file change.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct FileEvent {
    /// The file's URI.
    pub uri: Url,

    /// The change type.
    #[serde(rename = "type")]
    pub typ: FileChangeType,
}

impl FileEvent {
    pub fn new(uri: Url, typ: FileChangeType) -> FileEvent {
        FileEvent { uri, typ }
    }
}

/// Describe options to be used when registered for text document change events.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Deserialize, Serialize)]
pub struct DidChangeWatchedFilesRegistrationOptions {
    /// The watchers to register.
    pub watchers: Vec<FileSystemWatcher>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSystemWatcher {
    /// The  glob pattern to watch
    pub glob_pattern: String,

    /// The kind of events of interest. If omitted it defaults to WatchKind.Create |
    /// WatchKind.Change | WatchKind.Delete which is 7.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<WatchKind>,
}

bitflags! {
pub struct WatchKind: u8 {
    /// Interested in create events.
    const Create = 1;
    /// Interested in change events
    const Change = 2;
    /// Interested in delete events
    const Delete = 4;
}
}

impl<'de> serde::Deserialize<'de> for WatchKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let i = u8::deserialize(deserializer)?;
        WatchKind::from_bits(i).ok_or_else(|| {
            D::Error::invalid_value(de::Unexpected::Unsigned(u64::from(i)), &"Unknown flag")
        })
    }
}

impl serde::Serialize for WatchKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(self.bits())
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct PublishDiagnosticsParams {
    /// The URI for which diagnostic information is reported.
    pub uri: Url,

    /// An array of diagnostic information items.
    pub diagnostics: Vec<Diagnostic>,

    /// Optional the version number of the document the diagnostics are published for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i64>,
}

impl PublishDiagnosticsParams {
    pub fn new(
        uri: Url,
        diagnostics: Vec<Diagnostic>,
        version: Option<i64>,
    ) -> PublishDiagnosticsParams {
        PublishDiagnosticsParams {
            uri,
            diagnostics,
            version,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CompletionRegistrationOptions {
    #[serde(flatten)]
    pub text_document_registration_options: TextDocumentRegistrationOptions,

    #[serde(flatten)]
    pub completion_options: CompletionOptions,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CompletionResponse {
    Array(Vec<CompletionItem>),
    List(CompletionList),
}

impl From<Vec<CompletionItem>> for CompletionResponse {
    fn from(items: Vec<CompletionItem>) -> Self {
        CompletionResponse::Array(items)
    }
}

impl From<CompletionList> for CompletionResponse {
    fn from(list: CompletionList) -> Self {
        CompletionResponse::List(list)
    }
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionParams {
    // This field was "mixed-in" from TextDocumentPositionParams
    #[serde(flatten)]
    pub text_document_position: TextDocumentPositionParams,

    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,

    #[serde(flatten)]
    pub partial_result_params: PartialResultParams,

    // CompletionParams properties:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<CompletionContext>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionContext {
    /// How the completion was triggered.
    pub trigger_kind: CompletionTriggerKind,

    /// The trigger character (a single character) that has trigger code complete.
    /// Is undefined if `triggerKind !== CompletionTriggerKind.TriggerCharacter`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_character: Option<String>,
}

/// How a completion was triggered.
#[derive(Debug, PartialEq, Clone, Copy, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum CompletionTriggerKind {
    Invoked = 1,
    TriggerCharacter = 2,
    TriggerForIncompleteCompletions = 3,
}

/// Represents a collection of [completion items](#CompletionItem) to be presented
/// in the editor.
#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionList {
    /// This list it not complete. Further typing should result in recomputing
    /// this list.
    pub is_incomplete: bool,

    /// The completion items.
    pub items: Vec<CompletionItem>,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum Documentation {
    String(String),
    MarkupContent(MarkupContent),
}

#[derive(Debug, PartialEq, Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompletionItem {
    /// The label of this completion item. By default
    /// also the text that is inserted when selecting
    /// this completion.
    pub label: String,

    /// The kind of this completion item. Based of the kind
    /// an icon is chosen by the editor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<CompletionItemKind>,

    /// A human-readable string with additional information
    /// about this item, like type or symbol information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,

    /// A human-readable string that represents a doc-comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<Documentation>,

    /// Indicates if this item is deprecated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,

    /// Select this item when showing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preselect: Option<bool>,

    /// A string that shoud be used when comparing this item
    /// with other items. When `falsy` the label is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_text: Option<String>,

    /// A string that should be used when filtering a set of
    /// completion items. When `falsy` the label is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_text: Option<String>,

    /// A string that should be inserted a document when selecting
    /// this completion. When `falsy` the label is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_text: Option<String>,

    /// The format of the insert text. The format applies to both the `insertText` property
    /// and the `newText` property of a provided `textEdit`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_text_format: Option<InsertTextFormat>,

    /// An edit which is applied to a document when selecting
    /// this completion. When an edit is provided the value of
    /// insertText is ignored.
    ///
    /// Most editors support two different operation when accepting a completion item. One is to insert a
    /// completion text and the other is to replace an existing text with a competion text. Since this can
    /// usually not predetermend by a server it can report both ranges. Clients need to signal support for
    /// `InsertReplaceEdits` via the `textDocument.completion.insertReplaceSupport` client capability
    /// property.
    ///
    /// *Note 1:* The text edit's range as well as both ranges from a insert replace edit must be a
    /// [single line] and they must contain the position at which completion has been requested.
    /// *Note 2:* If an `InsertReplaceEdit` is returned the edit's insert range must be a prefix of
    /// the edit's replace range, that means it must be contained and starting at the same position.
    ///
    /// @since 3.16.0 additional type `InsertReplaceEdit` - Proposed state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_edit: Option<CompletionTextEdit>,

    /// An optional array of additional text edits that are applied when
    /// selecting this completion. Edits must not overlap with the main edit
    /// nor with themselves.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_text_edits: Option<Vec<TextEdit>>,

    /// An optional command that is executed *after* inserting this completion. *Note* that
    /// additional modifications to the current document should be described with the
    /// additionalTextEdits-property.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Command>,

    /// An data entry field that is preserved on a completion item between
    /// a completion and a completion resolve request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,

    /// Tags for this completion item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<CompletionItemTag>>,
}

impl CompletionItem {
    /// Create a CompletionItem with the minimum possible info (label and detail).
    pub fn new_simple(label: String, detail: String) -> CompletionItem {
        CompletionItem {
            label,
            detail: Some(detail),
            ..Self::default()
        }
    }
}

/// The kind of a completion entry.
#[derive(Debug, Eq, PartialEq, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum CompletionItemKind {
    Text = 1,
    Method = 2,
    Function = 3,
    Constructor = 4,
    Field = 5,
    Variable = 6,
    Class = 7,
    Interface = 8,
    Module = 9,
    Property = 10,
    Unit = 11,
    Value = 12,
    Enum = 13,
    Keyword = 14,
    Snippet = 15,
    Color = 16,
    File = 17,
    Reference = 18,
    Folder = 19,
    EnumMember = 20,
    Constant = 21,
    Struct = 22,
    Event = 23,
    Operator = 24,
    TypeParameter = 25,
}

/// Defines how to interpret the insert text in a completion item
#[derive(Debug, Eq, PartialEq, Clone, Copy, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum InsertTextFormat {
    PlainText = 1,
    Snippet = 2,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HoverParams {
    #[serde(flatten)]
    pub text_document_position_params: TextDocumentPositionParams,

    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,
}

/// The result of a hover request.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct Hover {
    /// The hover's content
    pub contents: HoverContents,
    /// An optional range is a range inside a text document
    /// that is used to visualize a hover, e.g. by changing the background color.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<Range>,
}

/// Hover contents could be single entry or multiple entries.
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HoverContents {
    Scalar(MarkedString),
    Array(Vec<MarkedString>),
    Markup(MarkupContent),
}

/// The marked string is rendered:
/// - as markdown if it is represented as a string
/// - as code block of the given langauge if it is represented as a pair of a language and a value
///
/// The pair of a language and a value is an equivalent to markdown:
///     ```${language}
///     ${value}
///     ```
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MarkedString {
    String(String),
    LanguageString(LanguageString),
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct LanguageString {
    pub language: String,
    pub value: String,
}

impl MarkedString {
    pub fn from_markdown(markdown: String) -> MarkedString {
        MarkedString::String(markdown)
    }

    pub fn from_language_code(language: String, code_block: String) -> MarkedString {
        MarkedString::LanguageString(LanguageString {
            language,
            value: code_block,
        })
    }
}

/// Signature help represents the signature of something
/// callable. There can be multiple signature but only one
/// active and only one active parameter.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureHelp {
    /// One or more signatures.
    pub signatures: Vec<SignatureInformation>,

    /// The active signature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_signature: Option<i64>,

    /// The active parameter of the active signature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_parameter: Option<i64>,
}

/// Represents the signature of something callable. A signature
/// can have a label, like a function-name, a doc-comment, and
/// a set of parameters.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct SignatureInformation {
    /// The label of this signature. Will be shown in
    /// the UI.
    pub label: String,

    /// The human-readable doc-comment of this signature. Will be shown
    /// in the UI but can be omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<Documentation>,

    /// The parameters of this signature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<ParameterInformation>>,
}

/// Represents a parameter of a callable-signature. A parameter can
/// have a label and a doc-comment.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct ParameterInformation {
    /// The label of this parameter information.
    ///
    /// Either a string or an inclusive start and exclusive end offsets within its containing
    /// signature label. (see SignatureInformation.label). *Note*: A label of type string must be
    /// a substring of its containing signature label.
    pub label: ParameterLabel,

    /// The human-readable doc-comment of this parameter. Will be shown
    /// in the UI but can be omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<Documentation>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ParameterLabel {
    Simple(String),
    LabelOffsets([u64; 2]),
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GotoDefinitionParams {
    #[serde(flatten)]
    pub text_document_position_params: TextDocumentPositionParams,

    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,

    #[serde(flatten)]
    pub partial_result_params: PartialResultParams,
}

/// GotoDefinition response can be single location, or multiple Locations or a link.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GotoDefinitionResponse {
    Scalar(Location),
    Array(Vec<Location>),
    Link(Vec<LocationLink>),
}

impl From<Location> for GotoDefinitionResponse {
    fn from(location: Location) -> Self {
        GotoDefinitionResponse::Scalar(location)
    }
}

impl From<Vec<Location>> for GotoDefinitionResponse {
    fn from(locations: Vec<Location>) -> Self {
        GotoDefinitionResponse::Array(locations)
    }
}

impl From<Vec<LocationLink>> for GotoDefinitionResponse {
    fn from(locations: Vec<LocationLink>) -> Self {
        GotoDefinitionResponse::Link(locations)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceParams {
    // Text Document and Position fields
    #[serde(flatten)]
    pub text_document_position: TextDocumentPositionParams,

    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,

    #[serde(flatten)]
    pub partial_result_params: PartialResultParams,

    // ReferenceParams properties:
    pub context: ReferenceContext,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceContext {
    /// Include the declaration of the current symbol.
    pub include_declaration: bool,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentHighlightParams {
    #[serde(flatten)]
    pub text_document_position_params: TextDocumentPositionParams,

    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,

    #[serde(flatten)]
    pub partial_result_params: PartialResultParams,
}

/// A document highlight is a range inside a text document which deserves
/// special attention. Usually a document highlight is visualized by changing
/// the background color of its range.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct DocumentHighlight {
    /// The range this highlight applies to.
    pub range: Range,

    /// The highlight kind, default is DocumentHighlightKind.Text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<DocumentHighlightKind>,
}

/// A document highlight kind.
#[derive(Debug, Eq, PartialEq, Copy, Clone, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
pub enum DocumentHighlightKind {
    /// A textual occurrance.
    Text = 1,

    /// Read-access of a symbol, like reading a variable.
    Read = 2,

    /// Write-access of a symbol, like writing to a variable.
    Write = 3,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSymbolClientCapabilities {
    /// This capability supports dynamic registration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /// Specific capabilities for the `SymbolKind`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol_kind: Option<SymbolKindCapability>,

    /// The client support hierarchical document symbols.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hierarchical_document_symbol_support: Option<bool>,

    /// The client supports tags on `SymbolInformation`. Tags are supported on
    /// `DocumentSymbol` if `hierarchicalDocumentSymbolSupport` is set to true.
    /// Clients supporting tags have to handle unknown tags gracefully.
    ///
    /// @since 3.16.0
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "TagSupport::deserialize_compat"
    )]
    #[cfg(feature = "proposed")]
    pub tag_support: Option<TagSupport<SymbolTag>>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DocumentSymbolResponse {
    Flat(Vec<SymbolInformation>),
    Nested(Vec<DocumentSymbol>),
}

impl From<Vec<SymbolInformation>> for DocumentSymbolResponse {
    fn from(info: Vec<SymbolInformation>) -> Self {
        DocumentSymbolResponse::Flat(info)
    }
}

impl From<Vec<DocumentSymbol>> for DocumentSymbolResponse {
    fn from(symbols: Vec<DocumentSymbol>) -> Self {
        DocumentSymbolResponse::Nested(symbols)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSymbolParams {
    /// The text document.
    pub text_document: TextDocumentIdentifier,

    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,

    #[serde(flatten)]
    pub partial_result_params: PartialResultParams,
}

/// Represents programming constructs like variables, classes, interfaces etc.
/// that appear in a document. Document symbols can be hierarchical and they have two ranges:
/// one that encloses its definition and one that points to its most interesting range,
/// e.g. the range of an identifier.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSymbol {
    /// The name of this symbol.
    pub name: String,
    /// More detail for this symbol, e.g the signature of a function. If not provided the
    /// name is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// The kind of this symbol.
    pub kind: SymbolKind,
    /// Tags for this completion item.
    ///  since 3.16.0
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(feature = "proposed")]
    pub tags: Option<Vec<SymbolTag>>,
    /// Indicates if this symbol is deprecated.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[deprecated(note = "Use tags instead")]
    pub deprecated: Option<bool>,
    /// The range enclosing this symbol not including leading/trailing whitespace but everything else
    /// like comments. This information is typically used to determine if the the clients cursor is
    /// inside the symbol to reveal in the symbol in the UI.
    pub range: Range,
    /// The range that should be selected and revealed when this symbol is being picked, e.g the name of a function.
    /// Must be contained by the the `range`.
    pub selection_range: Range,
    /// Children of this symbol, e.g. properties of a class.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<DocumentSymbol>>,
}

/// Represents information about programming constructs like variables, classes,
/// interfaces etc.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolInformation {
    /// The name of this symbol.
    pub name: String,

    /// The kind of this symbol.
    pub kind: SymbolKind,

    /// Tags for this completion item.
    ///  since 3.16.0
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg(feature = "proposed")]
    pub tags: Option<Vec<SymbolTag>>,

    /// Indicates if this symbol is deprecated.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[deprecated(note = "Use tags instead")]
    pub deprecated: Option<bool>,

    /// The location of this symbol.
    pub location: Location,

    /// The name of the symbol containing this symbol.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_name: Option<String>,
}

/// A symbol kind.
#[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum SymbolKind {
    File = 1,
    Module = 2,
    Namespace = 3,
    Package = 4,
    Class = 5,
    Method = 6,
    Property = 7,
    Field = 8,
    Constructor = 9,
    Enum = 10,
    Interface = 11,
    Function = 12,
    Variable = 13,
    Constant = 14,
    String = 15,
    Number = 16,
    Boolean = 17,
    Array = 18,
    Object = 19,
    Key = 20,
    Null = 21,
    EnumMember = 22,
    Struct = 23,
    Event = 24,
    Operator = 25,
    TypeParameter = 26,

    // Capturing all unknown enums by this lib.
    Unknown = 255,
}

/// The parameters of a Workspace Symbol Request.
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
pub struct WorkspaceSymbolParams {
    #[serde(flatten)]
    pub partial_result_params: PartialResultParams,

    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,

    /// A non-empty query string
    pub query: String,
}

#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
pub struct ExecuteCommandParams {
    /// The identifier of the actual command handler.
    pub command: String,
    /// Arguments that the command should be invoked with.
    #[serde(default)]
    pub arguments: Vec<Value>,

    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,
}

/// Execute command registration options.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct ExecuteCommandRegistrationOptions {
    /// The commands to be executed on the server
    pub commands: Vec<String>,

    #[serde(flatten)]
    pub execute_command_options: ExecuteCommandOptions,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct ApplyWorkspaceEditParams {
    /// The edits to apply.
    pub edit: WorkspaceEdit,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct ApplyWorkspaceEditResponse {
    /// Indicates whether the edit was applied or not.
    pub applied: bool,
}

/// Params for the CodeActionRequest
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeActionParams {
    /// The document in which the command was invoked.
    pub text_document: TextDocumentIdentifier,

    /// The range for which the command was invoked.
    pub range: Range,

    /// Context carrying additional information.
    pub context: CodeActionContext,

    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,

    #[serde(flatten)]
    pub partial_result_params: PartialResultParams,
}

/// response for CodeActionRequest
pub type CodeActionResponse = Vec<CodeActionOrCommand>;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CodeActionOrCommand {
    Command(Command),
    CodeAction(CodeAction),
}

impl From<Command> for CodeActionOrCommand {
    fn from(comand: Command) -> Self {
        CodeActionOrCommand::Command(comand)
    }
}

impl From<CodeAction> for CodeActionOrCommand {
    fn from(action: CodeAction) -> Self {
        CodeActionOrCommand::CodeAction(action)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, PartialOrd, Clone, Deserialize, Serialize)]
pub struct CodeActionKind(Cow<'static, str>);

impl CodeActionKind {
    /// Empty kind.
    pub const EMPTY: CodeActionKind = CodeActionKind::new("");

    /// Base kind for quickfix actions: 'quickfix'
    pub const QUICKFIX: CodeActionKind = CodeActionKind::new("quickfix");

    /// Base kind for refactoring actions: 'refactor'
    pub const REFACTOR: CodeActionKind = CodeActionKind::new("refactor");

    /// Base kind for refactoring extraction actions: 'refactor.extract'
    ///
    /// Example extract actions:
    ///
    /// - Extract method
    /// - Extract function
    /// - Extract variable
    /// - Extract interface from class
    /// - ...
    pub const REFACTOR_EXTRACT: CodeActionKind = CodeActionKind::new("refactor.extract");

    /// Base kind for refactoring inline actions: 'refactor.inline'
    ///
    /// Example inline actions:
    ///
    /// - Inline function
    /// - Inline variable
    /// - Inline constant
    /// - ...
    pub const REFACTOR_INLINE: CodeActionKind = CodeActionKind::new("refactor.inline");

    /// Base kind for refactoring rewrite actions: 'refactor.rewrite'
    ///
    /// Example rewrite actions:
    ///
    /// - Convert JavaScript function to class
    /// - Add or remove parameter
    /// - Encapsulate field
    /// - Make method static
    /// - Move method to base class
    /// - ...
    pub const REFACTOR_REWRITE: CodeActionKind = CodeActionKind::new("refactor.rewrite");

    /// Base kind for source actions: `source`
    ///
    /// Source code actions apply to the entire file.
    pub const SOURCE: CodeActionKind = CodeActionKind::new("source");

    /// Base kind for an organize imports source action: `source.organizeImports`
    pub const SOURCE_ORGANIZE_IMPORTS: CodeActionKind =
        CodeActionKind::new("source.organizeImports");

    pub const fn new(tag: &'static str) -> Self {
        CodeActionKind(Cow::Borrowed(tag))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for CodeActionKind {
    fn from(from: String) -> Self {
        CodeActionKind(Cow::from(from))
    }
}

impl From<&'static str> for CodeActionKind {
    fn from(from: &'static str) -> Self {
        CodeActionKind::new(from)
    }
}

#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeAction {
    /// A short, human-readable, title for this code action.
    pub title: String,

    /// The kind of the code action.
    /// Used to filter code actions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<CodeActionKind>,

    /// The diagnostics that this code action resolves.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Vec<Diagnostic>>,

    /// The workspace edit this code action performs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit: Option<WorkspaceEdit>,

    /// A command this code action executes. If a code action
    /// provides an edit and a command, first the edit is
    /// executed and then the command.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Command>,

    /// Marks this as a preferred action. Preferred actions are used by the `auto fix` command and can be targeted
    /// by keybindings.
    /// A quick fix should be marked preferred if it properly addresses the underlying error.
    /// A refactoring should be marked preferred if it is the most reasonable choice of actions to take.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_preferred: Option<bool>,
}

/// Contains additional diagnostic information about the context in which
/// a code action is run.
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
pub struct CodeActionContext {
    /// An array of diagnostics.
    pub diagnostics: Vec<Diagnostic>,

    /// Requested kind of actions to return.
    ///
    /// Actions not of this kind are filtered out by the client before being shown. So servers
    /// can omit computing them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only: Option<Vec<CodeActionKind>>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeActionOptions {
    /// CodeActionKinds that this server may return.
    ///
    /// The list of kinds may be generic, such as `CodeActionKind.Refactor`, or the server
    /// may list out every specific kind they provide.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_action_kinds: Option<Vec<CodeActionKind>>,

    #[serde(flatten)]
    pub work_done_progress_options: WorkDoneProgressOptions,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeLensParams {
    /// The document to request code lens for.
    pub text_document: TextDocumentIdentifier,

    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,

    #[serde(flatten)]
    pub partial_result_params: PartialResultParams,
}

/// A code lens represents a command that should be shown along with
/// source text, like the number of references, a way to run tests, etc.
///
/// A code lens is _unresolved_ when no command is associated to it. For performance
/// reasons the creation of a code lens and resolving should be done in two stages.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CodeLens {
    /// The range in which this code lens is valid. Should only span a single line.
    pub range: Range,

    /// The command this code lens represents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Command>,

    /// A data entry field that is preserved on a code lens item between
    /// a code lens and a code lens resolve request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentLinkParams {
    /// The document to provide document links for.
    pub text_document: TextDocumentIdentifier,

    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,

    #[serde(flatten)]
    pub partial_result_params: PartialResultParams,
}

/// A document link is a range in a text document that links to an internal or external resource, like another
/// text document or a web site.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct DocumentLink {
    /// The range this link applies to.
    pub range: Range,
    /// The uri this link points to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<Url>,

    /// The tooltip text when you hover over this link.
    ///
    /// If a tooltip is provided, is will be displayed in a string that includes instructions on how to
    /// trigger the link, such as `{0} (ctrl + click)`. The specific instructions vary depending on OS,
    /// user settings, and localization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<String>,

    /// A data entry field that is preserved on a document link between a DocumentLinkRequest
    /// and a DocumentLinkResolveRequest.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentFormattingParams {
    /// The document to format.
    pub text_document: TextDocumentIdentifier,

    /// The format options.
    pub options: FormattingOptions,

    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,
}

/// Value-object describing what options formatting should use.
#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormattingOptions {
    /// Size of a tab in spaces.
    pub tab_size: u64,

    /// Prefer spaces over tabs.
    pub insert_spaces: bool,

    /// Signature for further properties.
    #[serde(flatten)]
    pub properties: HashMap<String, FormattingProperty>,

    /// Trim trailing whitespaces on a line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trim_trailing_whitespace: Option<bool>,

    /// Insert a newline character at the end of the file if one does not exist.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insert_final_newline: Option<bool>,

    /// Trim all newlines after the final newline at the end of the file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trim_final_newlines: Option<bool>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum FormattingProperty {
    Bool(bool),
    Number(f64),
    String(String),
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentRangeFormattingParams {
    /// The document to format.
    pub text_document: TextDocumentIdentifier,

    /// The range to format
    pub range: Range,

    /// The format options
    pub options: FormattingOptions,

    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentOnTypeFormattingParams {
    /// Text Document and Position fields.
    #[serde(flatten)]
    pub text_document_position: TextDocumentPositionParams,

    /// The character that has been typed.
    pub ch: String,

    /// The format options.
    pub options: FormattingOptions,
}

/// Extends TextDocumentRegistrationOptions
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentOnTypeFormattingRegistrationOptions {
    /// A document selector to identify the scope of the registration. If set to null
    /// the document selector provided on the client side will be used.
    pub document_selector: Option<DocumentSelector>,

    /// A character on which formatting should be triggered, like `}`.
    pub first_trigger_character: String,

    /// More trigger characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub more_trigger_character: Option<Vec<String>>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameParams {
    /// Text Document and Position fields
    #[serde(flatten)]
    pub text_document_position: TextDocumentPositionParams,

    /// The new name of the symbol. If the given name is not valid the
    /// request must return a [ResponseError](#ResponseError) with an
    /// appropriate message set.
    pub new_name: String,

    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RenameProviderCapability {
    Simple(bool),
    Options(RenameOptions),
}

impl From<RenameOptions> for RenameProviderCapability {
    fn from(from: RenameOptions) -> Self {
        Self::Options(from)
    }
}

impl From<bool> for RenameProviderCapability {
    fn from(from: bool) -> Self {
        Self::Simple(from)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameOptions {
    /// Renames should be checked and tested before being executed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepare_provider: Option<bool>,

    #[serde(flatten)]
    pub work_done_progress_options: WorkDoneProgressOptions,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameCapability {
    /// Whether rename supports dynamic registration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /// Client supports testing for validity of rename operations before execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepare_support: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PrepareRenameResponse {
    Range(Range),
    RangeWithPlaceholder { range: Range, placeholder: String },
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentLinkOptions {
    /// Document links have a resolve provider as well.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolve_provider: Option<bool>,

    #[serde(flatten)]
    pub work_done_progress_options: WorkDoneProgressOptions,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentColorParams {
    /// The text document
    pub text_document: TextDocumentIdentifier,

    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,

    #[serde(flatten)]
    pub partial_result_params: PartialResultParams,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorInformation {
    /// The range in the document where this color appears.
    pub range: Range,
    /// The actual color value for this color range.
    pub color: Color,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Color {
    /// The red component of this color in the range [0-1].
    pub red: f64,
    /// The green component of this color in the range [0-1].
    pub green: f64,
    /// The blue component of this color in the range [0-1].
    pub blue: f64,
    /// The alpha component of this color in the range [0-1].
    pub alpha: f64,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorPresentationParams {
    /// The text document.
    pub text_document: TextDocumentIdentifier,

    /// The color information to request presentations for.
    pub color: Color,

    /// The range where the color would be inserted. Serves as a context.
    pub range: Range,

    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,

    #[serde(flatten)]
    pub partial_result_params: PartialResultParams,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ColorPresentation {
    /// The label of this color presentation. It will be shown on the color
    /// picker header. By default this is also the text that is inserted when selecting
    /// this color presentation.
    pub label: String,

    /// An [edit](#TextEdit) which is applied to a document when selecting
    /// this presentation for the color.  When `falsy` the [label](#ColorPresentation.label)
    /// is used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_edit: Option<TextEdit>,

    /// An optional array of additional [text edits](#TextEdit) that are applied when
    /// selecting this color presentation. Edits must not overlap with the main [edit](#ColorPresentation.textEdit) nor with themselves.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_text_edits: Option<Vec<TextEdit>>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FoldingRangeParams {
    /// The text document.
    pub text_document: TextDocumentIdentifier,

    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,

    #[serde(flatten)]
    pub partial_result_params: PartialResultParams,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum FoldingRangeProviderCapability {
    Simple(bool),
    FoldingProvider(FoldingProviderOptions),
    Options(StaticTextDocumentColorProviderOptions),
}

impl From<StaticTextDocumentColorProviderOptions> for FoldingRangeProviderCapability {
    fn from(from: StaticTextDocumentColorProviderOptions) -> Self {
        Self::Options(from)
    }
}

impl From<FoldingProviderOptions> for FoldingRangeProviderCapability {
    fn from(from: FoldingProviderOptions) -> Self {
        Self::FoldingProvider(from)
    }
}

impl From<bool> for FoldingRangeProviderCapability {
    fn from(from: bool) -> Self {
        Self::Simple(from)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct FoldingProviderOptions {}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FoldingRangeCapability {
    /// Whether implementation supports dynamic registration for folding range providers. If this is set to `true`
    /// the client supports the new `(FoldingRangeProviderOptions & TextDocumentRegistrationOptions & StaticRegistrationOptions)`
    /// return value for the corresponding server capability as well.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /// The maximum number of folding ranges that the client prefers to receive per document. The value serves as a
    /// hint, servers are free to follow the limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_limit: Option<u64>,
    /// If set, the client signals that it only supports folding complete lines. If set, client will
    /// ignore specified `startCharacter` and `endCharacter` properties in a FoldingRange.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_folding_only: Option<bool>,
}

/// Represents a folding range.
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FoldingRange {
    /// The zero-based line number from where the folded range starts.
    pub start_line: u64,

    /// The zero-based character offset from where the folded range starts. If not defined, defaults to the length of the start line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_character: Option<u64>,

    /// The zero-based line number where the folded range ends.
    pub end_line: u64,

    /// The zero-based character offset before the folded range ends. If not defined, defaults to the length of the end line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_character: Option<u64>,

    /// Describes the kind of the folding range such as `comment' or 'region'. The kind
    /// is used to categorize folding ranges and used by commands like 'Fold all comments'. See
    /// [FoldingRangeKind](#FoldingRangeKind) for an enumeration of standardized kinds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<FoldingRangeKind>,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
pub struct SelectionRangeOptions {
    #[serde(flatten)]
    pub work_done_progress_options: WorkDoneProgressOptions,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct SelectionRangeRegistrationOptions {
    #[serde(flatten)]
    pub selection_range_options: SelectionRangeOptions,

    #[serde(flatten)]
    pub registration_options: StaticTextDocumentRegistrationOptions,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SelectionRangeProviderCapability {
    Simple(bool),
    Options(SelectionRangeOptions),
    RegistrationOptions(SelectionRangeRegistrationOptions),
}

impl From<SelectionRangeRegistrationOptions> for SelectionRangeProviderCapability {
    fn from(from: SelectionRangeRegistrationOptions) -> Self {
        Self::RegistrationOptions(from)
    }
}

impl From<SelectionRangeOptions> for SelectionRangeProviderCapability {
    fn from(from: SelectionRangeOptions) -> Self {
        Self::Options(from)
    }
}

impl From<bool> for SelectionRangeProviderCapability {
    fn from(from: bool) -> Self {
        Self::Simple(from)
    }
}

/// A parameter literal used in selection range requests.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectionRangeParams {
    /// The text document.
    pub text_document: TextDocumentIdentifier,

    /// The positions inside the text document.
    pub positions: Vec<Position>,

    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,

    #[serde(flatten)]
    pub partial_result_params: PartialResultParams,
}

/// Represents a selection range.
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectionRange {
    /// Range of the selection.
    pub range: Range,

    /// The parent selection range containing this range.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<Box<SelectionRange>>,
}

/// Enum of known range kinds
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum FoldingRangeKind {
    /// Folding range for a comment
    Comment,
    /// Folding range for a imports or includes
    Imports,
    /// Folding range for a region (e.g. `#region`)
    Region,
}

/// Describes the content type that a client supports in various
/// result literals like `Hover`, `ParameterInfo` or `CompletionItem`.
///
/// Please note that `MarkupKinds` must not start with a `$`. This kinds
/// are reserved for internal usage.
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MarkupKind {
    /// Plain text is supported as a content format
    PlainText,
    /// Markdown is supported as a content format
    Markdown,
}

/// A `MarkupContent` literal represents a string value which content is interpreted base on its
/// kind flag. Currently the protocol supports `plaintext` and `markdown` as markup kinds.
///
/// If the kind is `markdown` then the value can contain fenced code blocks like in GitHub issues.
/// See <https://help.github.com/articles/creating-and-highlighting-code-blocks/#syntax-highlighting>
///
/// Here is an example how such a string can be constructed using JavaScript / TypeScript:
/// ```ignore
/// let markdown: MarkupContent = {
///     kind: MarkupKind::Markdown,
///     value: [
///         "# Header",
///         "Some text",
///         "```typescript",
///         "someCode();",
///         "```"
///     ]
///     .join("\n"),
/// };
/// ```
///
/// Please Note* that clients might sanitize the return markdown. A client could decide to
/// remove HTML from the markdown to avoid script execution.
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize, Clone)]
pub struct MarkupContent {
    pub kind: MarkupKind,
    pub value: String,
}

pub type ProgressToken = NumberOrString;

/// The progress notification is sent from the server to the client to ask
/// the client to indicate progress.
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProgressParams {
    /// The progress token provided by the client.
    pub token: ProgressToken,

    /// The progress data.
    pub value: ProgressParamsValue,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum ProgressParamsValue {
    WorkDone(WorkDoneProgress),
}

/// The `window/workDoneProgress/create` request is sent from the server
/// to the clientto ask the client to create a work done progress.
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkDoneProgressCreateParams {
    /// The token to be used to report progress.
    pub token: ProgressToken,
}

/// The `window/workDoneProgress/cancel` notification is sent from the client
/// to the server to cancel a progress initiated on the server side using the `window/workDoneProgress/create`.
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkDoneProgressCancelParams {
    /// The token to be used to report progress.
    pub token: ProgressToken,
}

/// Options to signal work done progress support in server capabilities.
#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkDoneProgressOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_done_progress: Option<bool>,
}

/// An optional token that a server can use to report work done progress
#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkDoneProgressParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_done_token: Option<ProgressToken>,
}

#[derive(Debug, PartialEq, Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkDoneProgressBegin {
    /// Mandatory title of the progress operation. Used to briefly inform
    /// about the kind of operation being performed.
    /// Examples: "Indexing" or "Linking dependencies".
    pub title: String,

    /// Controls if a cancel button should show to allow the user to cancel the
    /// long running operation. Clients that don't support cancellation are allowed
    /// to ignore the setting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancellable: Option<bool>,

    /// Optional, more detailed associated progress message. Contains
    /// complementary information to the `title`.
    /// Examples: "3/25 files", "project/src/module2", "node_modules/some_dep".
    /// If unset, the previous progress message (if any) is still valid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Optional progress percentage to display (value 100 is considered 100%).
    /// If unset, the previous progress percentage (if any) is still valid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage: Option<f64>,
}

#[derive(Debug, PartialEq, Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkDoneProgressReport {
    /// Controls if a cancel button should show to allow the user to cancel the
    /// long running operation. Clients that don't support cancellation are allowed
    /// to ignore the setting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancellable: Option<bool>,

    /// Optional, more detailed associated progress message. Contains
    /// complementary information to the `title`.
    /// Examples: "3/25 files", "project/src/module2", "node_modules/some_dep".
    /// If unset, the previous progress message (if any) is still valid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Optional progress percentage to display (value 100 is considered 100%).
    /// If unset, the previous progress percentage (if any) is still valid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percentage: Option<f64>,
}

#[derive(Debug, PartialEq, Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkDoneProgressEnd {
    /// Optional, more detailed associated progress message. Contains
    /// complementary information to the `title`.
    /// Examples: "3/25 files", "project/src/module2", "node_modules/some_dep".
    /// If unset, the previous progress message (if any) is still valid.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum WorkDoneProgress {
    Begin(WorkDoneProgressBegin),
    Report(WorkDoneProgressReport),
    End(WorkDoneProgressEnd),
}

/// A parameter literal used to pass a partial result token.
#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PartialResultParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_result_token: Option<ProgressToken>,
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct SemanticHighlightingClientCapability {
    /// `true` if the client supports semantic highlighting support text documents. Otherwise, `false`. It is `false` by default.
    pub semantic_highlighting: bool,
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize, Clone)]
#[cfg(feature = "proposed")]
pub struct SemanticHighlightingServerCapability {
    /// A "lookup table" of semantic highlighting [TextMate scopes](https://manual.macromates.com/en/language_grammars)
    /// supported by the language server. If not defined or empty, then the server does not support the semantic highlighting
    /// feature. Otherwise, clients should reuse this "lookup table" when receiving semantic highlighting notifications from
    /// the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
#[cfg(feature = "proposed")]
pub struct SemanticHighlightingToken {
    pub character: u32,
    pub length: u16,
    pub scope: u16,
}

#[cfg(feature = "proposed")]
impl SemanticHighlightingToken {
    /// Deserializes the tokens from a base64 encoded string
    fn deserialize_tokens<'de, D>(
        deserializer: D,
    ) -> Result<Option<Vec<SemanticHighlightingToken>>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let opt_s = Option::<String>::deserialize(deserializer)?;

        if let Some(s) = opt_s {
            let bytes = base64::decode_config(s.as_str(), base64::STANDARD)
                .map_err(|_| serde::de::Error::custom("Error parsing base64 string"))?;
            let mut res = Vec::new();
            for chunk in bytes.chunks_exact(8) {
                res.push(SemanticHighlightingToken {
                    character: u32::from_be_bytes(<[u8; 4]>::try_from(&chunk[0..4]).unwrap()),
                    length: u16::from_be_bytes(<[u8; 2]>::try_from(&chunk[4..6]).unwrap()),
                    scope: u16::from_be_bytes(<[u8; 2]>::try_from(&chunk[6..8]).unwrap()),
                });
            }
            Result::Ok(Some(res))
        } else {
            Result::Ok(None)
        }
    }

    /// Serialize the tokens to a base64 encoded string
    fn serialize_tokens<S>(
        tokens: &Option<Vec<SemanticHighlightingToken>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Some(tokens) = tokens {
            let mut bytes = vec![];
            for token in tokens {
                bytes.extend_from_slice(&token.character.to_be_bytes());
                bytes.extend_from_slice(&token.length.to_be_bytes());
                bytes.extend_from_slice(&token.scope.to_be_bytes());
            }
            serializer.collect_str(&base64::display::Base64Display::with_config(
                &bytes,
                base64::STANDARD,
            ))
        } else {
            serializer.serialize_none()
        }
    }
}

/// Represents a semantic highlighting information that has to be applied on a specific line of the text document.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[cfg(feature = "proposed")]
pub struct SemanticHighlightingInformation {
    /// The zero-based line position in the text document.
    pub line: i32,

    /// A base64 encoded string representing every single highlighted characters with its start position, length and the "lookup table" index of
    /// of the semantic highlighting [TextMate scopes](https://manual.macromates.com/en/language_grammars).
    /// If the `tokens` is empty or not defined, then no highlighted positions are available for the line.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "SemanticHighlightingToken::deserialize_tokens",
        serialize_with = "SemanticHighlightingToken::serialize_tokens"
    )]
    pub tokens: Option<Vec<SemanticHighlightingToken>>,
}

/// Parameters for the semantic highlighting (server-side) push notification.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct SemanticHighlightingParams {
    /// The text document that has to be decorated with the semantic highlighting information.
    pub text_document: VersionedTextDocumentIdentifier,

    /// An array of semantic highlighting information.
    pub lines: Vec<SemanticHighlightingInformation>,
}

/// A set of predefined token types. This set is not fixed
/// and clients can specify additional token types via the
/// corresponding client capabilities.
///
/// @since 3.16.0 - Proposed state
#[derive(Debug, Eq, PartialEq, Hash, PartialOrd, Clone, Deserialize, Serialize)]
#[cfg(feature = "proposed")]
pub struct SemanticTokenType(Cow<'static, str>);

#[cfg(feature = "proposed")]
impl SemanticTokenType {
    pub const COMMENT: SemanticTokenType = SemanticTokenType::new("comment");
    pub const KEYWORD: SemanticTokenType = SemanticTokenType::new("keyword");
    pub const STRING: SemanticTokenType = SemanticTokenType::new("string");
    pub const NUMBER: SemanticTokenType = SemanticTokenType::new("number");
    pub const REGEXP: SemanticTokenType = SemanticTokenType::new("regexp");
    pub const OPERATOR: SemanticTokenType = SemanticTokenType::new("operator");
    pub const NAMESPACE: SemanticTokenType = SemanticTokenType::new("namespace");
    pub const TYPE: SemanticTokenType = SemanticTokenType::new("type");
    pub const STRUCT: SemanticTokenType = SemanticTokenType::new("struct");
    pub const CLASS: SemanticTokenType = SemanticTokenType::new("class");
    pub const INTERFACE: SemanticTokenType = SemanticTokenType::new("interface");
    pub const ENUM: SemanticTokenType = SemanticTokenType::new("enum");
    pub const TYPE_PARAMETER: SemanticTokenType = SemanticTokenType::new("typeParameter");
    pub const FUNCTION: SemanticTokenType = SemanticTokenType::new("function");
    pub const MEMBER: SemanticTokenType = SemanticTokenType::new("member");
    pub const PROPERTY: SemanticTokenType = SemanticTokenType::new("property");
    pub const MACRO: SemanticTokenType = SemanticTokenType::new("macro");
    pub const VARIABLE: SemanticTokenType = SemanticTokenType::new("variable");
    pub const PARAMETER: SemanticTokenType = SemanticTokenType::new("parameter");
    pub const LABEL: SemanticTokenType = SemanticTokenType::new("label");

    pub const fn new(tag: &'static str) -> Self {
        SemanticTokenType(Cow::Borrowed(tag))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(feature = "proposed")]
impl From<String> for SemanticTokenType {
    fn from(from: String) -> Self {
        SemanticTokenType(Cow::from(from))
    }
}
#[cfg(feature = "proposed")]
impl From<&'static str> for SemanticTokenType {
    fn from(from: &'static str) -> Self {
        SemanticTokenType::new(from)
    }
}

/// A set of predefined token modifiers. This set is not fixed
/// and clients can specify additional token types via the
/// corresponding client capabilities.
///
/// @since 3.16.0 - Proposed state
#[derive(Debug, Eq, PartialEq, Hash, PartialOrd, Clone, Deserialize, Serialize)]
#[cfg(feature = "proposed")]
pub struct SemanticTokenModifier(Cow<'static, str>);

#[cfg(feature = "proposed")]
impl SemanticTokenModifier {
    pub const DOCUMENTATION: SemanticTokenModifier = SemanticTokenModifier::new("documentation");
    pub const DECLARATION: SemanticTokenModifier = SemanticTokenModifier::new("declaration");
    pub const DEFINITION: SemanticTokenModifier = SemanticTokenModifier::new("definition");
    pub const STATIC: SemanticTokenModifier = SemanticTokenModifier::new("static");
    pub const ABSTRACT: SemanticTokenModifier = SemanticTokenModifier::new("abstract");
    pub const DEPRECATED: SemanticTokenModifier = SemanticTokenModifier::new("deprecated");
    pub const READONLY: SemanticTokenModifier = SemanticTokenModifier::new("readonly");

    pub const fn new(tag: &'static str) -> Self {
        SemanticTokenModifier(Cow::Borrowed(tag))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(feature = "proposed")]
impl From<String> for SemanticTokenModifier {
    fn from(from: String) -> Self {
        SemanticTokenModifier(Cow::from(from))
    }
}
#[cfg(feature = "proposed")]
impl From<&'static str> for SemanticTokenModifier {
    fn from(from: &'static str) -> Self {
        SemanticTokenModifier::new(from)
    }
}

/// @since 3.16.0 - Proposed state
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct SemanticTokensLegend {
    /// The token types a server uses.
    pub token_types: Vec<SemanticTokenType>,

    /// The token modifiers a server uses.
    pub token_modifiers: Vec<SemanticTokenModifier>,
}

/// The actual tokens. For a detailed description about how the data is
/// structured please see
/// https://github.com/microsoft/vscode-extension-samples/blob/5ae1f7787122812dcc84e37427ca90af5ee09f14/semantic-tokens-sample/vscode.proposed.d.ts#L71
#[derive(Debug, Eq, PartialEq, Copy, Clone, Default)]
#[cfg(feature = "proposed")]
pub struct SemanticToken {
    pub delta_line: u32,
    pub delta_start: u32,
    pub length: u32,
    pub token_type: u32,
    pub token_modifiers_bitset: u32,
}

#[cfg(feature = "proposed")]
impl SemanticToken {
    fn deserialize_tokens<'de, D>(deserializer: D) -> Result<Vec<SemanticToken>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data = Vec::<u32>::deserialize(deserializer)?;
        let chunks = data.chunks_exact(5);

        if !chunks.remainder().is_empty() {
            return Result::Err(serde::de::Error::custom("Length is not divisible by 5"));
        }

        Result::Ok(
            chunks
                .map(|chunk| SemanticToken {
                    delta_line: chunk[0],
                    delta_start: chunk[1],
                    length: chunk[2],
                    token_type: chunk[3],
                    token_modifiers_bitset: chunk[4],
                })
                .collect(),
        )
    }

    fn serialize_tokens<S>(tokens: &[SemanticToken], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(tokens.len() * 5))?;
        for token in tokens.iter() {
            seq.serialize_element(&token.delta_line)?;
            seq.serialize_element(&token.delta_start)?;
            seq.serialize_element(&token.length)?;
            seq.serialize_element(&token.token_type)?;
            seq.serialize_element(&token.token_modifiers_bitset)?;
        }
        seq.end()
    }

    fn deserialize_tokens_opt<'de, D>(
        deserializer: D,
    ) -> Result<Option<Vec<SemanticToken>>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(transparent)]
        struct Wrapper {
            #[serde(deserialize_with = "SemanticToken::deserialize_tokens")]
            tokens: Vec<SemanticToken>,
        }

        Ok(Option::<Wrapper>::deserialize(deserializer)?.map(|wrapper| wrapper.tokens))
    }

    fn serialize_tokens_opt<S>(
        data: &Option<Vec<SemanticToken>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        #[serde(transparent)]
        struct Wrapper {
            #[serde(serialize_with = "SemanticToken::serialize_tokens")]
            tokens: Vec<SemanticToken>,
        }

        let opt = data.as_ref().map(|t| Wrapper { tokens: t.to_vec() });

        opt.serialize(serializer)
    }
}

/// @since 3.16.0 - Proposed state
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct SemanticTokens {
    /// An optional result id. If provided and clients support delta updating
    /// the client will include the result id in the next semantic token request.
    /// A server can then instead of computing all sematic tokens again simply
    /// send a delta.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_id: Option<String>,

    /// The actual tokens. For a detailed description about how the data is
    /// structured please see
    /// https://github.com/microsoft/vscode-extension-samples/blob/5ae1f7787122812dcc84e37427ca90af5ee09f14/semantic-tokens-sample/vscode.proposed.d.ts#L71
    #[serde(
        deserialize_with = "SemanticToken::deserialize_tokens",
        serialize_with = "SemanticToken::serialize_tokens"
    )]
    pub data: Vec<SemanticToken>,
}

/// @since 3.16.0 - Proposed state
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct SemanticTokensPartialResult {
    #[serde(
        deserialize_with = "SemanticToken::deserialize_tokens",
        serialize_with = "SemanticToken::serialize_tokens"
    )]
    pub data: Vec<SemanticToken>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
#[cfg(feature = "proposed")]
pub enum SemanticTokensResult {
    Tokens(SemanticTokens),
    Partial(SemanticTokensPartialResult),
}

#[cfg(feature = "proposed")]
impl From<SemanticTokens> for SemanticTokensResult {
    fn from(from: SemanticTokens) -> Self {
        SemanticTokensResult::Tokens(from)
    }
}

#[cfg(feature = "proposed")]
impl From<SemanticTokensPartialResult> for SemanticTokensResult {
    fn from(from: SemanticTokensPartialResult) -> Self {
        SemanticTokensResult::Partial(from)
    }
}

/// @since 3.16.0 - Proposed state
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct SemanticTokensEdit {
    pub start: u32,
    pub delete_count: u32,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "SemanticToken::deserialize_tokens_opt",
        serialize_with = "SemanticToken::serialize_tokens_opt"
    )]
    pub data: Option<Vec<SemanticToken>>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
#[cfg(feature = "proposed")]
pub enum SemanticTokensEditResult {
    Tokens(SemanticTokens),
    TokensEdits(SemanticTokensEdits),
    PartialTokens(SemanticTokensPartialResult),
    PartialTokensEdit(SemanticTokensEditsPartialResult),
}

#[cfg(feature = "proposed")]
impl From<SemanticTokens> for SemanticTokensEditResult {
    fn from(from: SemanticTokens) -> Self {
        SemanticTokensEditResult::Tokens(from)
    }
}

#[cfg(feature = "proposed")]
impl From<SemanticTokensEdits> for SemanticTokensEditResult {
    fn from(from: SemanticTokensEdits) -> Self {
        SemanticTokensEditResult::TokensEdits(from)
    }
}

#[cfg(feature = "proposed")]
impl From<SemanticTokensPartialResult> for SemanticTokensEditResult {
    fn from(from: SemanticTokensPartialResult) -> Self {
        SemanticTokensEditResult::PartialTokens(from)
    }
}

#[cfg(feature = "proposed")]
impl From<SemanticTokensEditsPartialResult> for SemanticTokensEditResult {
    fn from(from: SemanticTokensEditsPartialResult) -> Self {
        SemanticTokensEditResult::PartialTokensEdit(from)
    }
}

/// @since 3.16.0 - Proposed state
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct SemanticTokensEdits {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_id: Option<String>,
    /// For a detailed description how these edits are structured pls see
    /// https://github.com/microsoft/vscode-extension-samples/blob/5ae1f7787122812dcc84e37427ca90af5ee09f14/semantic-tokens-sample/vscode.proposed.d.ts#L131
    pub edits: Vec<SemanticTokensEdit>,
}

/// @since 3.16.0 - Proposed state
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct SemanticTokensEditsPartialResult {
    pub edits: Vec<SemanticTokensEdit>,
}

/// Capabilities specific to the `textDocument/semanticTokens`
///
/// @since 3.16.0 - Proposed state
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct SemanticTokensClientCapabilities {
    /// Whether implementation supports dynamic registration. If this is set to `true`
    /// the client supports the new `(TextDocumentRegistrationOptions & StaticRegistrationOptions)`
    /// return value for the corresponding server capability as well.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /// The token types known by the client.
    pub token_types: Vec<SemanticTokenType>,

    /// The token modifiers known by the client.
    pub token_modifiers: Vec<SemanticTokenModifier>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
#[cfg(feature = "proposed")]
pub enum SemanticTokensDocumentProvider {
    Bool(bool),

    /// The server supports deltas for full documents.
    Edits {
        #[serde(skip_serializing_if = "Option::is_none")]
        edits: Option<bool>,
    },
}

/// @since 3.16.0 - Proposed state
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct SemanticTokensOptions {
    #[serde(flatten)]
    pub work_done_progress_options: WorkDoneProgressOptions,

    /// The legend used by the server
    pub legend: SemanticTokensLegend,

    /// Server supports providing semantic tokens for a sepcific range
    /// of a document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_provider: Option<bool>,

    /// Server supports providing semantic tokens for a full document.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_provider: Option<SemanticTokensDocumentProvider>,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct SemanticTokensRegistrationOptions {
    #[serde(flatten)]
    pub text_document_registration_options: TextDocumentRegistrationOptions,

    #[serde(flatten)]
    pub semantic_tokens_options: SemanticTokensOptions,

    #[serde(flatten)]
    pub static_registration_options: StaticRegistrationOptions,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
#[cfg(feature = "proposed")]
pub enum SemanticTokensServerCapabilities {
    SemanticTokensOptions(SemanticTokensOptions),
    SemanticTokensRegistrationOptions(SemanticTokensRegistrationOptions),
}

#[cfg(feature = "proposed")]
impl From<SemanticTokensOptions> for SemanticTokensServerCapabilities {
    fn from(from: SemanticTokensOptions) -> Self {
        SemanticTokensServerCapabilities::SemanticTokensOptions(from)
    }
}

#[cfg(feature = "proposed")]
impl From<SemanticTokensRegistrationOptions> for SemanticTokensServerCapabilities {
    fn from(from: SemanticTokensRegistrationOptions) -> Self {
        SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(from)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct SemanticTokensParams {
    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,

    #[serde(flatten)]
    pub partial_result_params: PartialResultParams,

    /// The text document.
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct SemanticTokensEditsParams {
    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,

    #[serde(flatten)]
    pub partial_result_params: PartialResultParams,

    /// The text document.
    pub text_document: TextDocumentIdentifier,

    /// The previous result id.
    pub previous_result_id: String,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct SemanticTokensRangeParams {
    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,

    #[serde(flatten)]
    pub partial_result_params: PartialResultParams,

    /// The text document.
    pub text_document: TextDocumentIdentifier,

    /// The range the semantic tokens are requested for.
    pub range: Range,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
#[cfg(feature = "proposed")]
pub enum SemanticTokensRangeResult {
    Tokens(SemanticTokens),
    Partial(SemanticTokensPartialResult),
}

#[cfg(feature = "proposed")]
impl From<SemanticTokens> for SemanticTokensRangeResult {
    fn from(tokens: SemanticTokens) -> Self {
        SemanticTokensRangeResult::Tokens(tokens)
    }
}

#[cfg(feature = "proposed")]
impl From<SemanticTokensPartialResult> for SemanticTokensRangeResult {
    fn from(partial: SemanticTokensPartialResult) -> Self {
        SemanticTokensRangeResult::Partial(partial)
    }
}

/// Symbol tags are extra annotations that tweak the rendering of a symbol.
/// Since 3.15
#[derive(Debug, Eq, PartialEq, Clone, Deserialize_repr, Serialize_repr)]
#[repr(u8)]
#[cfg(feature = "proposed")]
pub enum SymbolTag {
    /// Render a symbol as obsolete, usually using a strike-out.
    Deprecated = 1,
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct CallHierarchyOptions {
    #[serde(flatten)]
    pub work_done_progress_options: WorkDoneProgressOptions,
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
#[cfg(feature = "proposed")]
pub enum CallHierarchyServerCapability {
    Simple(bool),
    Options(CallHierarchyOptions),
}

#[cfg(feature = "proposed")]
impl From<CallHierarchyOptions> for CallHierarchyServerCapability {
    fn from(from: CallHierarchyOptions) -> Self {
        Self::Options(from)
    }
}

#[cfg(feature = "proposed")]
impl From<bool> for CallHierarchyServerCapability {
    fn from(from: bool) -> Self {
        Self::Simple(from)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct CallHierarchyPrepareParams {
    #[serde(flatten)]
    pub text_document_position_params: TextDocumentPositionParams,

    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct CallHierarchyItem {
    /// The name of this item.
    pub name: String,

    /// The kind of this item.
    pub kind: SymbolKind,

    /// Tags for this item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<SymbolTag>>,

    /// More detail for this item, e.g. the signature of a function.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,

    /// The resource identifier of this item.
    pub uri: Url,

    /// The range enclosing this symbol not including leading/trailing whitespace but everything else, e.g. comments and code.
    pub range: Range,

    /// The range that should be selected and revealed when this symbol is being picked, e.g. the name of a function.
    /// Must be contained by the [`range`](#CallHierarchyItem.range).
    pub selection_range: Range,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct CallHierarchyIncomingCallsParams {
    pub item: CallHierarchyItem,

    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,

    #[serde(flatten)]
    pub partial_result_params: PartialResultParams,
}

/// Represents an incoming call, e.g. a caller of a method or constructor.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct CallHierarchyIncomingCall {
    /// The item that makes the call.
    pub from: CallHierarchyItem,

    /// The range at which at which the calls appears. This is relative to the caller
    /// denoted by [`this.from`](#CallHierarchyIncomingCall.from).
    pub from_ranges: Vec<Range>,
}

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct CallHierarchyOutgoingCallsParams {
    pub item: CallHierarchyItem,

    #[serde(flatten)]
    pub work_done_progress_params: WorkDoneProgressParams,

    #[serde(flatten)]
    pub partial_result_params: PartialResultParams,
}

/// Represents an outgoing call, e.g. calling a getter from a method or a method from a constructor etc.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[cfg(feature = "proposed")]
pub struct CallHierarchyOutgoingCall {
    /// The item that is called.
    pub to: CallHierarchyItem,

    /// The range at which this item is called. This is the range relative to the caller, e.g the item
    /// passed to [`provideCallHierarchyOutgoingCalls`](#CallHierarchyItemProvider.provideCallHierarchyOutgoingCalls)
    /// and not [`this.to`](#CallHierarchyOutgoingCall.to).
    pub from_ranges: Vec<Range>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    fn test_serialization<SER>(ms: &SER, expected: &str)
    where
        SER: Serialize + for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
    {
        let json_str = serde_json::to_string(ms).unwrap();
        assert_eq!(&json_str, expected);
        let deserialized: SER = serde_json::from_str(&json_str).unwrap();
        assert_eq!(&deserialized, ms);
    }

    fn test_deserialization<T>(json: &str, expected: &T)
    where
        T: for<'de> Deserialize<'de> + PartialEq + std::fmt::Debug,
    {
        let value = serde_json::from_str::<T>(json).unwrap();
        assert_eq!(&value, expected);
    }

    #[test]
    fn number_or_string() {
        test_serialization(&NumberOrString::Number(123), r#"123"#);

        test_serialization(&NumberOrString::String("abcd".into()), r#""abcd""#);
    }

    #[test]
    fn marked_string() {
        test_serialization(&MarkedString::from_markdown("xxx".into()), r#""xxx""#);

        test_serialization(
            &MarkedString::from_language_code("lang".into(), "code".into()),
            r#"{"language":"lang","value":"code"}"#,
        );
    }

    #[test]
    fn language_string() {
        test_serialization(
            &LanguageString {
                language: "LL".into(),
                value: "VV".into(),
            },
            r#"{"language":"LL","value":"VV"}"#,
        );
    }

    #[test]
    fn workspace_edit() {
        test_serialization(
            &WorkspaceEdit {
                changes: Some(vec![].into_iter().collect()),
                document_changes: None,
            },
            r#"{"changes":{}}"#,
        );

        test_serialization(
            &WorkspaceEdit {
                changes: None,
                document_changes: None,
            },
            r#"{}"#,
        );

        test_serialization(
            &WorkspaceEdit {
                changes: Some(
                    vec![(Url::parse("file://test").unwrap(), vec![])]
                        .into_iter()
                        .collect(),
                ),
                document_changes: None,
            },
            r#"{"changes":{"file://test/":[]}}"#,
        );
    }

    #[test]
    fn formatting_options() {
        test_serialization(
            &FormattingOptions {
                tab_size: 123,
                insert_spaces: true,
                properties: HashMap::new(),
                trim_trailing_whitespace: None,
                insert_final_newline: None,
                trim_final_newlines: None,
            },
            r#"{"tabSize":123,"insertSpaces":true}"#,
        );

        test_serialization(
            &FormattingOptions {
                tab_size: 123,
                insert_spaces: true,
                properties: vec![("prop".to_string(), FormattingProperty::Number(1.0))]
                    .into_iter()
                    .collect(),
                trim_trailing_whitespace: None,
                insert_final_newline: None,
                trim_final_newlines: None,
            },
            r#"{"tabSize":123,"insertSpaces":true,"prop":1.0}"#,
        );
    }

    #[test]
    fn root_uri_can_be_missing() {
        serde_json::from_str::<InitializeParams>(r#"{ "capabilities": {} }"#).unwrap();
    }

    #[test]
    fn test_watch_kind() {
        test_serialization(&WatchKind::Create, "1");
        test_serialization(&(WatchKind::Create | WatchKind::Change), "3");
        test_serialization(
            &(WatchKind::Create | WatchKind::Change | WatchKind::Delete),
            "7",
        );
    }

    #[test]
    fn test_resource_operation_kind() {
        test_serialization(
            &vec![
                ResourceOperationKind::Create,
                ResourceOperationKind::Rename,
                ResourceOperationKind::Delete,
            ],
            r#"["create","rename","delete"]"#,
        );
    }

    #[test]
    fn test_code_action_response() {
        test_serialization(
            &vec![
                CodeActionOrCommand::Command(Command {
                    title: "title".to_string(),
                    command: "command".to_string(),
                    arguments: None,
                }),
                CodeActionOrCommand::CodeAction(CodeAction {
                    title: "title".to_string(),
                    kind: Some(CodeActionKind::QUICKFIX),
                    command: None,
                    diagnostics: None,
                    edit: None,
                    is_preferred: None,
                }),
            ],
            r#"[{"title":"title","command":"command"},{"title":"title","kind":"quickfix"}]"#,
        )
    }

    #[cfg(feature = "proposed")]
    #[test]
    fn test_semantic_highlighting_information_serialization() {
        test_serialization(
            &SemanticHighlightingInformation {
                line: 10,
                tokens: Some(vec![
                    SemanticHighlightingToken {
                        character: 0x00000001,
                        length: 0x0002,
                        scope: 0x0003,
                    },
                    SemanticHighlightingToken {
                        character: 0x00112222,
                        length: 0x0FF0,
                        scope: 0x0202,
                    },
                ]),
            },
            r#"{"line":10,"tokens":"AAAAAQACAAMAESIiD/ACAg=="}"#,
        );

        test_serialization(
            &SemanticHighlightingInformation {
                line: 22,
                tokens: None,
            },
            r#"{"line":22}"#,
        );
    }

    #[test]
    fn test_tag_support_deserialization() {
        let mut empty = CompletionItemCapability::default();
        empty.tag_support = None;

        test_deserialization(r#"{}"#, &empty);
        test_deserialization(r#"{"tagSupport": false}"#, &empty);

        let mut t = CompletionItemCapability::default();
        t.tag_support = Some(TagSupport { value_set: vec![] });
        test_deserialization(r#"{"tagSupport": true}"#, &t);

        let mut t = CompletionItemCapability::default();
        t.tag_support = Some(TagSupport {
            value_set: vec![CompletionItemTag::Deprecated],
        });
        test_deserialization(r#"{"tagSupport": {"valueSet": [1]}}"#, &t);
    }

    #[cfg(feature = "proposed")]
    #[test]
    fn test_semantic_tokens_support_serialization() {
        test_serialization(
            &SemanticTokens {
                result_id: None,
                data: vec![],
            },
            r#"{"data":[]}"#,
        );

        test_serialization(
            &SemanticTokens {
                result_id: None,
                data: vec![SemanticToken {
                    delta_line: 2,
                    delta_start: 5,
                    length: 3,
                    token_type: 0,
                    token_modifiers_bitset: 3,
                }],
            },
            r#"{"data":[2,5,3,0,3]}"#,
        );

        test_serialization(
            &SemanticTokens {
                result_id: None,
                data: vec![
                    SemanticToken {
                        delta_line: 2,
                        delta_start: 5,
                        length: 3,
                        token_type: 0,
                        token_modifiers_bitset: 3,
                    },
                    SemanticToken {
                        delta_line: 0,
                        delta_start: 5,
                        length: 4,
                        token_type: 1,
                        token_modifiers_bitset: 0,
                    },
                ],
            },
            r#"{"data":[2,5,3,0,3,0,5,4,1,0]}"#,
        );
    }

    #[cfg(feature = "proposed")]
    #[test]
    fn test_semantic_tokens_support_deserialization() {
        test_deserialization(
            r#"{"data":[]}"#,
            &SemanticTokens {
                result_id: None,
                data: vec![],
            },
        );

        test_deserialization(
            r#"{"data":[2,5,3,0,3]}"#,
            &SemanticTokens {
                result_id: None,
                data: vec![SemanticToken {
                    delta_line: 2,
                    delta_start: 5,
                    length: 3,
                    token_type: 0,
                    token_modifiers_bitset: 3,
                }],
            },
        );

        test_deserialization(
            r#"{"data":[2,5,3,0,3,0,5,4,1,0]}"#,
            &SemanticTokens {
                result_id: None,
                data: vec![
                    SemanticToken {
                        delta_line: 2,
                        delta_start: 5,
                        length: 3,
                        token_type: 0,
                        token_modifiers_bitset: 3,
                    },
                    SemanticToken {
                        delta_line: 0,
                        delta_start: 5,
                        length: 4,
                        token_type: 1,
                        token_modifiers_bitset: 0,
                    },
                ],
            },
        );
    }

    #[cfg(feature = "proposed")]
    #[test]
    #[should_panic]
    fn test_semantic_tokens_support_deserialization_err() {
        test_deserialization(
            r#"{"data":[1]}"#,
            &SemanticTokens {
                result_id: None,
                data: vec![],
            },
        );
    }

    #[cfg(feature = "proposed")]
    #[test]
    fn test_semantic_tokens_edit_support_deserialization() {
        test_deserialization(
            r#"{"start":0,"deleteCount":1,"data":[2,5,3,0,3,0,5,4,1,0]}"#,
            &SemanticTokensEdit {
                start: 0,
                delete_count: 1,
                data: Some(vec![
                    SemanticToken {
                        delta_line: 2,
                        delta_start: 5,
                        length: 3,
                        token_type: 0,
                        token_modifiers_bitset: 3,
                    },
                    SemanticToken {
                        delta_line: 0,
                        delta_start: 5,
                        length: 4,
                        token_type: 1,
                        token_modifiers_bitset: 0,
                    },
                ]),
            },
        );

        test_deserialization(
            r#"{"start":0,"deleteCount":1}"#,
            &SemanticTokensEdit {
                start: 0,
                delete_count: 1,
                data: None,
            },
        );
    }

    #[cfg(feature = "proposed")]
    #[test]
    fn test_semantic_tokens_edit_support_serialization() {
        test_serialization(
            &SemanticTokensEdit {
                start: 0,
                delete_count: 1,
                data: Some(vec![
                    SemanticToken {
                        delta_line: 2,
                        delta_start: 5,
                        length: 3,
                        token_type: 0,
                        token_modifiers_bitset: 3,
                    },
                    SemanticToken {
                        delta_line: 0,
                        delta_start: 5,
                        length: 4,
                        token_type: 1,
                        token_modifiers_bitset: 0,
                    },
                ]),
            },
            r#"{"start":0,"deleteCount":1,"data":[2,5,3,0,3,0,5,4,1,0]}"#,
        );

        test_serialization(
            &SemanticTokensEdit {
                start: 0,
                delete_count: 1,
                data: None,
            },
            r#"{"start":0,"deleteCount":1}"#,
        );
    }
}
