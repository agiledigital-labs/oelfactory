use std::collections::HashMap;

use dashmap::DashMap;
use oel_language_server::oel::{parse, ImCompleteSemanticToken, Span};
use oel_language_server::semantic_token::{semantic_token_from_ast, LEGEND_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::notification::Notification;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
#[derive(Debug)]
struct Backend {
    client: Client,
    ast_map: DashMap<String, HashMap<String, Span>>,
    semantic_token_map: DashMap<String, Vec<ImCompleteSemanticToken>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        // //println!("initalizing");
        Ok(InitializeResult {
            server_info: None,
            offset_encoding: None,
            capabilities: ServerCapabilities {
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                        SemanticTokensRegistrationOptions {
                            text_document_registration_options: {
                                TextDocumentRegistrationOptions {
                                    document_selector: Some(vec![DocumentFilter {
                                        language: Some("oel".to_string()),
                                        scheme: Some("file".to_string()),
                                        pattern: None,
                                    }]),
                                }
                            },
                            semantic_tokens_options: SemanticTokensOptions {
                                work_done_progress_options: WorkDoneProgressOptions::default(),
                                legend: SemanticTokensLegend {
                                    token_types: LEGEND_TYPE.into(),
                                    token_modifiers: vec![],
                                },
                                range: Some(true),
                                full: Some(SemanticTokensFullOptions::Bool(true)),
                            },
                            static_registration_options: StaticRegistrationOptions::default(),
                        },
                    ),
                ),
                ..ServerCapabilities::default()
            },
        })
    }
    async fn initialized(&self, _: InitializedParams) {
        //println!("init");
        self.client
            .log_message(MessageType::INFO, "initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        //println!("shutdown");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        //println!("did open");
        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: params.text_document.text,
            version: params.text_document.version,
        })
        .await
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        //println!("did change");
        self.client
            .log_message(MessageType::INFO, "did change")
            .await;
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: std::mem::take(&mut params.content_changes[0].text),
            version: params.text_document.version,
        })
        .await
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {
        //println!("did save");
        self.client
            .log_message(MessageType::INFO, "file saved!")
            .await;
    }
    async fn did_close(&self, _: DidCloseTextDocumentParams) {
        //println!("did close");
        self.client
            .log_message(MessageType::INFO, "file closed!")
            .await;
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        //println!("goto_definition");
        // let definition = async {
        //     let uri = params.text_document_position_params.text_document.uri;
        //     let ast = self.ast_map.get(uri.as_str())?;

        //     let position = params.text_document_position_params.position;
        //     let char = rope.try_line_to_char(position.line as usize).ok()?;
        //     let offset = char + position.character as usize;
        //     // self.client.log_message(MessageType::INFO, &format!("{:#?}, {}", ast.value(), offset)).await;
        //     let span = get_definition(&ast, position);
        //     self.client
        //         .log_message(MessageType::INFO, &format!("{:?}, ", span))
        //         .await;
        //     span.and_then(|range| Some(GotoDefinitionResponse::Scalar(Location::new(uri, range))))
        // }
        // .await;
        // Ok(definition)
        Ok(None)
    }
    // async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
    //     let reference_list = || -> Option<Vec<Location>> {
    //         let uri = params.text_document_position.text_document.uri;
    //         let ast = self.ast_map.get(&uri.to_string())?;
    //         let rope = self.document_map.get(&uri.to_string())?;

    //         let position = params.text_document_position.position;
    //         let char = rope.try_line_to_char(position.line as usize).ok()?;
    //         let offset = char + position.character as usize;
    //         let reference_list = get_reference(&ast, offset, false);
    //         let ret = reference_list
    //             .into_iter()
    //             .filter_map(|(_, range)| {
    //                 let start_position = offset_to_position(range.start, &rope)?;
    //                 let end_position = offset_to_position(range.end, &rope)?;

    //                 let range = Range::new(start_position, end_position);

    //                 Some(Location::new(uri.clone(), range))
    //             })
    //             .collect::<Vec<_>>();
    //         Some(ret)
    //     }();
    //     Ok(reference_list)
    // }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri.to_string();
        self.client
            .log_message(MessageType::LOG, "semantic_token_full")
            .await;

        let toks = self
            .semantic_token_map
            .get(&uri)
            .iter()
            .map(|tok| format!("tok: {:#?}", tok))
            .collect::<Vec<_>>()
            .join(", ");
        self.client
            .log_message(MessageType::LOG, format!("t: {}", toks))
            .await;

        let semantic_tokens = || -> Option<Vec<SemanticToken>> {
            let mut im_complete_tokens = self.semantic_token_map.get_mut(&uri)?;
            let ast = self.ast_map.get(&uri)?;
            let extends_tokens = semantic_token_from_ast(&ast);
            im_complete_tokens.extend(extends_tokens);
            im_complete_tokens.sort_by(|a, b| a.start.cmp(&b.start));
            let mut pre_line = 0;
            let mut pre_start = 0;

            let semantic_tokens = im_complete_tokens
                .iter()
                .filter_map(|token| {
                    // TODO: CAST?
                    let line = token.start.row as u32;
                    let start = token.start.column as u32;
                    let delta_line = line - pre_line;
                    let delta_start = if delta_line == 0 {
                        start - pre_start
                    } else {
                        start
                    };
                    let ret = Some(SemanticToken {
                        delta_line,
                        delta_start,
                        length: (token.end.column - token.start.column) as u32,
                        token_type: token.token_type as u32,
                        token_modifiers_bitset: 0,
                    });
                    pre_line = line;
                    pre_start = start;
                    ret
                })
                .collect::<Vec<_>>();
            Some(semantic_tokens)
        }();
        if let Some(semantic_token) = semantic_tokens {
            return Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data: semantic_token,
            })));
        }
        Ok(None)
    }

    async fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> Result<Option<SemanticTokensRangeResult>> {
        //println!("semantic_tokens_range");
        // let uri = params.text_document.uri.to_string();
        // let semantic_tokens = || -> Option<Vec<SemanticToken>> {
        //     let im_complete_tokens = self.semantic_token_map.get(&uri)?;
        //     let rope = self.document_map.get(&uri)?;
        //     let mut pre_line = 0;
        //     let mut pre_start = 0;
        //     let semantic_tokens = im_complete_tokens
        //         .iter()
        //         .filter_map(|token| {
        //             let line = rope.try_byte_to_line(token.start as usize).ok()? as u32;
        //             let first = rope.try_line_to_char(line as usize).ok()? as u32;
        //             let start = rope.try_byte_to_char(token.start as usize).ok()? as u32 - first;
        //             let ret = Some(SemanticToken {
        //                 delta_line: line - pre_line,
        //                 delta_start: if start >= pre_start {
        //                     start - pre_start
        //                 } else {
        //                     start
        //                 },
        //                 length: token.length as u32,
        //                 token_type: token.token_type as u32,
        //                 token_modifiers_bitset: 0,
        //             });
        //             pre_line = line;
        //             pre_start = start;
        //             ret
        //         })
        //         .collect::<Vec<_>>();
        //     Some(semantic_tokens)
        // }();
        // if let Some(semantic_token) = semantic_tokens {
        //     return Ok(Some(SemanticTokensRangeResult::Tokens(SemanticTokens {
        //         result_id: None,
        //         data: semantic_token,
        //     })));
        // }
        Ok(None)
    }

    async fn inlay_hint(
        &self,
        params: tower_lsp::lsp_types::InlayHintParams,
    ) -> Result<Option<Vec<InlayHint>>> {
        //println!("inlay_hint");
        // self.client
        //     .log_message(MessageType::INFO, "inlay hint")
        //     .await;
        // let uri = &params.text_document.uri;
        // let mut hashmap = HashMap::new();
        // if let Some(ast) = self.ast_map.get(uri.as_str()) {
        //     ast.iter().for_each(|(_, v)| {
        //         type_inference(&v.body, &mut hashmap);
        //     });
        // }

        // let document = match self.document_map.get(uri.as_str()) {
        //     Some(rope) => rope,
        //     None => return Ok(None),
        // };
        // let inlay_hint_list = hashmap
        //     .into_iter()
        //     .map(|(k, v)| {
        //         (
        //             k.start,
        //             k.end,
        //             match v {
        //                 oel_language_server::chumsky::Value::Null => "null".to_string(),
        //                 oel_language_server::chumsky::Value::Bool(_) => "bool".to_string(),
        //                 oel_language_server::chumsky::Value::Num(_) => "number".to_string(),
        //                 oel_language_server::chumsky::Value::Str(_) => "string".to_string(),
        //                 oel_language_server::chumsky::Value::List(_) => "[]".to_string(),
        //                 oel_language_server::chumsky::Value::Func(_) => v.to_string(),
        //             },
        //         )
        //     })
        //     .filter_map(|item| {
        //         // let start_position = offset_to_position(item.0, document)?;
        //         let end_position = offset_to_position(item.1, &document)?;
        //         let inlay_hint = InlayHint {
        //             text_edits: None,
        //             tooltip: None,
        //             kind: Some(InlayHintKind::TYPE),
        //             padding_left: None,
        //             padding_right: None,
        //             data: None,
        //             position: end_position,
        //             label: InlayHintLabel::LabelParts(vec![InlayHintLabelPart {
        //                 value: item.2,
        //                 tooltip: None,
        //                 location: Some(Location {
        //                     uri: params.text_document.uri.clone(),
        //                     range: Range {
        //                         start: Position::new(0, 4),
        //                         end: Position::new(0, 5),
        //                     },
        //                 }),
        //                 command: None,
        //             }]),
        //         };
        //         Some(inlay_hint)
        //     })
        //     .collect::<Vec<_>>();

        // Ok(Some(inlay_hint_list))
        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        //println!("completion");
        // let uri = params.text_document_position.text_document.uri;
        // let position = params.text_document_position.position;
        // let completions = || -> Option<Vec<CompletionItem>> {
        //     let ast = self.ast_map.get(&uri.to_string())?;
        //     let offset = char + position.character as usize;
        //     let completions = completion(&ast, offset);
        //     let mut ret = Vec::with_capacity(completions.len());
        //     for (_, item) in completions {
        //         match item {
        //             oel_language_server::completion::ImCompleteCompletionItem::Variable(var) => {
        //                 ret.push(CompletionItem {
        //                     label: var.clone(),
        //                     insert_text: Some(var.clone()),
        //                     kind: Some(CompletionItemKind::VARIABLE),
        //                     detail: Some(var),
        //                     ..Default::default()
        //                 });
        //             }
        //             oel_language_server::completion::ImCompleteCompletionItem::Function(
        //                 name,
        //                 args,
        //             ) => {
        //                 ret.push(CompletionItem {
        //                     label: name.clone(),
        //                     kind: Some(CompletionItemKind::FUNCTION),
        //                     detail: Some(name.clone()),
        //                     insert_text: Some(format!(
        //                         "{}({})",
        //                         name,
        //                         args.iter()
        //                             .enumerate()
        //                             .map(|(index, item)| { format!("${{{}:{}}}", index + 1, item) })
        //                             .collect::<Vec<_>>()
        //                             .join(",")
        //                     )),
        //                     insert_text_format: Some(InsertTextFormat::SNIPPET),
        //                     ..Default::default()
        //                 });
        //             }
        //         }
        //     }
        //     Some(ret)
        // }();
        // Ok(completions.map(CompletionResponse::Array))
        Ok(None)
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        //println!("rename");
        // let workspace_edit = || -> Option<WorkspaceEdit> {
        //     let uri = params.text_document_position.text_document.uri;
        //     let ast = self.ast_map.get(&uri.to_string())?;
        //     let rope = self.document_map.get(&uri.to_string())?;

        //     let position = params.text_document_position.position;
        //     let char = rope.try_line_to_char(position.line as usize).ok()?;
        //     let offset = char + position.character as usize;
        //     let reference_list = get_reference(&ast, position, true);
        //     let new_name = params.new_name;
        //     if reference_list.len() > 0 {
        //         let edit_list = reference_list
        //             .into_iter()
        //             .filter_map(|(_, range)| {
        //                 // let start_position = offset_to_position(range.start, &rope)?;
        //                 // let end_position = offset_to_position(range.end, &rope)?;
        //                 Some(TextEdit::new(range, new_name.clone()))
        //             })
        //             .collect::<Vec<_>>();
        //         let mut map = HashMap::new();
        //         map.insert(uri, edit_list);
        //         let workspace_edit = WorkspaceEdit::new(map);
        //         Some(workspace_edit)
        //     } else {
        //         None
        //     }
        // }();
        // Ok(workspace_edit)
        Ok(None)
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        //println!("did_change_configuration");
        self.client
            .log_message(MessageType::INFO, "configuration changed!")
            .await;
    }

    async fn did_change_workspace_folders(&self, _: DidChangeWorkspaceFoldersParams) {
        //println!("did_change_workspace_folders");
        self.client
            .log_message(MessageType::INFO, "workspace folders changed!")
            .await;
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        //println!("did_change_watched_files");
        self.client
            .log_message(MessageType::INFO, "watched files have changed!")
            .await;
    }

    async fn execute_command(&self, _: ExecuteCommandParams) -> Result<Option<Value>> {
        //println!("execute_command");
        self.client
            .log_message(MessageType::INFO, "command executed!")
            .await;

        match self.client.apply_edit(WorkspaceEdit::default()).await {
            Ok(res) if res.applied => self.client.log_message(MessageType::INFO, "applied").await,
            Ok(_) => self.client.log_message(MessageType::INFO, "rejected").await,
            Err(err) => self.client.log_message(MessageType::ERROR, err).await,
        }

        Ok(None)
    }
}
#[derive(Debug, Deserialize, Serialize)]
struct InlayHintParams {
    path: String,
}

enum CustomNotification {}
impl Notification for CustomNotification {
    type Params = InlayHintParams;
    const METHOD: &'static str = "custom/notification";
}
struct TextDocumentItem {
    uri: Url,
    text: String,
    version: i32,
}
impl Backend {
    async fn on_change(&self, params: TextDocumentItem) {
        //println!("on_change");
        self.client
            .log_message(MessageType::INFO, format!("parsing ast: {}", params.uri))
            .await;
        let (ast, errors, semantic_tokens) = parse(&params.text);
        self.client
            .log_message(MessageType::INFO, format!("errors: {:?}", errors))
            .await;
        self.client
            .log_message(
                MessageType::INFO,
                format!("semantic_tokens: {:?}", semantic_tokens),
            )
            .await;
        let diagnostics = errors
            .into_iter()
            .filter_map(|item: oel_language_server::oel::ErrorToken| {
                let diagnostic = || -> Option<Diagnostic> {
                    // TODO: OMG
                    Some(Diagnostic::new_simple(
                        Range::new(
                            Position {
                                line: item.start.row.try_into().unwrap(),
                                character: item.start.column.try_into().unwrap(),
                            },
                            Position {
                                line: item.end.row.try_into().unwrap(),
                                character: item.end.row.try_into().unwrap(),
                            },
                        ),
                        item.message,
                    ))
                }();
                diagnostic
            })
            .collect::<Vec<_>>();

        self.client
            .publish_diagnostics(params.uri.clone(), diagnostics, Some(params.version))
            .await;

        if let Some(ast) = ast {
            self.ast_map.insert(params.uri.to_string(), ast);
        }
        self.client
            .log_message(
                MessageType::INFO,
                &format!("semantic tokens: {:?}", semantic_tokens),
            )
            .await;
        self.semantic_token_map
            .insert(params.uri.to_string(), semantic_tokens);
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    log_panics::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    // //println!("Starting server...");

    let (service, socket) = LspService::new(|client| Backend {
        client,
        ast_map: DashMap::new(),
        semantic_token_map: DashMap::new(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
