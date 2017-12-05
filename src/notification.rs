use super::*;

pub trait Notification {
    type Params;
    const METHOD: &'static str;
}

#[macro_export]
macro_rules! lsp_notification {
    ("$/cancelRequest") => { $crate::notification::Cancel };
    ("initialized") => { $crate::notification::Initialized };
    ("exit") => { $crate::notification::Exit };

    ("window/showMessage") => { $crate::notification::ShowMessage };
    ("window/logMessage") => { $crate::notification::LogMessage };

    ("telemetry/event") => { $crate::notification::Event };

    ("client/registerCapability") => { $crate::notification::RegisterCapability };
    ("client/unregisterCapability") => { $crate::notification::UnregisterCapability };

    ("textDocument/didOpen") => { $crate::notification::DidOpenTextDocument };
    ("textDocument/didChange") => { $crate::notification::DidChangeTextDocument };
    ("textDocument/willSave") => { $crate::notification::WillSaveTextDocument };
    ("textDocument/didSave") => { $crate::notification::DidSaveTextDocument };
    ("textDocument/didClose") => { $crate::notification::DidCloseTextDocument };
    ("textDocument/publishDiagnostics") => { $crate::notification::PublishDiagnostics };

    ("workspace/didChangeConfiguration") => { $crate::notification::DidChangeConfiguration };
    ("workspace/didChangeWatchedFiles") => { $crate::notification::DidChangeWatchedFiles };
}


/// The base protocol now offers support for request cancellation. To cancel a request,
/// a notification message with the following properties is sent:
///
/// A request that got canceled still needs to return from the server and send a response back.
/// It can not be left open / hanging. This is in line with the JSON RPC protocol that requires
/// that every request sends a response back. In addition it allows for returning partial results on cancel.
#[derive(Debug)]
pub enum Cancel {}

impl Notification for Cancel {
    type Params = CancelParams;
    const METHOD: &'static str = "$/cancelRequest";
}

/// The initialized notification is sent from the client to the server after the client received
/// the result of the initialize request but before the client is sending any other request or
/// notification to the server. The server can use the initialized notification for example to
/// dynamically register capabilities.
#[derive(Debug)]
pub enum Initialized {}

impl Notification for Initialized {
    type Params = ();
    const METHOD: &'static str = "initialized";
}

/**
 * A notification to ask the server to exit its process.
 * The server should exit with success code 0 if the shutdown request has been received before;
 * otherwise with error code 1.
 */
#[derive(Debug)]
pub enum Exit {}

impl Notification for Exit {
    type Params = ();
    const METHOD: &'static str = "exit";
}

/**
 * The show message notification is sent from a server to a client to ask the client to display a particular message
 * in the user interface.
 */
#[derive(Debug)]
pub enum ShowMessage {}

impl Notification for ShowMessage {
    type Params = ShowMessageParams;
    const METHOD: &'static str = "window/showMessage";
}

/**
 * The log message notification is sent from the server to the client to ask the client to log a particular message.
 */
#[derive(Debug)]
pub enum LogMessage {}

impl Notification for LogMessage {
    type Params = LogMessageParams;
    const METHOD: &'static str = "window/logMessage";
}

/**
 * The telemetry notification is sent from the server to the client to ask the client to log a telemetry event.
 */
#[derive(Debug)]
pub enum TelemetryEvent {}

impl Notification for TelemetryEvent {
    type Params = ();
    const METHOD: &'static str = "telemetry/event";
}

/**
 * The client/registerCapability request is sent from the server to the client to register for a new capability on the client side. Not all clients need to support dynamic capability registration. A client opts in via the ClientCapabilities.GenericCapability property.
 */
#[derive(Debug)]
pub enum RegisterCapability {}

impl Notification for RegisterCapability {
    type Params = RegistrationParams;
    const METHOD: &'static str = "client/registerCapability";
}


/// The client/unregisterCapability request is sent from the server to the client to unregister a
/// previously register capability.
#[derive(Debug)]
pub enum UnregisterCapability {}

impl Notification for UnregisterCapability {
    type Params = UnregistrationParams;
    const METHOD: &'static str = "client/unregisterCapability";
}

/**
 * A notification sent from the client to the server to signal the change of configuration settings.
 */
#[derive(Debug)]
pub enum DidChangeConfiguration {}

impl Notification for DidChangeConfiguration {
    type Params = DidChangeConfigurationParams;
    const METHOD: &'static str = "workspace/didChangeConfiguration";
}

/**
 * The document open notification is sent from the client to the server to signal newly opened text documents.
 * The document's truth is now managed by the client and the server must not try to read the document's truth
 * using the document's uri.
 */
#[derive(Debug)]
pub enum DidOpenTextDocument {}

impl Notification for DidOpenTextDocument {
    type Params = DidOpenTextDocumentParams;
    const METHOD: &'static str = "textDocument/didOpen";
}

/**
 * The document change notification is sent from the client to the server to signal changes to a text document.
 * In 2.0 the shape of the params has changed to include proper version numbers and language ids.
 */
#[derive(Debug)]
pub enum DidChangeTextDocument {}

impl Notification for DidChangeTextDocument {
    type Params = DidChangeTextDocumentParams;
    const METHOD: &'static str = "textDocument/didChange";
}

/// The document will save notification is sent from the client to the server before the document
/// is actually saved.
#[derive(Debug)]
pub enum WillSave {}

impl Notification for WillSave {
    type Params = ();
    const METHOD: &'static str = "textDocument/willSave";
}

/// The document will save request is sent from the client to the server before the document is
/// actually saved. The request can return an array of TextEdits which will be applied to the text
/// document before it is saved. Please note that clients might drop results if computing the text
/// edits took too long or if a server constantly fails on this request. This is done to keep the
/// save fast and reliable.
#[derive(Debug)]
pub enum WillSaveWaitUntil {}

impl Notification for WillSaveWaitUntil {
    type Params = ();
    const METHOD: &'static str = "textDocument/willSaveWaitUntil";
}


/**
 * The document close notification is sent from the client to the server when the document got closed in the client.
 * The document's truth now exists where the document's uri points to (e.g. if the document's uri is a file uri
 * the truth now exists on disk).
 */
#[derive(Debug)]
pub enum DidCloseTextDocument {}

impl Notification for DidCloseTextDocument {
    type Params = DidCloseTextDocumentParams;
    const METHOD: &'static str = "textDocument/didClose";
}

/**
 * The document save notification is sent from the client to the server when the document was saved in the client.
 */
#[derive(Debug)]
pub enum DidSaveTextDocument {}

impl Notification for DidSaveTextDocument {
    type Params = DidSaveTextDocumentParams;
    const METHOD: &'static str = "textDocument/didSave";
}

/**
 * The watched files notification is sent from the client to the server when the client detects changes to files
 * watched by the language client.
 */
#[derive(Debug)]
pub enum DidChangeWatchedFiles {}

impl Notification for DidChangeWatchedFiles {
    type Params = DidChangeWatchedFilesParams;
    const METHOD: &'static str = "workspace/didChangeWatchedFiles";
}

/**
 * Diagnostics notification are sent from the server to the client to signal results of validation runs.
 */
#[derive(Debug)]
pub enum PublishDiagnostics {}

impl Notification for PublishDiagnostics {
    type Params = PublishDiagnosticsParams;
    const METHOD: &'static str = "textDocument/publishDiagnostics";
}
