use super::*;

pub trait Request {
    type Params;
    type Result;
    const METHOD: &'static str;
}

/**

 The initialize request is sent as the first request from the client to the server.
 If the server receives request or notification before the `initialize` request it should act as follows:

 * for a request the respond should be errored with `code: -32001`. The message can be picked by the server.
 * notifications should be dropped.

*/
#[derive(Debug)]
pub enum Initialize {}

impl Request for Initialize {
    type Params = InitializeParams;
    type Result = InitializeResult;
    const METHOD: &'static str = "initialize";
}

/**
 * The shutdown request is sent from the client to the server. It asks the server to shut down,
 * but to not exit (otherwise the response might not be delivered correctly to the client).
 * There is a separate exit notification that asks the server to exit.
 */
#[derive(Debug)]
pub enum Shutdown {}

impl Request for Shutdown {
    type Params = ();
    type Result = ();
    const METHOD: &'static str = "shutdown";
}

/**
 * The show message request is sent from a server to a client to ask the client to display a particular message
 * in the user interface. In addition to the show message notification the request allows to pass actions and to
 * wait for an answer from the client.
 */
#[derive(Debug)]
pub enum ShowMessageRequest {}

impl Request for ShowMessageRequest {
    type Params = ShowMessageRequestParams;
    type Result = Option<MessageActionItem>;
    const METHOD: &'static str = "window/showMessageRequest";
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
#[derive(Debug)]
pub enum Completion {}

impl Request for Completion {
    type Params = CompletionParams;
    type Result = CompletionResponse;
    const METHOD: &'static str = "textDocument/completion";
}

/// The request is sent from the client to the server to resolve additional information for a given completion item.
#[derive(Debug)]
pub enum ResolveCompletionItem {}

impl Request for ResolveCompletionItem {
    type Params = CompletionItem;
    type Result = CompletionItem;
    const METHOD: &'static str = "completionItem/resolve";
}

/// The hover request is sent from the client to the server to request hover information at a given text
/// document position.
#[derive(Debug)]
pub enum HoverRequest {}

impl Request for HoverRequest {
    type Params = TextDocumentPositionParams;
    type Result = Option<Hover>;
    const METHOD: &'static str = "textDocument/hover";
}

/// The signature help request is sent from the client to the server to request signature information at
/// a given cursor position.
#[derive(Debug)]
pub enum SignatureHelpRequest {}

impl Request for SignatureHelpRequest {
    type Params = TextDocumentPositionParams;
    type Result = Option<SignatureHelp>;
    const METHOD: &'static str = "textDocument/signatureHelp";
}

/// The goto definition request is sent from the client to the server to resolve the definition location of
/// a symbol at a given text document position.
#[derive(Debug)]
pub enum GotoDefinition {}

impl Request for GotoDefinition {
    type Params = TextDocumentPositionParams;
    type Result = Option<GotoDefinitionResponse>;
    const METHOD: &'static str = "textDocument/definition";
}

/**
 * GotoDefinition response can be single location or multiple ones.
 */
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GotoDefinitionResponse {
    Scalar(Location),
    Array(Vec<Location>),
}

/// The references request is sent from the client to the server to resolve project-wide references for the
/// symbol denoted by the given text document position.
pub enum References {}

impl Request for References {
    type Params = ReferenceParams;
    type Result = Option<Vec<Location>>;
    const METHOD: &'static str = "textDocument/references";
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
#[derive(Debug)]
pub enum DocumentHighlightRequest {}

impl Request for DocumentHighlightRequest {
    type Params = TextDocumentPositionParams;
    type Result = Option<Vec<DocumentHighlight>>;
    const METHOD: &'static str = "textDocument/documentHighlight";
}

/**
 * The document symbol request is sent from the client to the server to list all symbols found in a given
 * text document.
 */
#[derive(Debug)]
pub enum DocumentSymbol {}

impl Request for DocumentSymbol {
    type Params = DocumentSymbolParams;
    type Result = Option<Vec<SymbolInformation>>;
    const METHOD: &'static str = "textDocument/documentSymbol";
}

/**
 * The workspace symbol request is sent from the client to the server to list project-wide symbols
 * matching the query string.
 */
#[derive(Debug)]
pub enum WorkspaceSymbol {}

impl Request for WorkspaceSymbol {
    type Params = WorkspaceSymbolParams;
    type Result = Option<Vec<SymbolInformation>>;
    const METHOD: &'static str = "workspace/symbol";
}

/// The workspace/executeCommand request is sent from the client to the server to trigger command execution on the server. In most cases the server creates a WorkspaceEdit structure and applies the changes to the workspace using the request workspace/applyEdit which is sent from the server to the client.
#[derive(Debug)]
pub enum ExecuteCommand {}

impl Request for ExecuteCommand {
    type Params = ExecuteCommandParams;
    type Result = Option<Value>;
    const METHOD: &'static str = "workspace/executeCommand";
}

/// The workspace/applyEdit request is sent from the server to the client to modify resource on the
/// client side.
#[derive(Debug)]
pub enum ApplyWorkspaceEdit {}

impl Request for ApplyWorkspaceEdit {
    type Params = ApplyWorkspaceEditParams;
    type Result = ApplyWorkspaceEditResponse;
    const METHOD: &'static str = "workspace/applyEdit";
}

/**
 * The code action request is sent from the client to the server to compute commands for a given text document
 * and range. The request is triggered when the user moves the cursor into a problem marker in the editor or
 * presses the lightbulb associated with a marker.
 */
#[derive(Debug)]
pub enum CodeActionRequest {}

impl Request for CodeActionRequest {
    type Params = CodeActionParams;
    type Result = Option<Vec<Command>>;
    const METHOD: &'static str = "textDocument/codeAction";
}

/**
 * The code lens request is sent from the client to the server to compute code lenses for a given text document.
 */
#[derive(Debug)]
pub enum CodeLensRequest {}

impl Request for CodeLensRequest {
    type Params = CodeLensParams;
    type Result = Option<Vec<CodeLens>>;
    const METHOD: &'static str = "textDocument/codeLens";
}

/**
 * The code lens resolve request is sent from the client to the server to resolve the command for a
 * given code lens item.
 */
#[derive(Debug)]
pub enum CodeLensResolve {}

impl Request for CodeLensResolve {
    type Params = CodeLens;
    type Result = CodeLens;
    const METHOD: &'static str = "codeLens/resolve";
}

/// The document links request is sent from the client to the server to request the location of links in a document.
#[derive(Debug)]
pub enum DocumentLinkRequest {}

impl Request for DocumentLinkRequest {
    type Params = DocumentLinkParams;
    type Result = Option<Vec<DocumentLink>>;
    const METHOD: &'static str = "textDocument/documentLink";
}

/**

 The document link resolve request is sent from the client to the server to resolve the target of
 a given document link.

*/
#[derive(Debug)]
pub enum DocumentLinkResolve {}

impl Request for DocumentLinkResolve {
    type Params = DocumentLink;
    type Result = DocumentLink;
    const METHOD: &'static str = "documentLink/resolve";
}

/**
 * The document formatting request is sent from the server to the client to format a whole document.
 */
#[derive(Debug)]
pub enum Formatting {}

impl Request for Formatting {
    type Params = DocumentFormattingParams;
    type Result = Option<Vec<TextEdit>>;
    const METHOD: &'static str = "textDocument/formatting";
}

/// The document range formatting request is sent from the client to the server to format a given range in a document.
#[derive(Debug)]
pub enum RangeFormatting {}

impl Request for RangeFormatting {
    type Params = DocumentRangeFormattingParams;
    type Result = Option<Vec<TextEdit>>;
    const METHOD: &'static str = "textDocument/rangeFormatting";
}

/**
 * The document on type formatting request is sent from the client to the server to format parts of
 * the document during typing.
 */
#[derive(Debug)]
pub enum OnTypeFormatting {}

impl Request for OnTypeFormatting {
    type Params = DocumentOnTypeFormattingParams;
    type Result = Option<Vec<TextEdit>>;
    const METHOD: &'static str = "textDocument/onTypeFormatting";
}

/**
 * The rename request is sent from the client to the server to perform a workspace-wide rename of a symbol.
 */
#[derive(Debug)]
pub enum Rename {}

impl Request for Rename {
    type Params = RenameParams;
    type Result = Option<WorkspaceEdit>;
    const METHOD: &'static str = "textDocument/rename";
}
