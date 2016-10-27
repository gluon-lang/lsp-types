//! Language Server Protocol types for Rust.
//! Based on: https://github.com/Microsoft/language-server-protocol/blob/master/protocol.md
//! Last protocol update 14/Oct/2016 at commit: 
//! https://github.com/Microsoft/language-server-protocol/commit/63f5d02d39d0135c234162a28d0523c9323ab3f7


#![feature(proc_macro)]

#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

#[macro_use]
extern crate enum_primitive;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::collections::HashMap;

use serde::Serialize;
use serde::Deserialize;
use serde::de;
use serde::de::Error;
use serde_json::Value;

#[derive(Debug, PartialEq, Clone)]
pub enum NumberOrString {
    Number(u64),
    String(String),
}

impl Serialize for NumberOrString {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        match *self {
            NumberOrString::Number(number) => serializer.serialize_u64(number),
            NumberOrString::String(ref string) => serializer.serialize_str(string),
        }
    }
}

impl Deserialize for NumberOrString {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> 
        where D: serde::Deserializer
    {
        #[allow(non_camel_case_types)]
        struct NumberOrString_Visitor;
        impl de::Visitor for NumberOrString_Visitor {
            type Value = NumberOrString;
            
            fn visit_u64<E>(&mut self, value: u64) -> Result<Self::Value, E> where E: de::Error {
                Ok(NumberOrString::Number(value))
            }

            fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E> where E: de::Error {
                Ok(NumberOrString::String(value.to_string()))
            }
        }
        
        deserializer.deserialize(NumberOrString_Visitor)
    }
}


#[test]
fn test_NumberOrString() {
    
    test_serialization(
        &NumberOrString::Number(123),
        r#"123"#
    );
    
    test_serialization(
        &NumberOrString::String("abcd".into()),
        r#""abcd""#
    );
}

/// The base protocol now offers support for request cancellation. To cancel a request, 
/// a notification message with the following properties is sent:
///
/// A request that got canceled still needs to return from the server and send a response back. 
/// It can not be left open / hanging. This is in line with the JSON RPC protocol that requires 
/// that every request sends a response back. In addition it allows for returning partial results on cancel.
pub const NOTIFICATION__Cancel: &'static str = "$/cancelRequest";

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct CancelParams {
    /// The request id to cancel.
    pub id: NumberOrString,
}

/* ----------------- Basic JSON Structures ----------------- */

/// Position in a text document expressed as zero-based line and character offset. 
/// A position is between two characters like an 'insert' cursor in a editor.
#[derive(Debug, PartialEq, Copy, Clone, Default, Deserialize, Serialize)]
pub struct Position {
    /// Line position in a document (zero-based).
    pub line: u64,
    /// Character offset on a line in a document (zero-based).
    pub character: u64,
}

impl Position {
    pub fn new(line: u64, character: u64) -> Position {
        Position { line : line, character : character }
    }
}

/// A range in a text document expressed as (zero-based) start and end positions. 
/// A range is comparable to a selection in an editor. Therefore the end position is exclusive.
#[derive(Debug, PartialEq, Copy, Clone, Default, Deserialize, Serialize)]
pub struct Range {
    /// The range's start position.
    pub start: Position,
    /// The range's end position.
    pub end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Range {
        Range { start : start, end : end }
    }
}

/// Represents a location inside a resource, such as a line inside a text file.
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}

impl Location {
    pub fn new(uri: String, range: Range) -> Location {
        Location { uri : uri, range : range }
    }
}

/// Represents a diagnostic, such as a compiler error or warning. 
/// Diagnostic objects are only valid in the scope of a resource.
#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
pub struct Diagnostic {
    /// The range at which the message applies.
    pub range: Range,

    /// The diagnostic's severity. Can be omitted. If omitted it is up to the
    /// client to interpret diagnostics as error, warning, info or hint.
    pub severity: Option<DiagnosticSeverity>,

    /// The diagnostic's code. Can be omitted.
    pub code: Option<NumberOrString>,
//    code?: number | string;

    /// A human-readable string describing the source of this
    /// diagnostic, e.g. 'typescript' or 'super lint'.
    pub source: Option<String>,

    /// The diagnostic's message.
    pub message: String,
}

impl Diagnostic {
    
    pub fn new(
        range: Range, 
        severity: Option<DiagnosticSeverity>, 
        code: Option<NumberOrString>, 
        source: Option<String>, 
        message: String
    ) -> Diagnostic 
    {
        Diagnostic { 
            range : range,
            severity : severity,
            code : code,
            source : source,  
            message : message 
        }
    }
    
    pub fn new_simple(range: Range, message: String) -> Diagnostic {
        Self::new(range, None, None, None, message)
    }
    
    pub fn new_with_code_number(
        range: Range, 
        severity: DiagnosticSeverity, 
        code_number: u64, 
        source: Option<String>, 
        message: String
    ) -> Diagnostic 
    {
        let code = Some(NumberOrString::Number(code_number));
        Self::new(range, Some(severity), code, source, message)
    }
    
}

/// The protocol currently supports the following diagnostic severities:
#[derive(Debug, PartialEq, Clone, Copy)]
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

impl serde::Deserialize for DiagnosticSeverity {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        Ok(match try!(u8::deserialize(deserializer)) {
            1 => DiagnosticSeverity::Error,
            2 => DiagnosticSeverity::Warning,
            3 => DiagnosticSeverity::Information,
            4 => DiagnosticSeverity::Hint,
            _ => {
                return Err(D::Error::invalid_value("Expected a value of 1, 2, 3 or 4 to \
                                                    deserialize to DiagnosticSeverity"))
            }
        })
    }
}

impl serde::Serialize for DiagnosticSeverity {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
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
    #[serde(skip_serializing_if="Option::is_none")]
    pub arguments: Option<Vec<Value>>,
}

impl Command {
    pub fn new(title: String, command: String, arguments: Option<Vec<Value>>) -> Command {
        Command{ title : title, command : command, arguments : arguments } 
    }
}

/// A textual edit applicable to a text document.
#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
pub struct TextEdit {
    /// The range of the text document to be manipulated. To insert
    /// text into a document create a range where start === end.
    pub range: Range,
    /// The string to be inserted. For delete operations use an
    /// empty string.
    #[serde(rename="newText")]
    pub new_text: String,
}

impl TextEdit {
    pub fn new(range: Range, new_text: String) -> TextEdit {
        TextEdit{ range : range, new_text : new_text } 
    }
}

/// A workspace edit represents changes to many resources managed in the workspace.
#[derive(Debug, PartialEq, Clone, Default, Deserialize, Serialize)]
pub struct WorkspaceEdit {
    /// Holds changes to existing resources.
    pub changes: HashMap<String, Vec<TextEdit>>,
//    changes: { [uri: string]: TextEdit[]; };
}

impl WorkspaceEdit {
    pub fn new(changes: HashMap<String, Vec<TextEdit>>) -> WorkspaceEdit {
        WorkspaceEdit{ changes : changes } 
    }
}

/// Text documents are identified using a URI. On the protocol level, URIs are passed as strings. 
/// The corresponding JSON structure looks like this:
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct TextDocumentIdentifier {
// !!!!!! Note: 
// In the spec VersionedTextDocumentIdentifier extends TextDocumentIdentifier
// This modelled by "mixing-in" TextDocumentIdentifier in VersionedTextDocumentIdentifier,
// so any changes to this type must be effected in the sub-type as well.


    /// The text document's URI.
    pub uri: String,
}

impl TextDocumentIdentifier {
    pub fn new(uri: String) -> TextDocumentIdentifier {
        TextDocumentIdentifier{ uri : uri } 
    }
}

/// An item to transfer a text document from the client to the server. 
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct TextDocumentItem {
    /// The text document's URI.
    pub uri: String,

    /// The text document's language identifier.
    #[serde(rename="languageId")]
    pub language_id: Option<String>,

    /// The version number of this document (it will strictly increase after each
    /// change, including undo/redo).
    pub version: Option<u64>,

    /// The content of the opened text document.
    pub text: String,
}

impl TextDocumentItem {
    pub fn new(uri: String, language_id: Option<String>, version: Option<u64>, text: String) -> TextDocumentItem {
        TextDocumentItem{ uri : uri, language_id : language_id, version : version, text : text,} 
    }
}

/// An identifier to denote a specific version of a text document.
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct VersionedTextDocumentIdentifier 
//extends TextDocumentIdentifier 
{
    // This field was "mixed-in" from TextDocumentIdentifier
    /// The text document's URI.
    pub uri: String,

    /// The version number of this document.
    pub version: u64,
}


impl VersionedTextDocumentIdentifier {
    pub fn new(uri: String, version: u64,) -> VersionedTextDocumentIdentifier {
        VersionedTextDocumentIdentifier{ uri : uri, version : version} 
    }
}


/// A parameter literal used in requests to pass a text document and a position inside that document.
#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct TextDocumentPositionParams {
// !!!!!! Note: 
// In the spec ReferenceParams extends TextDocumentPositionParams
// This modelled by "mixing-in" TextDocumentPositionParams in ReferenceParams,
// so any changes to this type must be effected in sub-type as well.
    
    /// The text document.
    #[serde(rename="textDocument")]
    pub text_document: TextDocumentIdentifier,
    
    /// The position inside the text document.
    pub position: Position,
}


impl TextDocumentPositionParams {
    pub fn new(text_document: TextDocumentIdentifier, position: Position) -> TextDocumentPositionParams {
        TextDocumentPositionParams{ text_document : text_document, position : position} 
    }
}


/* ========================= Actual Protocol ========================= */

/**
 * The initialize request is sent as the first request from the client to the server.
 */
pub const REQUEST__Initialize: &'static str = "initialize";

#[derive(Debug, PartialEq, Deserialize, Serialize)] 
pub struct InitializeParams {
    /// The process Id of the parent process that started
    /// the server. Is null if the process has not been started by another process.
    /// If the parent process is not alive then the server should exit (see exit notification) its process.
    #[serde(rename="processId")]
    pub process_id: Option<u64>,


    /// The rootPath of the workspace. Is null
    /// if no folder is open.
    #[serde(rename="rootPath")]
    pub root_path: Option<String>,

    /// User provided initialization options.
    #[serde(rename="initializationOptions")]
    pub initialization_options: Option<Value>,

    /// The capabilities provided by the client (editor)
    pub capabilities: ClientCapabilities,
}

/**
 * Where ClientCapabilities are currently empty:
 */
pub type ClientCapabilities = Value;

#[derive(Debug, PartialEq, Default, Deserialize, Serialize)]
pub struct InitializeResult {
    /// The capabilities the language server provides.
    pub capabilities: ServerCapabilities,
}

#[derive(Debug, PartialEq, Default, Deserialize, Serialize)]
pub struct InitializeError {
    /// Indicates whether the client should retry to send the
    /// initilize request after showing the message provided
    /// in the ResponseError.
    pub retry: bool,
}

// The server can signal the following capabilities:

/// Defines how the host (editor) should sync document changes to the language server.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TextDocumentSyncKind {
    /// Documents should not be synced at all.
    None = 0,

    /// Documents are synced by always sending the full content of the document.
    Full = 1,
    
    /// Documents are synced by sending the full content on open. After that only 
    /// incremental updates to the document are sent.
    Incremental = 2,
}

impl serde::Deserialize for TextDocumentSyncKind {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        Ok(match try!(u8::deserialize(deserializer)) {
            0 => TextDocumentSyncKind::None,
            1 => TextDocumentSyncKind::Full,
            2 => TextDocumentSyncKind::Incremental,
            _ => {
                return Err(D::Error::invalid_value("Expected a value between 1 and 2 (inclusive) \
                                                    to deserialize to TextDocumentSyncKind"))
            }
        })
    }
}

impl serde::Serialize for TextDocumentSyncKind {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        serializer.serialize_u8(*self as u8)
    }
}

/// Completion options.
#[derive(Debug, PartialEq, Default, Deserialize, Serialize)]
pub struct CompletionOptions {
    /// The server provides support to resolve additional information for a completion item.
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(rename="resolveProvider")]
    pub resolve_provider: Option<bool>,

    /// The characters that trigger completion automatically.
    #[serde(rename="triggerCharacters")]
    pub trigger_characters: Vec<String>,
}

/// Signature help options.
#[derive(Debug, PartialEq, Default, Deserialize, Serialize)]
pub struct SignatureHelpOptions {
    /// The characters that trigger signature help automatically.
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(rename="triggerCharacters")]
    pub trigger_characters: Option<Vec<String>>,
}

/// Code Lens options.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct CodeLensOptions {
    /// Code lens has a resolve provider as well.
    #[serde(rename="resolveProvider")]
    pub resolve_provider: Option<bool>,
}

/// Format document on type options
#[derive(Debug, PartialEq, Default, Deserialize, Serialize)]
pub struct DocumentOnTypeFormattingOptions {
    /// A character on which formatting should be triggered, like `}`.
    #[serde(rename="firstTriggerCharacter")]
    pub first_trigger_character: String,
    
    /// More trigger characters.
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(rename="moreTriggerCharacter")]
    pub more_trigger_character: Option<Vec<String>>,
}

#[derive(Debug, PartialEq, Default, Deserialize, Serialize)]
pub struct ServerCapabilities {
    /// Defines how text documents are synced.
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(rename="textDocumentSync")]
    pub text_document_sync: Option<TextDocumentSyncKind>,
    
    /// The server provides hover support.
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(rename="hoverProvider")]
    pub hover_provider: Option<bool>,
    
    /// The server provides completion support.
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(rename="completionProvider")]
    pub completion_provider: Option<CompletionOptions>,
    
    /// The server provides signature help support.
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(rename="signatureHelpProvider")]
    pub signature_help_provider: Option<SignatureHelpOptions>,
    
    /// The server provides goto definition support.
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(rename="definitionProvider")]
    pub definition_provider: Option<bool>,
    
    /// The server provides find references support.
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(rename="referencesProvider")]
    pub references_provider: Option<bool>,
    
    /// The server provides document highlight support.
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(rename="documentHighlightProvider")]
    pub document_highlight_provider: Option<bool>,
    
    /// The server provides document symbol support.
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(rename="documentSymbolProvider")]
    pub document_symbol_provider: Option<bool>,
    
    /// The server provides workspace symbol support.
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(rename="workspaceSymbolProvider")]
    pub workspace_symbol_provider: Option<bool>,
    
    /// The server provides code actions.
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(rename="codeActionProvider")]
    pub code_action_provider: Option<bool>,
    
    /// The server provides code lens.
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(rename="codeLensProvider")]
    pub code_lens_provider: Option<CodeLensOptions>,
    
    /// The server provides document formatting.
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(rename="documentFormattingProvider")]
    pub document_formatting_provider: Option<bool>,
    
    /// The server provides document range formatting.
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(rename="documentRangeFormattingProvider")]
    pub document_range_formatting_provider: Option<bool>,
    
    /// The server provides document formatting on typing.
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(rename="documentOnTypeFormattingProvider")]
    pub document_on_type_formatting_provider: Option<DocumentOnTypeFormattingOptions>,
    
    /// The server provides rename support.
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(rename="renameProvider")]
    pub rename_provider: Option<bool>,
}

/**
 * The shutdown request is sent from the client to the server. It asks the server to shut down,
 * but to not exit (otherwise the response might not be delivered correctly to the client).
 * There is a separate exit notification that asks the server to exit.
 */
pub const REQUEST__Shutdown: &'static str = "shutdown";

/**
 * A notification to ask the server to exit its process. 
 * The server should exit with success code 0 if the shutdown request has been received before; 
 * otherwise with error code 1.
 */
pub const NOTIFICATION__Exit: &'static str = "exit";

/**
 * The show message notification is sent from a server to a client to ask the client to display a particular message
 * in the user interface.
 */
pub const NOTIFICATION__ShowMessage: &'static str = "window/showMessage";

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ShowMessageParams {
    /// The message type. See {@link MessageType}.
    #[serde(rename="type")]
    pub typ: MessageType,

    /// The actual message.
    pub message: String,
}

#[derive(Debug, PartialEq, Clone, Copy)]
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

impl serde::Deserialize for MessageType {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        Ok(match try!(u8::deserialize(deserializer)) {
            1 => MessageType::Error,
            2 => MessageType::Warning,
            3 => MessageType::Info,
            4 => MessageType::Log,
            _ => {
                return Err(D::Error::invalid_value("Expected a value of 1, 2, 3 or 4 to \
                                                    deserialze to MessageType"))
            }
        })
    }
}

impl serde::Serialize for MessageType {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        serializer.serialize_u8(*self as u8)
    }
}

/**
 * The show message request is sent from a server to a client to ask the client to display a particular message
 * in the user interface. In addition to the show message notification the request allows to pass actions and to
 * wait for an answer from the client.
 */
pub const REQUEST__ShowMessageRequest: &'static str = "window/showMessageRequest";


#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ShowMessageRequestParams {
    /// The message type. See {@link MessageType}
    #[serde(rename="type")]
    pub typ: MessageType,

    /// The actual message
    pub message: String,

    /// The message action items to present.
    #[serde(skip_serializing_if="Option::is_none")]
    pub actions: Option<Vec<MessageActionItem>>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct MessageActionItem {
    /// A short title like 'Retry', 'Open Log' etc.
    pub title: String,
}

/**
 * The log message notification is sent from the server to the client to ask the client to log a particular message.
 */
pub const NOTIFICATION__LogMessage: &'static str = "window/logMessage";

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct LogMessageParams {
    /// The message type. See {@link MessageType}
    #[serde(rename="type")]
    pub typ: MessageType,

    /// The actual message
    pub message: String,
}

/**
 * The telemetry notification is sent from the server to the client to ask the client to log a telemetry event.
 */
pub const NOTIFICATION__TelemetryEvent: &'static str = "telemetry/event";

/**
 * A notification sent from the client to the server to signal the change of configuration settings.
 */
pub const NOTIFICATION__WorkspaceChangeConfiguration: &'static str = "workspace/didChangeConfiguration";

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct DidChangeConfigurationParams {
    /// The actual changed settings
    pub settings: Value,
}

/**
 * The document open notification is sent from the client to the server to signal newly opened text documents.
 * The document's truth is now managed by the client and the server must not try to read the document's truth
 * using the document's uri.
 */
pub const NOTIFICATION__DidOpenTextDocument: &'static str = "textDocument/didOpen";

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct DidOpenTextDocumentParams {
    /// The document that was opened.
    #[serde(rename="textDocument")]
    pub text_document: TextDocumentItem,
}

/**
 * The document change notification is sent from the client to the server to signal changes to a text document.
 * In 2.0 the shape of the params has changed to include proper version numbers and language ids.
 */
pub const NOTIFICATION__DidChangeTextDocument: &'static str = "textDocument/didChange";

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct DidChangeTextDocumentParams {
    /// The document that did change. The version number points
    /// to the version after all provided content changes have
    /// been applied.
    #[serde(rename="textDocument")]
    pub text_document: VersionedTextDocumentIdentifier,
    /// The actual content changes.
    #[serde(rename="contentChanges")]
    pub content_changes: Vec<TextDocumentContentChangeEvent>,
}

/// An event describing a change to a text document. If range and rangeLength are omitted
/// the new text is considered to be the full content of the document.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct TextDocumentContentChangeEvent {
    /// The range of the document that changed.
    pub range: Option<Range>,

    /// The length of the range that got replaced.
    /// NOTE: seems redundant, see: https://github.com/Microsoft/language-server-protocol/issues/9
    #[serde(rename="rangeLength")]
    pub range_length: Option<u64>,

    /// The new text of the document.
    pub text: String,
}

/**
 * The document close notification is sent from the client to the server when the document got closed in the client.
 * The document's truth now exists where the document's uri points to (e.g. if the document's uri is a file uri
 * the truth now exists on disk).
 */
pub const NOTIFICATION__DidCloseTextDocument: &'static str = "textDocument/didClose";

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct DidCloseTextDocumentParams {
    /// The document that was closed.
    #[serde(rename="textDocument")]
    pub text_document: TextDocumentIdentifier,
}

/**
 * The document save notification is sent from the client to the server when the document was saved in the client.
 */
pub const NOTIFICATION__DidSaveTextDocument: &'static str = "textDocument/didSave";

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct DidSaveTextDocumentParams {
    /// The document that was saved.
    #[serde(rename="textDocument")]
    pub text_document: TextDocumentIdentifier,
}

/**
 * The watched files notification is sent from the client to the server when the client detects changes to files
 * watched by the language client.
 */
pub const NOTIFICATION__DidChangeWatchedFiles: &'static str = "workspace/didChangeWatchedFiles";

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct DidChangeWatchedFilesParams {
    /// The actual file events.
    pub changes: Vec<FileEvent>,
}

/// The file event type.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum FileChangeType {
    /// The file got created.
    Created = 1,

    /// The file got changed.
    Changed = 2,

    /// The file got deleted.
    Deleted = 3,
}

impl serde::Deserialize for FileChangeType {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        Ok(match try!(u8::deserialize(deserializer)) {
            1 => FileChangeType::Created,
            2 => FileChangeType::Changed,
            3 => FileChangeType::Deleted,
            _ => {
                return Err(D::Error::invalid_value("Expected a value of 1, 2 or 3 to deserialze \
                                                    to FileChangeType"))
            }
        })
    }
}

impl serde::Serialize for FileChangeType {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        serializer.serialize_u8(*self as u8)
    }
}

/// An event describing a file change.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct FileEvent {

    /// The file's URI.
    pub uri: String,

    /// The change type.
    #[serde(rename="type")]
    pub typ: FileChangeType,
}

/**
 * Diagnostics notification are sent from the server to the client to signal results of validation runs.
 */
pub const NOTIFICATION__PublishDiagnostics: &'static str = "textDocument/publishDiagnostics";

#[derive(Debug, PartialEq, Default, Deserialize, Serialize)]
pub struct PublishDiagnosticsParams {
    /// The URI for which diagnostic information is reported.
    pub uri: String,

    /// An array of diagnostic information items.
    pub diagnostics: Vec<Diagnostic>,
}

/**
 The Completion request is sent from the client to the server to compute completion items at a given cursor position. 
 Completion items are presented in the IntelliSense user interface. If computing full completion items is expensive, 
 servers can additionally provide a handler for the completion item resolve request ('completionItem/resolve'). 
 This request is sent when a completion item is selected in the user interface. A typically use case is for example: 
 the 'textDocument/completion' request doesn't fill in the documentation property for returned completion items 
 since it is expensive to compute. When the item is selected in the user interface then a 'completionItem/resolve' 
 request is sent with the selected completion item as a param. The returned completion item should have the 
 documentation property filled in.
*/
// result: CompletionItem[] | CompletionList FIXME
pub const REQUEST__Completion: &'static str = "textDocument/completion";

/// Represents a collection of [completion items](#CompletionItem) to be presented
/// in the editor.
#[derive(Debug, PartialEq, Default, Deserialize, Serialize)]
pub struct CompletionList {

    /// This list it not complete. Further typing should result in recomputing
    /// this list.
    #[serde(rename="isIncomplete")]
    pub is_incomplete: bool,

    /// The completion items.
    pub items: Vec<CompletionItem>,
}

#[derive(Debug, PartialEq, Default, Deserialize, Serialize)]
pub struct CompletionItem {

    /// The label of this completion item. By default
    /// also the text that is inserted when selecting
    /// this completion.
    pub label: String,

    /// The kind of this completion item. Based of the kind
    /// an icon is chosen by the editor.
    #[serde(skip_serializing_if="Option::is_none")]
    pub kind: Option<CompletionItemKind>,
    
    /// A human-readable string with additional information
    /// about this item, like type or symbol information.
    #[serde(skip_serializing_if="Option::is_none")]
    pub detail: Option<String>,

    /// A human-readable string that represents a doc-comment.
    #[serde(skip_serializing_if="Option::is_none")]
    pub documentation: Option<String>,

    /// A string that shoud be used when comparing this item
    /// with other items. When `falsy` the label is used.
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(rename="sortText")]
    pub sort_text: Option<String>,

    /// A string that should be used when filtering a set of
    /// completion items. When `falsy` the label is used.
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(rename="filterText")]
    pub filter_text: Option<String>,

    /// A string that should be inserted a document when selecting
    /// this completion. When `falsy` the label is used.
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(rename="insertText")]
    pub insert_text: Option<String>,

    /// An edit which is applied to a document when selecting
    /// this completion. When an edit is provided the value of
    /// insertText is ignored.
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(rename="textEdit")]
    pub text_edit: Option<TextEdit>,

    /// An optional array of additional text edits that are applied when
    /// selecting this completion. Edits must not overlap with the main edit
    /// nor with themselves.
    #[serde(skip_serializing_if="Option::is_none")]
    #[serde(rename="additionalTextEdits")]
    pub additional_text_edits: Option<Vec<TextEdit>>,

    /// An optional command that is executed *after* inserting this completion. *Note* that
    /// additional modifications to the current document should be described with the
    /// additionalTextEdits-property.
    #[serde(skip_serializing_if="Option::is_none")]
    pub command: Option<Command>,

    /// An data entry field that is preserved on a completion item between
    /// a completion and a completion resolve request.
    #[serde(skip_serializing_if="Option::is_none")]
    pub data: Option<Value>,
}

impl CompletionItem {
    /// Create a CompletionItem with the minimum possible info (label and detail).
    pub fn new_simple(label: String, detail: String) -> CompletionItem {
        CompletionItem {
            label : label,
            kind : None,
            detail : Some(detail),
            documentation : None,
            sort_text : None,
            filter_text : None,
            insert_text : None,
            text_edit : None,
            additional_text_edits : None,
            command : None,
            data : None,
        }
    }
}

enum_from_primitive!{
/// The kind of a completion entry.
#[derive(Debug, PartialEq, Clone, Copy)]
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
}
}

impl serde::Deserialize for CompletionItemKind {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        use enum_primitive::FromPrimitive;

        let i = try!(u8::deserialize(deserializer));
        CompletionItemKind::from_u8(i).ok_or_else(|| {
            D::Error::invalid_value("Expected a value between 1 and 18 (inclusive) to deserialize \
                                     to CompletionItemKind")
        })
    }
}

impl serde::Serialize for CompletionItemKind {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        serializer.serialize_u8(*self as u8)
    }
}

/// The request is sent from the client to the server to resolve additional information for a given completion item. 
pub const REQUEST__ResolveCompletionItem: &'static str = "completionItem/resolve";


/// The hover request is sent from the client to the server to request hover information at a given text 
/// document position.
pub const REQUEST__Hover: &'static str = "textDocument/hover";

/// The result of a hover request.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Hover {
    /// The hover's content
    pub contents: Vec<MarkedString>, /* FIXME: need to review if this is correct*/
    //contents: MarkedString | MarkedString[];

    /// An optional range is a range inside a text document 
    /// that is used to visualize a hover, e.g. by changing the background color.
    pub range: Option<Range>,
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
//type MarkedString = string | { language: string; value: string };
#[derive(Debug, PartialEq)]
pub enum MarkedString {
    String(String),
    LanguageString(LanguageString),
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct LanguageString {
    language: String,
    value: String,
}

impl MarkedString {
    
    pub fn from_markdown(markdown: String) -> MarkedString {
        MarkedString::String(markdown)
    }
    
    pub fn from_language_code(language: String, code_block: String) -> MarkedString {
        MarkedString::LanguageString(LanguageString{ language: language, value: code_block })
    }
    
}

#[test]
fn test_LanguageString() {
    test_serialization(
        &LanguageString { language : "LL".into(), value : "VV".into() } ,
        r#"{"language":"LL","value":"VV"}"#
    );
}

impl Serialize for MarkedString {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        match *self {
            MarkedString::String(ref string) => serializer.serialize_str(string),
            MarkedString::LanguageString(ref language_string) => language_string.serialize(serializer),
        }
    }
}

// See example from: https://serde.rs/string-or-struct.html
impl Deserialize for MarkedString {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error> 
        where D: serde::Deserializer
    {
        #[allow(non_camel_case_types)]
        struct MarkedString_Visitor;
        impl de::Visitor for MarkedString_Visitor {
            type Value = MarkedString;
            
            fn visit_str<E>(&mut self, value: &str) -> Result<Self::Value, E> where E: de::Error {
                Ok(MarkedString::String(value.to_string()))
            }
            
            fn visit_map<V>(&mut self, visitor: V) -> Result<Self::Value, V::Error> where V: de::MapVisitor {
                // `MapVisitorDeserializer` is a wrapper that turns a `MapVisitor`
                // into a `Deserializer`, allowing it to be used as the input to T's
                // `Deserialize` implementation. T then deserializes itself using
                // the entries from the map visitor.
                let mut mvd = de::value::MapVisitorDeserializer::new(visitor);
                let language_string = try!(LanguageString::deserialize(&mut mvd));
                Ok(MarkedString::LanguageString(language_string))
            }
        }
        
        deserializer.deserialize(MarkedString_Visitor)
    }
}


#[test]
fn test_MarkedString() {
    
    test_serialization(
        &MarkedString::from_markdown("xxx".into()),
        r#""xxx""#
    );
    
    test_serialization(
        &MarkedString::from_language_code("lang".into(), "code".into()),
        r#"{"language":"lang","value":"code"}"#
    );
}


/// The signature help request is sent from the client to the server to request signature information at 
/// a given cursor position.
pub const REQUEST__SignatureHelp: &'static str = "textDocument/signatureHelp";


/// Signature help represents the signature of something
/// callable. There can be multiple signature but only one
/// active and only one active parameter.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct SignatureHelp {
    /// One or more signatures.
    pub signatures: Vec<SignatureInformation>,

    /// The active signature.
    #[serde(rename="activeSignature")]
    pub active_signature: Option<u64>,

    /// The active parameter of the active signature.
    #[serde(rename="activeParameter")]
    pub active_parameter: Option<u64>,
}

/// Represents the signature of something callable. A signature
/// can have a label, like a function-name, a doc-comment, and
/// a set of parameters.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct SignatureInformation {
    /// The label of this signature. Will be shown in
    /// the UI.
    pub label: String,

    /// The human-readable doc-comment of this signature. Will be shown
    /// in the UI but can be omitted.
    pub documentation: Option<String>,

    /// The parameters of this signature.
    #[serde(skip_serializing_if="Option::is_none")]
    pub parameters: Option<Vec<ParameterInformation>>,
}

/// Represents a parameter of a callable-signature. A parameter can
/// have a label and a doc-comment.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ParameterInformation {
    /// The label of this signature. Will be shown in
    /// the UI.
    pub label: String,

    /// The human-readable doc-comment of this signature. Will be shown
    /// in the UI but can be omitted.
    #[serde(skip_serializing_if="Option::is_none")]
    pub documentation: Option<String>,
}

/// The goto definition request is sent from the client to the server to resolve the definition location of 
/// a symbol at a given text document position.
pub const REQUEST__GotoDefinition: &'static str = "textDocument/definition";

/// The references request is sent from the client to the server to resolve project-wide references for the 
/// symbol denoted by the given text document position.
pub const REQUEST__References: &'static str = "textDocument/references";

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ReferenceParams 
//extends TextDocumentPositionParams 
{
    
    // This field was "mixed-in" from TextDocumentPositionParams
    /// The text document.
    #[serde(rename="textDocument")]
    pub text_document: TextDocumentIdentifier,

    // This field was "mixed-in" from TextDocumentPositionParams
    /// The position inside the text document.
    pub position: Position,

    // ReferenceParams properties:
    
    pub context: ReferenceContext,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ReferenceContext {
    /// Include the declaration of the current symbol.
    #[serde(rename="includeDeclaration")]
    pub include_declaration: bool,
}

/**
 The document highlight request is sent from the client to the server to resolve a document highlights 
 for a given text document position. 
 For programming languages this usually highlights all references to the symbol scoped to this file. 
 However we kept 'textDocument/documentHighlight' and 'textDocument/references' separate requests since 
 the first one is allowed to be more fuzzy. 
 Symbol matches usually have a DocumentHighlightKind of Read or Write whereas fuzzy or textual matches 
 use Textas the kind.
*/
pub const REQUEST__DocumentHighlight: &'static str = "textDocument/documentHighlight";

/// A document highlight is a range inside a text document which deserves
/// special attention. Usually a document highlight is visualized by changing
/// the background color of its range.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct DocumentHighlight {
    /// The range this highlight applies to.
    pub range: Range,

    /// The highlight kind, default is DocumentHighlightKind.Text.
    pub kind: Option<DocumentHighlightKind>,
}

/// A document highlight kind.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DocumentHighlightKind {
    /// A textual occurrance.
    Text = 1,

    /// Read-access of a symbol, like reading a variable.
    Read = 2,

    /// Write-access of a symbol, like writing to a variable.
    Write = 3,
}

impl serde::Deserialize for DocumentHighlightKind {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        Ok(match try!(u8::deserialize(deserializer)) {
            1 => DocumentHighlightKind::Text,
            2 => DocumentHighlightKind::Read,
            3 => DocumentHighlightKind::Write,
            _ => {
                return Err(D::Error::invalid_value("Expected a value of 1, 2, or 3 to \
                                                    deserialze to DocumentHighlightKiny"))
            }
        })
    }
}

impl serde::Serialize for DocumentHighlightKind {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        serializer.serialize_u8(*self as u8)
    }
}

/**
 * The document symbol request is sent from the client to the server to list all symbols found in a given 
 * text document.
 */
pub const REQUEST__DocumentSymbols: &'static str = "textDocument/documentSymbol";

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct DocumentSymbolParams {
    /// The text document.
    #[serde(rename="textDocument")]
    pub text_document: TextDocumentIdentifier,
}

/// Represents information about programming constructs like variables, classes,
/// interfaces etc.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct SymbolInformation {
    /// The name of this symbol.
    pub name: String,

    /// The kind of this symbol.
    pub kind: SymbolKind,

    /// The location of this symbol.
    pub location: Location,

    /// The name of the symbol containing this symbol.
    #[serde(rename="containerName")]
    pub container_name: Option<String>
}

/// A symbol kind.
enum_from_primitive!{
#[derive(Debug, PartialEq, Copy, Clone)]
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
}
}

impl serde::Deserialize for SymbolKind {
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        use enum_primitive::FromPrimitive;

        let i = try!(u8::deserialize(deserializer));
        SymbolKind::from_u8(i).ok_or_else(|| {
            D::Error::invalid_value("Expected a value between 1 and 18 (inclusive) to deserialize \
                                     to SymbolKind")
        })
    }
}

impl serde::Serialize for SymbolKind {
    fn serialize<S>(&self, serializer: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
    {
        serializer.serialize_u8(*self as u8)
    }
}

/**
 * The workspace symbol request is sent from the client to the server to list project-wide symbols 
 * matching the query string.
 */
pub const REQUEST__WorkspaceSymbols: &'static str = "workspace/symbol";

/// The parameters of a Workspace Symbol Request.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct WorkspaceSymbolParams {
    /// A non-empty query string
    pub query: String,
}

/**
 * The code action request is sent from the client to the server to compute commands for a given text document
 * and range. The request is triggered when the user moves the cursor into a problem marker in the editor or 
 * presses the lightbulb associated with a marker.
 */
pub const REQUEST__CodeAction: &'static str = "textDocument/codeAction";

/// Params for the CodeActionRequest
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct CodeActionParams {
    /// The document in which the command was invoked.
    #[serde(rename="textDocument")]
    pub text_document: TextDocumentIdentifier,

    /// The range for which the command was invoked.
    pub range: Range,

    /// Context carrying additional information.
    pub context: CodeActionContext,
}

/// Contains additional diagnostic information about the context in which
/// a code action is run.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct CodeActionContext {
    /// An array of diagnostics.
    pub diagnostics: Vec<Diagnostic>,
}

/**
 * The code lens request is sent from the client to the server to compute code lenses for a given text document.
 */
pub const REQUEST__CodeLens: &'static str = "textDocument/codeLens";

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct CodeLensParams {
    /// The document to request code lens for.
    #[serde(rename="textDocument")]
    pub text_document: TextDocumentIdentifier,
}

/// A code lens represents a command that should be shown along with
/// source text, like the number of references, a way to run tests, etc.
///
/// A code lens is _unresolved_ when no command is associated to it. For performance
/// reasons the creation of a code lens and resolving should be done in two stages.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct CodeLens {
    /// The range in which this code lens is valid. Should only span a single line.
    pub range: Range,

    /// The command this code lens represents.
    pub command: Option<Command>,

    /// A data entry field that is preserved on a code lens item between
    /// a code lens and a code lens resolve request.
    pub data: Option<Value>,
}

/**
 * The code lens resolve request is sent from the client to the server to resolve the command for a 
 * given code lens item.
 */
pub const REQUEST__CodeLensResolve: &'static str = "codeLens/resolve";

/**
 * The document formatting request is sent from the server to the client to format a whole document.
 */
pub const REQUEST__Formatting: &'static str = "textDocument/formatting";

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct DocumentFormattingParams {
    /// The document to format.
    #[serde(rename="textDocument")]
    pub text_document: TextDocumentIdentifier,

    /// The format options.
    pub options: FormattingOptions,
}

/// Value-object describing what options formatting should use.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct FormattingOptions {
    /// Size of a tab in spaces.
    #[serde(rename="tabSize")]
    pub tab_size: u64,

    #[serde(rename="insertSpaces")]
    /// Prefer spaces over tabs.
    pub insert_spaces: bool,

//
//    /// Signature for further properties.
//
    //[key: string]: boolean | number | string;
    // FIXME what is this, I don't quite get it
    
}

/// The document range formatting request is sent from the client to the server to format a given range in a document.
pub const REQUEST__RangeFormatting: &'static str = "textDocument/rangeFormatting";

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct DocumentRangeFormattingParams {

    /// The document to format.
    #[serde(rename="textDocument")]
    pub text_document: TextDocumentIdentifier,


    /// The range to format
    pub range: Range,

    /// The format options
    pub options: FormattingOptions,
}

/**
 * The document on type formatting request is sent from the client to the server to format parts of 
 * the document during typing.
 */
pub const REQUEST__OnTypeFormatting: &'static str = "textDocument/onTypeFormatting";

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct DocumentOnTypeFormattingParams {
    /// The document to format.
    #[serde(rename="textDocument")]
    pub text_document: TextDocumentIdentifier,

    /// The position at which this request was sent.
    pub position: Position,

    /// The character that has been typed.
    pub ch: String,

    /// The format options.
    pub options: FormattingOptions,
}

/**
 * The rename request is sent from the client to the server to perform a workspace-wide rename of a symbol.
 */
pub const REQUEST__Rename: &'static str = "textDocument/rename";

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct RenameParams {
    /// The document to format.
    #[serde(rename="textDocument")]
    pub text_document: TextDocumentIdentifier,

    /// The position at which this request was sent.
    pub position: Position,

    /// The new name of the symbol. If the given name is not valid the
    /// request must return a [ResponseError](#ResponseError) with an
    /// appropriate message set.
    #[serde(rename="newName")]
    pub new_name: String,
}


/* -----------------  ----------------- */

#[cfg(test)]
fn test_serialization<SER>(ms: &SER, expected: &str) 
where SER : Serialize + Deserialize + PartialEq + std::fmt::Debug
{
    let json_str = serde_json::to_string(ms).unwrap();
    assert_eq!(&json_str, expected);
    let deserialized : SER = serde_json::from_str(&json_str).unwrap();
    assert_eq!(&deserialized, ms);
}