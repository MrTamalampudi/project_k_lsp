use parking_lot::RwLock;
use parser::parser::Parser;
use parser::token::Token;
use parser::{source_code_to_lexer, CompilationContext};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use tokio_util::sync::CancellationToken;
use tower_lsp::jsonrpc::Result;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

mod completion;
mod semantic_tokens;

pub struct Backend {
    client: Client,
    token_map: RwLock<HashMap<Uri, Vec<Token>>>,
}

impl Backend {
    async fn compile_and_publish_diagnostics(&self, uri: Uri, version: i32, text: String) {
        //clear diagnostics if we already published any diagnostics
        self.client
            .publish_diagnostics(uri.clone(), Vec::new(), Some(version))
            .await;
        let mut ctx = CompilationContext::new(PathBuf::from(uri.path().as_str()));
        let mut lexer = source_code_to_lexer(text.clone(), &mut ctx);
        let mut lock = self.token_map.write();
        lock.insert(uri, lexer.tokens.clone());
        Parser::new(&mut lexer, &mut ctx).parse();
        let mut diagnostic_map: HashMap<Uri, Vec<Diagnostic>> = HashMap::new();
        //converting compiler errors to lsp errors and
        //adding diagnostic_map grouped by file URI
        self.client
            .show_message(MessageType::INFO, format!("{:#?}", ctx.errors.errors))
            .await;
        for error in ctx.errors.errors.into_iter() {
            diagnostic_map
                .entry(Uri::from_str(&error.source_path).expect(""))
                .or_insert_with(Vec::new)
                .push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: (error.span.start.line - 1) as u32,
                            character: (error.span.start.column - 1) as u32,
                        },
                        end: Position {
                            line: (error.span.end.line - 1) as u32,
                            character: (error.span.end.column - 1) as u32,
                        },
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
                definition_provider: Some(OneOf::Right(DefinitionOptions {
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                })),
                code_lens_provider: Some(CodeLensOptions {
                    resolve_provider: Some(true),
                }),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), "#".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
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

    async fn code_lens(
        &self,
        params: CodeLensParams,
        token: CancellationToken,
    ) -> Result<Option<Vec<CodeLens>>> {
        let code_lens = CodeLens {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 10,
                },
            },
            command: Some(Command {
                title: String::from("check"),
                command: String::from("echo \"hello\""),
                arguments: None,
            }),
            data: None,
        };

        Ok(Some(vec![code_lens.clone(), code_lens.clone()]))
    }

    // async fn completion(
    //     &self,
    //     params: CompletionParams,
    //     token: CancellationToken,
    // ) -> Result<Option<CompletionResponse>> {
    //     let tokens = self.token_map.read();
    //     let tokens = tokens
    //         .get(&params.text_document_position.text_document.uri)
    //         .unwrap();
    //     // self.client
    //     //     .show_message(MessageType::INFO, format!("tokendds {:#?}", tokens))
    //     //     .await;
    //     let t = IntelliSense::new(tokens, &params.text_document_position.position).complete();
    //     Ok(Some(CompletionResponse::Array(t)))
    //     // self.client
    //     //     .show_message(MessageType::INFO, format!("tokens {:#?}", token))
    //     //     .await;
    //     //Ok(None)
    // }
}

#[tokio::main]
async fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

    let (service, socket, pending) = LspService::new(|client| Backend {
        client,
        token_map: RwLock::new(HashMap::new()),
    });
    Server::new(stdin, stdout, socket, pending)
        .serve(service)
        .await;
}
