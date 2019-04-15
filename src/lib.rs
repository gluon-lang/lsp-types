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

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate num_derive;
extern crate num_traits;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate url;
extern crate url_serde;

pub use url::Url;

use std::collections::HashMap;

use num_traits::FromPrimitive;
use serde::de;
use serde::de::Error as Error_;
use serde_json::Value;

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

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
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
        Position {
            line: line,
            character: character,
        }
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
        Range {
            start: start,
            end: end,
        }
    }
}

/// Represents a location inside a resource, such as a line inside a text file.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct Location {
    #[serde(with = "url_serde")]
    pub uri: Url,
    pub range: Range,
}

impl Location {
    pub fn new(uri: Url, range: Range) -> Location {
        Location {
            uri: uri,
            range: range,
        }
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
    #[serde(with = "url_serde")]
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
}

impl Diagnostic {
    pub fn new(
        range: Range,
        severity: Option<DiagnosticSeverity>,
        code: Option<NumberOrString>,
        source: Option<String>,
        message: String,
        related_information: Option<Vec<DiagnosticRelatedInformation>>,
    ) -> Diagnostic {
        Diagnostic {
            range,
            severity,
            code,
            source,
            message,
            related_information,
        }
    }

    pub fn new_simple(range: Range, message: String) -> Diagnostic {
        Self::new(range, None, None, None, message, None)
    }

    pub fn new_with_code_number(
        range: Range,
        severity: DiagnosticSeverity,
        code_number: u64,
        source: Option<String>,
        message: String,
    ) -> Diagnostic {
        let code = Some(NumberOrString::Number(code_number));
        Self::new(range, Some(severity), code, source, message, None)
    }
}

/// The protocol currently supports the following diagnostic severities:
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
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

impl<'de> serde::Deserialize<'de> for DiagnosticSeverity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(match try!(u8::deserialize(deserializer)) {
            1 => DiagnosticSeverity::Error,
            2 => DiagnosticSeverity::Warning,
            3 => DiagnosticSeverity::Information,
            4 => DiagnosticSeverity::Hint,
            i => {
                return Err(D::Error::invalid_value(
                    de::Unexpected::Unsigned(i as u64),
                    &"value of 1, 2, 3 or 4",
                ));
            }
        })
    }
}

impl serde::Serialize for DiagnosticSeverity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

/**
 Represents a reference to a command. Provides a title which will be used to represent a command in the UI.
 Commands are identitifed using a string identifier and the protocol currently doesn't specify a set of
 well known commands. So executing a command requires some tool extension code.
*/
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
            title: title,
            command: command,
            arguments: arguments,
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
        TextEdit {
            range: range,
            new_text: new_text,
        }
    }
}

/**
  Describes textual changes on a single text document. The text document is referred to as a
  `VersionedTextDocumentIdentifier` to allow clients to check the text document version before an
  edit is applied. A `TextDocumentEdit` describes all changes on a version Si and after they are
  applied move the document to version Si+1. So the creator of a `TextDocumentEdit` doesn't need to
  sort the array or do any kind of ordering. However the edits must be non overlapping.
  */
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentEdit {
    /**
     * The text document to change.
     */
    pub text_document: VersionedTextDocumentIdentifier,

    /**
     * The edits to be applied.
     */
    pub edits: Vec<TextEdit>,
}

/**
 * Options to create a file.
 */
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFileOptions {
    /**
     * Overwrite existing file. Overwrite wins over `ignoreIfExists`
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overwrite: Option<bool>,
    /**
     * Ignore if exists.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_if_exists: Option<bool>,
}

/**
 * Create file operation
 */
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFile {
    /**
     * The resource to create.
     */
    #[serde(with = "url_serde")]
    pub uri: Url,
    /**
     * Additional options
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<CreateFileOptions>,
}

/**
 * Rename file options
 */
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameFileOptions {
    /**
     * Overwrite target if existing. Overwrite wins over `ignoreIfExists`
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overwrite: Option<bool>,
    /**
     * Ignores if target exists.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_if_exists: Option<bool>,
}

/**
 * Rename file operation
 */
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameFile {
    /**
     * The old (existing) location.
     */
    #[serde(with = "url_serde")]
    pub old_uri: Url,
    /**
     * The new location.
     */
    #[serde(with = "url_serde")]
    pub new_uri: Url,
    /**
     * Rename options.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<RenameFileOptions>,
}

/**
 * Delete file options
 */
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteFileOptions {
    /**
     * Delete the content recursively if a folder is denoted.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recursive: Option<bool>,
    /**
     * Ignore the operation if the file doesn't exist.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_if_not_exists: Option<bool>,
}

/**
 * Delete file operation
 */
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteFile {
    /**
     * The file to delete.
     */
    #[serde(with = "url_serde")]
    pub uri: Url,
    /**
     * Delete options.
     */
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

    /**
     * Depending on the client capability `workspace.workspaceEdit.resourceOperations` document changes
     * are either an array of `TextDocumentEdit`s to express changes to n different text documents
     * where each text document edit addresses a specific version of a text document. Or it can contain
     * above `TextDocumentEdit`s mixed with create, rename and delete file / folder operations.
     *
     * Whether a client supports versioned document edits is expressed via
     * `workspace.workspaceEdit.documentChanges` client capability.
     *
     * If a client neither supports `documentChanges` nor `workspace.workspaceEdit.resourceOperations` then
     * only plain `TextEdit`s using the `changes` property are supported.
     */
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
	pub items: Vec<ConfigurationItem>
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
                while let Some((key, value)) = visitor.next_entry::<url_serde::De<Url>, _>()? {
                    values.insert(key.into_inner(), value);
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
    #[serde(with = "url_serde")]
    pub uri: Url,
}

impl TextDocumentIdentifier {
    pub fn new(uri: Url) -> TextDocumentIdentifier {
        TextDocumentIdentifier { uri: uri }
    }
}

/// An item to transfer a text document from the client to the server.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentItem {
    /// The text document's URI.
    #[serde(with = "url_serde")]
    pub uri: Url,

    /// The text document's language identifier.
    pub language_id: String,

    /// The version number of this document (it will strictly increase after each
    /// change, including undo/redo).
    pub version: u64,

    /// The content of the opened text document.
    pub text: String,
}

impl TextDocumentItem {
    pub fn new(uri: Url, language_id: String, version: u64, text: String) -> TextDocumentItem {
        TextDocumentItem {
            uri: uri,
            language_id: language_id,
            version: version,
            text: text,
        }
    }
}

/// An identifier to denote a specific version of a text document.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct VersionedTextDocumentIdentifier {
    // This field was "mixed-in" from TextDocumentIdentifier
    /// The text document's URI.
    #[serde(with = "url_serde")]
    pub uri: Url,

    /// The version number of this document.
    pub version: Option<u64>,
}

impl VersionedTextDocumentIdentifier {
    pub fn new(uri: Url, version: u64) -> VersionedTextDocumentIdentifier {
        VersionedTextDocumentIdentifier {
            uri: uri,
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
            text_document: text_document,
            position: position,
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
    /**
     * A language id, like `typescript`.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /**
     * A Uri [scheme](#Uri.scheme), like `file` or `untitled`.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheme: Option<String>,

    /**
     * A glob pattern, like `*.{ts,js}`.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

/// A document selector is the combination of one or many document filters.
pub type DocumentSelector = Vec<DocumentFilter>;

// ========================= Actual Protocol =========================

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    /// The process Id of the parent process that started
    /// the server. Is null if the process has not been started by another process.
    /// If the parent process is not alive then the server should exit (see exit notification) its process.
    pub process_id: Option<u64>,

    /// The rootPath of the workspace. Is null
    /// if no folder is open.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_path: Option<String>,

    /// The rootUri of the workspace. Is null if no
    /// folder is open. If both `rootPath` and `rootUri` are set
    /// `rootUri` wins.
    #[serde(with = "option_url")]
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
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct InitializedParams {}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
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

mod option_url {
    use serde::{self, Serialize};
    use url::Url;
    use url_serde::{De, Ser};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Url>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        serde::Deserialize::deserialize(deserializer)
            .map(|x: Option<De<Url>>| x.map(|url| url.into_inner()))
    }

    pub fn serialize<S>(self_: &Option<Url>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self_.as_ref().map(Ser::new).serialize(serializer)
    }
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenericCapability {
    /**
     * This capability supports dynamic registration.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GotoCapability {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /// The client supports additional metadata in the form of definition links.
    pub link_support: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceEditCapability {
    /**
     * The client supports versioned document changes in `WorkspaceEdit`s
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_changes: Option<bool>,

    /**
     * The resource operations the client supports. Clients should at least
     * support 'create', 'rename' and 'delete' files and folders.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_operations: Option<Vec<ResourceOperationKind>>,

    /**
     * The failure handling strategy of a client if applying the workspace edit
     * failes.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_handling: Option<FailureHandlingKind>,
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceCapability {
    /// The server supports workspace folder.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_folders: Option<WorkspaceFolderCapability>,
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFolderCapability {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_notifications: Option<WorkspaceFolderCapabilityChangeNotifications>,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum WorkspaceFolderCapabilityChangeNotifications {
    Bool(bool),
    Id(String),
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFolder {
    /// The associated URI for this workspace folder.
    #[serde(with = "url_serde")]
    pub uri: Url,
    /// The name of the workspace folder. Defaults to the uri's basename.
    pub name: String,
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DidChangeWorkspaceFoldersParams {
    /**
     * The actual workspace folder change event.
     */
    pub event: WorkspaceFoldersChangeEvent,
}

/**
 * The workspace folder change event.
 */
#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFoldersChangeEvent {
    /**
     * The array of added workspace folders
     */
    pub added: Vec<WorkspaceFolder>,

    /**
     * The array of the removed workspace folders
     */
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

/**
 * Specific capabilities for the `SymbolKind` in the `workspace/symbol` request.
 */
#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolKindCapability {
    /**
     * The symbol kind values the client supports. When this
     * property exists the client also guarantees that it will
     * handle values outside its set gracefully and falls back
     * to a default value when unknown.
     *
     * If this property is not present the client only supports
     * the symbol kinds from `File` to `Array` as defined in
     * the initial version of the protocol.
     */
    pub value_set: Option<Vec<SymbolKind>>,
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolCapability {
    /**
     * This capability supports dynamic registration.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /**
     * Specific capabilities for the `SymbolKind` in the `workspace/symbol` request.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol_kind: Option<SymbolKindCapability>,
}

/**
 * Workspace specific client capabilities.
 */
#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceClientCapabilities {
    /**
     * The client supports applying batch edits to the workspace by supporting
     * the request 'workspace/applyEdit'
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply_edit: Option<bool>,

    /**
     * Capabilities specific to `WorkspaceEdit`s
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_edit: Option<WorkspaceEditCapability>,

    /**
     * Capabilities specific to the `workspace/didChangeConfiguration` notification.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_change_configuration: Option<GenericCapability>,

    /**
     * Capabilities specific to the `workspace/didChangeWatchedFiles` notification.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_change_watched_files: Option<GenericCapability>,

    /**
     * Capabilities specific to the `workspace/symbol` request.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<SymbolCapability>,

    /**
     * Capabilities specific to the `workspace/executeCommand` request.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execute_command: Option<GenericCapability>,

    /**
     * The client has support for workspace folders.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_folders: Option<bool>,

    /**
     * The client supports `workspace/configuration` requests.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SynchronizationCapability {
    /**
     * Whether text document synchronization supports dynamic registration.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /**
     * The client supports sending will save notifications.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_save: Option<bool>,

    /**
     * The client supports sending a will save request and
     * waits for a response providing text edits which will
     * be applied to the document before it is saved.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_save_wait_until: Option<bool>,

    /**
     * The client supports did save notifications.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_save: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionItemCapability {
    /**
     * Client supports snippets as insert text.
     *
     * A snippet can define tab stops and placeholders with `$1`, `$2`
     * and `${3:foo}`. `$0` defines the final tab stop, it defaults to
     * the end of the snippet. Placeholders with equal identifiers are linked,
     * that is typing in one will update others too.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet_support: Option<bool>,

    /**
     * Client supports commit characters on a completion item.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_characters_support: Option<bool>,

    /**
     * Client supports the follow content formats for the documentation
     * property. The order describes the preferred format of the client.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_format: Option<Vec<MarkupKind>>,

    /**
     * Client supports the deprecated property on a completion item.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated_support: Option<bool>,

    /**
     * Client supports the preselect property on a completion item.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preselect_support: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionItemKindCapability {
    /**
     * The completion item kind values the client supports. When this
     * property exists the client also guarantees that it will
     * handle values outside its set gracefully and falls back
     * to a default value when unknown.
     *
     * If this property is not present the client only supports
     * the completion items kinds from `Text` to `Reference` as defined in
     * the initial version of the protocol.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_set: Option<Vec<CompletionItemKind>>,
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HoverCapability {
    /**
     * Whether completion supports dynamic registration.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /**
     * Client supports the follow content formats for the content
     * property. The order describes the preferred format of the client.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_format: Option<Vec<MarkupKind>>,
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionCapability {
    /**
     * Whether completion supports dynamic registration.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /**
     * The client supports the following `CompletionItem` specific
     * capabilities.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_item: Option<CompletionItemCapability>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_item_kind: Option<CompletionItemKindCapability>,

    /**
     * The client supports to send additional context information for a
     * `textDocument/completion` requestion.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_support: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureInformationSettings {
    /**
     * Client supports the follow content formats for the documentation
     * property. The order describes the preferred format of the client.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_format: Option<Vec<MarkupKind>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_information: Option<ParameterInformationSettings>,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParameterInformationSettings {
    /**
    * The client supports processing label offsets instead of a
    * simple label string.
    */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_offset_support: Option<bool>
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureHelpCapability {
    /**
     * Whether completion supports dynamic registration.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /**
     * The client supports the following `SignatureInformation`
     * specific properties.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_information: Option<SignatureInformationSettings>,
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishDiagnosticsCapability {
    /**
     * Whether the clients accepts diagnostics with related information.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_information: Option<bool>,
}

/**
 * Text document specific client capabilities.
 */
#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentClientCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synchronization: Option<SynchronizationCapability>,
    /**
     * Capabilities specific to the `textDocument/completion`
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion: Option<CompletionCapability>,

    /**
     * Capabilities specific to the `textDocument/hover`
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hover: Option<HoverCapability>,

    /**
     * Capabilities specific to the `textDocument/signatureHelp`
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_help: Option<SignatureHelpCapability>,

    /**
     * Capabilities specific to the `textDocument/references`
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<GenericCapability>,

    /**
     * Capabilities specific to the `textDocument/documentHighlight`
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_highlight: Option<GenericCapability>,

    /**
     * Capabilities specific to the `textDocument/documentSymbol`
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_symbol: Option<DocumentSymbolCapability>,
    /**
     * Capabilities specific to the `textDocument/formatting`
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatting: Option<GenericCapability>,

    /**
     * Capabilities specific to the `textDocument/rangeFormatting`
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_formatting: Option<GenericCapability>,

    /**
     * Capabilities specific to the `textDocument/onTypeFormatting`
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_type_formatting: Option<GenericCapability>,

    /**
     * Capabilities specific to the `textDocument/declaration`
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaration: Option<GotoCapability>,

    /**
     * Capabilities specific to the `textDocument/definition`
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub definition: Option<GotoCapability>,

    /**
     * Capabilities specific to the `textDocument/typeDefinition`
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_definition: Option<GotoCapability>,

    /**
     * Capabilities specific to the `textDocument/implementation`
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation: Option<GotoCapability>,

    /**
     * Capabilities specific to the `textDocument/codeAction`
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_action: Option<CodeActionCapability>,

    /**
     * Capabilities specific to the `textDocument/codeLens`
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_lens: Option<GenericCapability>,

    /**
     * Capabilities specific to the `textDocument/documentLink`
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_link: Option<GenericCapability>,

    /**
     * Capabilities specific to the `textDocument/documentColor` and the
     * `textDocument/colorPresentation` request.
     */
    pub color_provider: Option<GenericCapability>,

    /**
     * Capabilities specific to the `textDocument/rename`
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rename: Option<RenameCapability>,

    /**
     * Capabilities specific to `textDocument/publishDiagnostics`.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub publish_diagnostics: Option<PublishDiagnosticsCapability>,

    /**
     * Capabilities specific to `textDocument/foldingRange` requests.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folding_range: Option<FoldingRangeCapability>,
}

/**
 * Window specific client capabilities.
 */
#[derive(Debug, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowClientCapabilities {
    /**
     * Whether `window/progress` server notifications are supported.
     */
    #[cfg(feature = "proposed")]
    pub progress: Option<bool>,
}

/**
 * Where ClientCapabilities are currently empty:
 */
#[derive(Debug, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    /**
     * Workspace specific client capabilities.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<WorkspaceClientCapabilities>,

    /**
     * Text document specific client capabilities.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_document: Option<TextDocumentClientCapabilities>,

    /**
     * Window specific client capabilities.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<WindowClientCapabilities>,

    /**
     * Experimental client capabilities.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<Value>,
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
pub struct InitializeResult {
    /// The capabilities the language server provides.
    pub capabilities: ServerCapabilities,
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
pub struct InitializeError {
    /// Indicates whether the client should retry to send the
    /// initilize request after showing the message provided
    /// in the ResponseError.
    pub retry: bool,
}

// The server can signal the following capabilities:

/// Defines how the host (editor) should sync document changes to the language server.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum TextDocumentSyncKind {
    /// Documents should not be synced at all.
    None = 0,

    /// Documents are synced by always sending the full content of the document.
    Full = 1,

    /// Documents are synced by sending the full content on open. After that only
    /// incremental updates to the document are sent.
    Incremental = 2,
}

impl<'de> serde::Deserialize<'de> for TextDocumentSyncKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(match try!(u8::deserialize(deserializer)) {
            0 => TextDocumentSyncKind::None,
            1 => TextDocumentSyncKind::Full,
            2 => TextDocumentSyncKind::Incremental,
            i => {
                return Err(D::Error::invalid_value(
                    de::Unexpected::Unsigned(i as u64),
                    &"value between 0 and 2 (inclusive)",
                ));
            }
        })
    }
}

impl serde::Serialize for TextDocumentSyncKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

/// Completion options.
#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionOptions {
    /// The server provides support to resolve additional information for a completion item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolve_provider: Option<bool>,

    /// The characters that trigger completion automatically.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_characters: Option<Vec<String>>,
}

/// Signature help options.
#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureHelpOptions {
    /// The characters that trigger signature help automatically.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_characters: Option<Vec<String>>,
}

/// Code Lens options.
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeLensOptions {
    /// Code lens has a resolve provider as well.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolve_provider: Option<bool>,
}

/// Format document on type options
#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentOnTypeFormattingOptions {
    /// A character on which formatting should be triggered, like `}`.
    pub first_trigger_character: String,

    /// More trigger characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub more_trigger_character: Option<Vec<String>>,
}

/// Execute command options.
#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
pub struct ExecuteCommandOptions {
    /// The commands to be executed on the server
    pub commands: Vec<String>,
}

/**
 * Save options.
 */
#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveOptions {
    /**
     * The client is supposed to include the content on save.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_text: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentSyncOptions {
    /**
     * Open and close notifications are sent to the server.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_close: Option<bool>,

    /**
     * Change notifications are sent to the server. See TextDocumentSyncKind.None, TextDocumentSyncKind.Full
     * and TextDocumentSyncKindIncremental.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change: Option<TextDocumentSyncKind>,

    /**
     * Will save notifications are sent to the server.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_save: Option<bool>,

    /**
     * Will save wait until requests are sent to the server.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub will_save_wait_until: Option<bool>,

    /**
     * Save notifications are sent to the server.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save: Option<SaveOptions>,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TextDocumentSyncCapability {
    Kind(TextDocumentSyncKind),
    Options(TextDocumentSyncOptions),
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ImplementationProviderCapability {
    Simple(bool),
    Options(StaticTextDocumentRegistrationOptions),
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TypeDefinitionProviderCapability {
    Simple(bool),
    Options(StaticTextDocumentRegistrationOptions),
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ColorProviderCapability {
    Simple(bool),
    ColorProvider(ColorProviderOptions),
    Options(StaticTextDocumentColorProviderOptions),
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CodeActionProviderCapability {
    Simple(bool),
    Options(CodeActionOptions),
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
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
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeActionLiteralSupport {
    /// The code action kind is support with the following value set.
    pub code_action_kind: CodeActionKindLiteralSupport,
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeActionKindLiteralSupport {
    /// The code action kind values the client supports. When this
    /// property exists the client also guarantees that it will
    /// handle values outside its set gracefully and falls back
    /// to a default value when unknown.
    pub value_set: Vec<String>,
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    /// Defines how text documents are synced.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_document_sync: Option<TextDocumentSyncCapability>,

    /// The server provides hover support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hover_provider: Option<bool>,

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

    /// The server provides color provider support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_provider: Option<ColorProviderCapability>,

    /// The server provides folding provider support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folding_range_provider: Option<FoldingRangeProviderCapability>,

    /// The server provides execute command support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execute_command_provider: Option<ExecuteCommandOptions>,

    /// Workspace specific server capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<WorkspaceCapability>,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct ShowMessageParams {
    /// The message type. See {@link MessageType}.
    #[serde(rename = "type")]
    pub typ: MessageType,

    /// The actual message.
    pub message: String,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
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

impl<'de> serde::Deserialize<'de> for MessageType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(match try!(u8::deserialize(deserializer)) {
            1 => MessageType::Error,
            2 => MessageType::Warning,
            3 => MessageType::Info,
            4 => MessageType::Log,
            i => {
                return Err(D::Error::invalid_value(
                    de::Unexpected::Unsigned(i as u64),
                    &"value of 1, 2, 3 or 4",
                ));
            }
        })
    }
}

impl serde::Serialize for MessageType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
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

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct MessageActionItem {
    /// A short title like 'Retry', 'Open Log' etc.
    pub title: String,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct LogMessageParams {
    /// The message type. See {@link MessageType}
    #[serde(rename = "type")]
    pub typ: MessageType,

    /// The actual message
    pub message: String,
}

/**
 * General parameters to to register for a capability.
 */
#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Registration {
    /**
     * The id used to register the request. The id can be used to deregister
     * the request again.
     */
    pub id: String,

    /**
     * The method / capability to register for.
     */
    pub method: String,

    /**
     * Options necessary for the registration.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub register_options: Option<Value>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct RegistrationParams {
    pub registrations: Vec<Registration>,
}

/// Since most of the registration options require to specify a document selector there is a base
/// interface that can be used.
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentRegistrationOptions {
    /**
     * A document selector to identify the scope of the registration. If set to null
     * the document selector provided on the client side will be used.
     */
    pub document_selector: Option<DocumentSelector>,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticRegistrationOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticTextDocumentRegistrationOptions {
    /**
     * A document selector to identify the scope of the registration. If set to null
     * the document selector provided on the client side will be used.
     */
    pub document_selector: Option<DocumentSelector>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorProviderOptions {}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticTextDocumentColorProviderOptions {
    /**
     * A document selector to identify the scope of the registration. If set to null
     * the document selector provided on the client side will be used.
     */
    pub document_selector: Option<DocumentSelector>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/**
 * General parameters to unregister a capability.
 */
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Unregistration {
    /**
     * The id used to unregister the request or notification. Usually an id
     * provided during the register request.
     */
    pub id: String,

    /**
     * The method / capability to unregister for.
     */
    pub method: String,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct UnregistrationParams {
    pub unregisterations: Vec<Unregistration>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct DidChangeConfigurationParams {
    /// The actual changed settings
    pub settings: Value,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DidOpenTextDocumentParams {
    /// The document that was opened.
    pub text_document: TextDocumentItem,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
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
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
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

/**
 * Descibe options to be used when registered for text document change events.
 *
 * Extends TextDocumentRegistrationOptions
 */
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentChangeRegistrationOptions {
    /**
     * A document selector to identify the scope of the registration. If set to null
     * the document selector provided on the client side will be used.
     */
    pub document_selector: Option<DocumentSelector>,

    /**
     * How documents are synced to the server. See TextDocumentSyncKind.Full
     * and TextDocumentSyncKindIncremental.
     */
    pub sync_kind: i32,
}

/**
 * The parameters send in a will save text document notification.
 */
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WillSaveTextDocumentParams {
    /**
     * The document that will be saved.
     */
    pub text_document: TextDocumentIdentifier,

    /**
     * The 'TextDocumentSaveReason'.
     */
    pub reason: TextDocumentSaveReason,
}

/**
 * Represents reasons why a text document is saved.
 */
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextDocumentSaveReason {
    /**
     * Manually triggered, e.g. by the user pressing save, by starting debugging,
     * or by an API call.
     */
    Manual = 1,

    /**
     * Automatic after a delay.
     */
    AfterDelay = 2,

    /**
     * When the editor lost focus.
     */
    FocusOut = 3,
}

impl<'de> serde::Deserialize<'de> for TextDocumentSaveReason {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(match try!(u8::deserialize(deserializer)) {
            1 => TextDocumentSaveReason::Manual,
            2 => TextDocumentSaveReason::AfterDelay,
            3 => TextDocumentSaveReason::FocusOut,
            i => {
                return Err(D::Error::invalid_value(
                    de::Unexpected::Unsigned(i as u64),
                    &"value of 1, 2 or 3",
                ))
            }
        })
    }
}

impl serde::Serialize for TextDocumentSaveReason {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DidCloseTextDocumentParams {
    /// The document that was closed.
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DidSaveTextDocumentParams {
    /// The document that was saved.
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct DidChangeWatchedFilesParams {
    /// The actual file events.
    pub changes: Vec<FileEvent>,
}

/// The file event type.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum FileChangeType {
    /// The file got created.
    Created = 1,

    /// The file got changed.
    Changed = 2,

    /// The file got deleted.
    Deleted = 3,
}

impl<'de> serde::Deserialize<'de> for FileChangeType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(match try!(u8::deserialize(deserializer)) {
            1 => FileChangeType::Created,
            2 => FileChangeType::Changed,
            3 => FileChangeType::Deleted,
            i => {
                return Err(D::Error::invalid_value(
                    de::Unexpected::Unsigned(i as u64),
                    &"value of 1, 2 or 3",
                ))
            }
        })
    }
}

impl serde::Serialize for FileChangeType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

/// An event describing a file change.
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct FileEvent {
    /// The file's URI.
    #[serde(with = "url_serde")]
    pub uri: Url,

    /// The change type.
    #[serde(rename = "type")]
    pub typ: FileChangeType,
}

impl FileEvent {
    pub fn new(uri: Url, typ: FileChangeType) -> FileEvent {
        FileEvent { uri: uri, typ: typ }
    }
}

/// Describe options to be used when registered for text document change events.
#[derive(Debug, Deserialize, Serialize)]
pub struct DidChangeWatchedFilesRegistrationOptions {
    /// The watchers to register.
    pub watchers: Vec<FileSystemWatcher>,
}

#[derive(Debug, Deserialize, Serialize)]
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
        let i = try!(u8::deserialize(deserializer));
        WatchKind::from_bits(i).ok_or_else(|| {
            D::Error::invalid_value(de::Unexpected::Unsigned(i as u64), &"Unknown flag")
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

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct PublishDiagnosticsParams {
    /// The URI for which diagnostic information is reported.
    #[serde(with = "url_serde")]
    pub uri: Url,

    /// An array of diagnostic information items.
    pub diagnostics: Vec<Diagnostic>,
}

impl PublishDiagnosticsParams {
    pub fn new(uri: Url, diagnostics: Vec<Diagnostic>) -> PublishDiagnosticsParams {
        PublishDiagnosticsParams {
            uri: uri,
            diagnostics: diagnostics,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CompletionResponse {
    Array(Vec<CompletionItem>),
    List(CompletionList),
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionParams {
    // This field was "mixed-in" from TextDocumentPositionParams
    /// The text document.
    pub text_document: TextDocumentIdentifier,

    // This field was "mixed-in" from TextDocumentPositionParams
    /// The position inside the text document.
    pub position: Position,

    // CompletionParams properties:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<CompletionContext>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionContext {
    /**
     * How the completion was triggered.
     */
    pub trigger_kind: CompletionTriggerKind,

    /**
     * The trigger character (a single character) that has trigger code complete.
     * Is undefined if `triggerKind !== CompletionTriggerKind.TriggerCharacter`
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_character: Option<String>,
}

/// How a completion was triggered.
#[derive(Debug, PartialEq, Clone, Copy, FromPrimitive)]
pub enum CompletionTriggerKind {
    Invoked = 1,
    TriggerCharacter = 2,
    TriggerForIncompleteCompletions = 3,
}

impl<'de> serde::Deserialize<'de> for CompletionTriggerKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let i = try!(u8::deserialize(deserializer));
        CompletionTriggerKind::from_u8(i).ok_or_else(|| {
            D::Error::invalid_value(
                de::Unexpected::Unsigned(i as u64),
                &"value between 1 and 3 (inclusive)",
            )
        })
    }
}

impl serde::Serialize for CompletionTriggerKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

/// Represents a collection of [completion items](#CompletionItem) to be presented
/// in the editor.
#[derive(Debug, PartialEq, Default, Deserialize, Serialize)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_edit: Option<TextEdit>,

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
}

impl CompletionItem {
    /// Create a CompletionItem with the minimum possible info (label and detail).
    pub fn new_simple(label: String, detail: String) -> CompletionItem {
        CompletionItem {
            label: label,
            detail: Some(detail),
            ..Self::default()
        }
    }
}

/// The kind of a completion entry.
#[derive(Debug, Eq, PartialEq, Clone, Copy, FromPrimitive)]
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

impl<'de> serde::Deserialize<'de> for CompletionItemKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let i = try!(u8::deserialize(deserializer));
        CompletionItemKind::from_u8(i).ok_or_else(|| {
            D::Error::invalid_value(
                de::Unexpected::Unsigned(i as u64),
                &"value between 1 and 18 (inclusive)",
            )
        })
    }
}

impl serde::Serialize for CompletionItemKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

/// Defines how to interpret the insert text in a completion item
#[derive(Debug, Eq, PartialEq, Clone, Copy, FromPrimitive)]
pub enum InsertTextFormat {
    PlainText = 1,
    Snippet = 2,
}

impl<'de> serde::Deserialize<'de> for InsertTextFormat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let i = try!(u8::deserialize(deserializer));
        InsertTextFormat::from_u8(i).ok_or_else(|| {
            D::Error::invalid_value(
                de::Unexpected::Unsigned(i as u64),
                &"value between 1 and 2 (inclusive)",
            )
        })
    }
}

impl serde::Serialize for InsertTextFormat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
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

/**
 * Hover contents could be single entry or multiple entries.
 */
#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HoverContents {
    Scalar(MarkedString),
    Array(Vec<MarkedString>),
    Markup(MarkupContent),
}

/**
 The marked string is rendered:
 - as markdown if it is represented as a string
 - as code block of the given langauge if it is represented as a pair of a language and a value

 The pair of a language and a value is an equivalent to markdown:
     ```${language}
     ${value}
     ```
 */
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
            language: language,
            value: code_block,
        })
    }
}

/// Signature help represents the signature of something
/// callable. There can be multiple signature but only one
/// active and only one active parameter.
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
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
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
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
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
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

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ParameterLabel {
    Simple(String),
    LabelOffsets([u64; 2]),
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceParams {
    // This field was "mixed-in" from TextDocumentPositionParams
    /// The text document.
    pub text_document: TextDocumentIdentifier,

    // This field was "mixed-in" from TextDocumentPositionParams
    /// The position inside the text document.
    pub position: Position,

    // ReferenceParams properties:
    pub context: ReferenceContext,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceContext {
    /// Include the declaration of the current symbol.
    pub include_declaration: bool,
}

/// A document highlight is a range inside a text document which deserves
/// special attention. Usually a document highlight is visualized by changing
/// the background color of its range.
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct DocumentHighlight {
    /// The range this highlight applies to.
    pub range: Range,

    /// The highlight kind, default is DocumentHighlightKind.Text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<DocumentHighlightKind>,
}

/// A document highlight kind.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum DocumentHighlightKind {
    /// A textual occurrance.
    Text = 1,

    /// Read-access of a symbol, like reading a variable.
    Read = 2,

    /// Write-access of a symbol, like writing to a variable.
    Write = 3,
}

impl<'de> serde::Deserialize<'de> for DocumentHighlightKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(match try!(u8::deserialize(deserializer)) {
            1 => DocumentHighlightKind::Text,
            2 => DocumentHighlightKind::Read,
            3 => DocumentHighlightKind::Write,
            i => {
                return Err(D::Error::invalid_value(
                    de::Unexpected::Unsigned(i as u64),
                    &"1, 2, or 3",
                ))
            }
        })
    }
}

impl serde::Serialize for DocumentHighlightKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSymbolCapability {
    /// This capability supports dynamic registration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /// Specific capabilities for the `SymbolKind`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol_kind: Option<SymbolKindCapability>,

    /// The client support hierarchical document symbols.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hierarchical_document_symbol_support: Option<bool>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DocumentSymbolResponse {
    Flat(Vec<SymbolInformation>),
    Nested(Vec<DocumentSymbol>),
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSymbolParams {
    /// The text document.
    pub text_document: TextDocumentIdentifier,
}

/// Represents programming constructs like variables, classes, interfaces etc.
/// that appear in a document. Document symbols can be hierarchical and they have two ranges:
/// one that encloses its definition and one that points to its most interesting range,
/// e.g. the range of an identifier.
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
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
    /// Indicates if this symbol is deprecated.
    #[serde(skip_serializing_if = "Option::is_none")]
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
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolInformation {
    /// The name of this symbol.
    pub name: String,

    /// The kind of this symbol.
    pub kind: SymbolKind,

    /// Indicates if this symbol is deprecated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<bool>,

    /// The location of this symbol.
    pub location: Location,

    /// The name of the symbol containing this symbol.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_name: Option<String>,
}

/// A symbol kind.
#[derive(Debug, Eq, PartialEq, Copy, Clone, FromPrimitive)]
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

impl<'de> serde::Deserialize<'de> for SymbolKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let i = try!(u8::deserialize(deserializer));
        Ok(SymbolKind::from_u8(i).unwrap_or(SymbolKind::Unknown))
    }
}

impl serde::Serialize for SymbolKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

/// The parameters of a Workspace Symbol Request.
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct WorkspaceSymbolParams {
    /// A non-empty query string
    pub query: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ExecuteCommandParams {
    /**
     * The identifier of the actual command handler.
     */
    pub command: String,
    /**
     * Arguments that the command should be invoked with.
     */
    #[serde(default)]
    pub arguments: Vec<Value>,
}

/**
 * Execute command registration options.
 */
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct ExecuteCommandRegistrationOptions {
    /**
     * The commands to be executed on the server
     */
    pub commands: Vec<String>,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct ApplyWorkspaceEditParams {
    /**
     * The edits to apply.
     */
    pub edit: WorkspaceEdit,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct ApplyWorkspaceEditResponse {
    /**
     * Indicates whether the edit was applied or not.
     */
    pub applied: bool,
}

/// Params for the CodeActionRequest
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeActionParams {
    /// The document in which the command was invoked.
    pub text_document: TextDocumentIdentifier,

    /// The range for which the command was invoked.
    pub range: Range,

    /// Context carrying additional information.
    pub context: CodeActionContext,
}

/// response for CodeActionRequest
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CodeActionResponse {
    Commands(Vec<Command>),
    Actions(Vec<CodeAction>),
}

/**
 * A set of predefined code action kinds
 */
pub mod code_action_kind {

    /**
     * Base kind for quickfix actions: 'quickfix'
     */
    pub const QUICKFIX: &'static str = "quickfix";

    /**
     * Base kind for refactoring actions: 'refactor'
     */
    pub const REFACTOR: &'static str = "refactor";

    /**
     * Base kind for refactoring extraction actions: 'refactor.extract'
     *
     * Example extract actions:
     *
     * - Extract method
     * - Extract function
     * - Extract variable
     * - Extract interface from class
     * - ...
     */
    pub const REFACTOR_EXTRACT: &'static str = "refactor.extract";

    /**
     * Base kind for refactoring inline actions: 'refactor.inline'
     *
     * Example inline actions:
     *
     * - Inline function
     * - Inline variable
     * - Inline constant
     * - ...
     */
    pub const REFACTOR_INLINE: &'static str = "refactor.inline";

    /**
     * Base kind for refactoring rewrite actions: 'refactor.rewrite'
     *
     * Example rewrite actions:
     *
     * - Convert JavaScript function to class
     * - Add or remove parameter
     * - Encapsulate field
     * - Make method static
     * - Move method to base class
     * - ...
     */
    pub const REFACTOR_REWRITE: &'static str = "refactor.rewrite";

    /**
     * Base kind for source actions: `source`
     *
     * Source code actions apply to the entire file.
     */
    pub const SOURCE: &'static str = "source";

    /**
     * Base kind for an organize imports source action: `source.organizeImports`
     */
    pub const SOURCE_ORGANIZE_IMPORTS: &'static str = "source.organizeImports";
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct CodeAction {
    /// A short, human-readable, title for this code action.
    pub title: String,

    /// The kind of the code action.
    /// Used to filter code actions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,

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
}

/// Contains additional diagnostic information about the context in which
/// a code action is run.
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct CodeActionContext {
    /// An array of diagnostics.
    pub diagnostics: Vec<Diagnostic>,

    /// Requested kind of actions to return.
    ///
    /// Actions not of this kind are filtered out by the client before being shown. So servers
    /// can omit computing them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only: Option<Vec<String>>,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeActionOptions {
    /**
     * CodeActionKinds that this server may return.
     *
     * The list of kinds may be generic, such as `CodeActionKind.Refactor`, or the server
     * may list out every specific kind they provide.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_action_kinds: Option<Vec<String>>,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeLensParams {
    /// The document to request code lens for.
    pub text_document: TextDocumentIdentifier,
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

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentLinkParams {
    /**
     * The document to provide document links for.
     */
    pub text_document: TextDocumentIdentifier,
}

/**
 * A document link is a range in a text document that links to an internal or external resource, like another
 * text document or a web site.
 */
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct DocumentLink {
    /**
     * The range this link applies to.
     */
    pub range: Range,
    /**
     * The uri this link points to.
     */
    #[serde(with = "url_serde")]
    pub target: Url,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentFormattingParams {
    /// The document to format.
    pub text_document: TextDocumentIdentifier,

    /// The format options.
    pub options: FormattingOptions,
}

/// Value-object describing what options formatting should use.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormattingOptions {
    /// Size of a tab in spaces.
    pub tab_size: u64,

    /// Prefer spaces over tabs.
    pub insert_spaces: bool,

    /// Signature for further properties.
    #[serde(flatten)]
    pub properties: HashMap<String, FormattingProperty>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum FormattingProperty {
    Bool(bool),
    Number(f64),
    String(String),
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentRangeFormattingParams {
    /// The document to format.
    pub text_document: TextDocumentIdentifier,

    /// The range to format
    pub range: Range,

    /// The format options
    pub options: FormattingOptions,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentOnTypeFormattingParams {
    /// The document to format.
    pub text_document: TextDocumentIdentifier,

    /// The position at which this request was sent.
    pub position: Position,

    /// The character that has been typed.
    pub ch: String,

    /// The format options.
    pub options: FormattingOptions,
}

/// Extends TextDocumentRegistrationOptions
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentOnTypeFormattingRegistrationOptions {
    /**
     * A document selector to identify the scope of the registration. If set to null
     * the document selector provided on the client side will be used.
     */
    pub document_selector: Option<DocumentSelector>,

    /**
     * A character on which formatting should be triggered, like `}`.
     */
    pub first_trigger_character: String,

    /**
     * More trigger characters.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub more_trigger_character: Option<Vec<String>>,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameParams {
    /// The document to format.
    pub text_document: TextDocumentIdentifier,

    /// The position at which this request was sent.
    pub position: Position,

    /// The new name of the symbol. If the given name is not valid the
    /// request must return a [ResponseError](#ResponseError) with an
    /// appropriate message set.
    pub new_name: String,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RenameProviderCapability {
    Simple(bool),
    Options(RenameOptions),
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameOptions {
    /// Renames should be checked and tested before being executed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepare_provider: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameCapability {
    /// Whether rename supports dynamic registration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /// Client supports testing for validity of rename operations before execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepare_support: Option<bool>,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PrepareRenameResponse {
    Range(Range),
    RangeWithPlaceholder { range: Range, placeholder: String },
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentColorParams {
    /// The text document
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorInformation {
    /**
     * The range in the document where this color appears.
     */
    pub range: Range,
    /**
     * The actual color value for this color range.
     */
    pub color: Color,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Color {
    /**
     * The red component of this color in the range [0-1].
     */
    pub red: f64,
    /**
     * The green component of this color in the range [0-1].
     */
    pub green: f64,
    /**
     * The blue component of this color in the range [0-1].
     */
    pub blue: f64,
    /**
     * The alpha component of this color in the range [0-1].
     */
    pub alpha: f64,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorPresentationParams {
    /**
     * The text document.
     */
    pub text_document: TextDocumentIdentifier,

    /**
     * The color information to request presentations for.
     */
    pub color: Color,

    /**
     * The range where the color would be inserted. Serves as a context.
     */
    pub range: Range,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ColorPresentation {
    /**
     * The label of this color presentation. It will be shown on the color
     * picker header. By default this is also the text that is inserted when selecting
     * this color presentation.
     */
    pub label: String,

    /**
     * An [edit](#TextEdit) which is applied to a document when selecting
     * this presentation for the color.  When `falsy` the [label](#ColorPresentation.label)
     * is used.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_edit: Option<TextEdit>,

    /**
     * An optional array of additional [text edits](#TextEdit) that are applied when
     * selecting this color presentation. Edits must not overlap with the main [edit](#ColorPresentation.textEdit) nor with themselves.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_text_edits: Option<Vec<TextEdit>>,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FoldingRangeParams {
    /// The text document.
    pub text_document: TextDocumentIdentifier,
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum FoldingRangeProviderCapability {
    Simple(bool),
    FoldingProvider(FoldingProviderOptions),
    Options(StaticTextDocumentColorProviderOptions),
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct FoldingProviderOptions {}

#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FoldingRangeCapability {
    /**
     * Whether implementation supports dynamic registration for folding range providers. If this is set to `true`
     * the client supports the new `(FoldingRangeProviderOptions & TextDocumentRegistrationOptions & StaticRegistrationOptions)`
     * return value for the corresponding server capability as well.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_registration: Option<bool>,

    /**
     * The maximum number of folding ranges that the client prefers to receive per document. The value serves as a
     * hint, servers are free to follow the limit.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range_limit: Option<u64>,
    /**
     * If set, the client signals that it only supports folding complete lines. If set, client will
     * ignore specified `startCharacter` and `endCharacter` properties in a FoldingRange.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_folding_only: Option<bool>,
}

/**
 * Represents a folding range.
 */
#[derive(Debug, Eq, PartialEq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FoldingRange {
    /**
     * The zero-based line number from where the folded range starts.
     */
    pub start_line: u64,

    /**
     * The zero-based character offset from where the folded range starts. If not defined, defaults to the length of the start line.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_character: Option<u64>,

    /**
     * The zero-based line number where the folded range ends.
     */
    pub end_line: u64,

    /**
     * The zero-based character offset before the folded range ends. If not defined, defaults to the length of the end line.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_character: Option<u64>,

    /**
     * Describes the kind of the folding range such as `comment' or 'region'. The kind
     * is used to categorize folding ranges and used by commands like 'Fold all comments'. See
     * [FoldingRangeKind](#FoldingRangeKind) for an enumeration of standardized kinds.
     */
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<FoldingRangeKind>,
}

/**
 * Enum of known range kinds
 */
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

/**
 * Describes the content type that a client supports in various
 * result literals like `Hover`, `ParameterInfo` or `CompletionItem`.
 *
 * Please note that `MarkupKinds` must not start with a `$`. This kinds
 * are reserved for internal usage.
 */
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MarkupKind {
    /// Plain text is supported as a content format
    PlainText,
    /// Markdown is supported as a content format
    Markdown,
}

/**
 * A `MarkupContent` literal represents a string value which content is interpreted base on its
 * kind flag. Currently the protocol supports `plaintext` and `markdown` as markup kinds.
 *
 * If the kind is `markdown` then the value can contain fenced code blocks like in GitHub issues.
 * See <https://help.github.com/articles/creating-and-highlighting-code-blocks/#syntax-highlighting>
 *
 * Here is an example how such a string can be constructed using JavaScript / TypeScript:
 * ```ts
 * let markdown: MarkdownContent = {
 *  kind: MarkupKind.Markdown,
 *	value: [
 *		'# Header',
 *		'Some text',
 *		'```typescript',
 *		'someCode();',
 *		'```'
 *	].join('\n')
 * };
 * ```
 *
 * *Please Note* that clients might sanitize the return markdown. A client could decide to
 * remove HTML from the markdown to avoid script execution.
 */
#[derive(Debug, Eq, PartialEq, Deserialize, Serialize, Clone)]
pub struct MarkupContent {
    pub kind: MarkupKind,
    pub value: String,
}

#[cfg(feature = "proposed")]
/// The progress notification is sent from the server to the client to ask
/// the client to indicate progress.
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct ProgressParams {
    /// A unique identifier to associate multiple progress notifications
    /// with the same progress.
    pub id: String,

    /// Mandatory title of the progress operation. Used to briefly inform
    /// about the kind of operation being performed.
    /// Examples: "Indexing" or "Linking dependencies".
    pub title: String,

    /// Optional, more detailed associated progress message. Contains
    /// complementary information to the `title`.
    /// Examples: "3/25 files", "project/src/module2", "node_modules/some_dep".
    /// If unset, the previous progress message (if any) is still valid.
    pub message: Option<String>,

    /// Optional progress percentage to display (value 100 is considered 100%).
    /// If unset, the previous progress percentage (if any) is still valid.
    pub percentage: Option<f64>,

    /// Set to true on the final progress update.
    /// No more progress notifications with the same ID should be sent.
    pub done: Option<bool>,
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

        test_serialization(&WorkspaceEdit {
                                changes: Some(vec![(Url::parse("file://test").unwrap(), vec![])]
                                    .into_iter()
                                    .collect()),
                                document_changes: None,
                            },
                           r#"{"changes":{"file://test/":[]}}"#);
    }

    #[test]
    fn formatting_options() {
        test_serialization(
            &FormattingOptions {
                tab_size: 123,
                insert_spaces: true,
                properties: HashMap::new(),
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
            &vec![ResourceOperationKind::Create, ResourceOperationKind::Rename, ResourceOperationKind::Delete],
            r#"["create","rename","delete"]"#);
    }
}
