use parser::parser::Parser;
use parser::{source_code_to_lexer, CompilationContext};
use std::collections::HashMap;
use std::env;
use std::ops::Range as R;
use std::path::PathBuf;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

pub struct Backend {
    client: Client,
}

impl Backend {
    async fn compile_and_publish_diagnostics(&self, uri: Uri, version: i32, text: String) {
        //clear diagnostics if we already published any diagnostics
        self.client
            .publish_diagnostics(uri.clone(), Vec::new(), Some(version))
            .await;
        let mut ctx = CompilationContext::new(PathBuf::from(uri.path().as_str()));
        let lexer = source_code_to_lexer(text.as_str());
        Parser::new(lexer, &mut ctx).parse();
        let mut diagnostic_map: HashMap<Uri, Vec<Diagnostic>> = HashMap::new();
        //converting compiler errors to lsp errors and
        //adding diagnostic_map grouped by file URI
        self.client
            .show_message(MessageType::INFO, format!("{:#?}", ctx.errors.errors))
            .await;
        for error in ctx.errors.errors.into_iter() {
            diagnostic_map
                .entry(uri.clone())
                .or_insert_with(Vec::new)
                .push(Diagnostic {
                    range: Range {
                        start: start_to_lsp_position(error.span.clone(), text.as_str()),
                        end: end_to_lsp_position(error.span, text.as_str()),
                    },
                    message: error.message.clone(),
                    ..Default::default()
                });
        }

        //publishing by uri
        for (uri, diagnostics) in diagnostic_map.into_iter() {
            self.client
                .publish_diagnostics(uri, diagnostics, Some(version))
                .await;
        }
    }
}

#[tower_lsp::async_trait(?Send)]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> LspResult<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }
    async fn shutdown(&self) -> LspResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.compile_and_publish_diagnostics(
            params.text_document.uri.clone(),
            params.text_document.version,
            params.text_document.text,
        )
        .await;
    }
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.compile_and_publish_diagnostics(
            params.text_document.uri.clone(),
            params.text_document.version,
            params.content_changes.get(0).unwrap().text.clone(),
        )
        .await;
        self.client
            .show_message(
                MessageType::INFO,
                params.content_changes.get(0).unwrap().text.clone(),
            )
            .await
    }
}

#[tokio::main]
async fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

    let (service, socket, pending) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket, pending)
        .serve(service)
        .await;
}

fn start_to_lsp_position(range: R<usize>, code: &str) -> Position {
    // Calculate the line and column of the error from the source range.
    // Lines are zero indexed in vscode so we need to subtract 1.
    let mut line = code.get(..range.start).unwrap_or_default().lines().count();
    if line > 0 {
        line = line.saturating_sub(1);
    }
    let column = code[..range.start]
        .lines()
        .last()
        .map(|l| l.len())
        .unwrap_or_default();

    Position {
        line: line as u32,
        character: column as u32,
    }
}

fn end_to_lsp_position(range: R<usize>, code: &str) -> Position {
    let lines = code.get(..range.end).unwrap_or_default().lines();
    if lines.clone().count() == 0 {
        return Position {
            line: 0,
            character: 0,
        };
    }

    // Calculate the line and column of the error from the source range.
    // Lines are zero indexed in vscode so we need to subtract 1.
    let line = lines.clone().count() - 1;
    let column = lines.last().map(|l| l.len()).unwrap_or_default();

    Position {
        line: line as u32,
        character: column as u32,
    }
}
