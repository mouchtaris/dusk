use std::collections::HashMap;
use std::error::Error;

use lsp_server::{Connection, Message, Notification, Request, RequestId, Response};
use lsp_types::notification::{DidChangeTextDocument, DidOpenTextDocument, DidSaveTextDocument};
use lsp_types::request::{GotoDefinition, HoverRequest};
use lsp_types::*;

mod analysis;

fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    env_logger::init();
    log::info!("dust-lsp starting");

    let (connection, io_threads) = Connection::stdio();

    let server_capabilities = serde_json::to_value(ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        definition_provider: Some(OneOf::Left(true)),
        ..Default::default()
    })?;

    let init_params = match connection.initialize(server_capabilities) {
        Ok(it) => it,
        Err(e) => {
            if e.channel_is_disconnected() {
                io_threads.join()?;
            }
            return Err(e.into());
        }
    };
    let _init_params: InitializeParams = serde_json::from_value(init_params)?;

    log::info!("dust-lsp initialized");

    let mut server = Server::new();
    server.main_loop(&connection)?;
    drop(connection);
    io_threads.join()?;
    log::info!("dust-lsp shut down");
    Ok(())
}

struct Server {
    documents: HashMap<Url, String>,
}

impl Server {
    fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    fn main_loop(&mut self, connection: &Connection) -> Result<(), Box<dyn Error + Sync + Send>> {
        for msg in &connection.receiver {
            match msg {
                Message::Request(req) => {
                    if connection.handle_shutdown(&req)? {
                        return Ok(());
                    }
                    self.handle_request(connection, req)?;
                }
                Message::Notification(notif) => {
                    self.handle_notification(connection, notif)?;
                }
                Message::Response(_) => {}
            }
        }
        Ok(())
    }

    fn handle_request(
        &self,
        connection: &Connection,
        req: Request,
    ) -> Result<(), Box<dyn Error + Sync + Send>> {
        let req = match cast_request::<HoverRequest>(req) {
            Ok((id, params)) => {
                let uri = &params.text_document_position_params.text_document.uri;
                let pos = params.text_document_position_params.position;
                let result = self.hover(uri, pos);
                let resp = Response::new_ok(id, result);
                connection.sender.send(Message::Response(resp))?;
                return Ok(());
            }
            Err(req) => req,
        };

        let _req = match cast_request::<GotoDefinition>(req) {
            Ok((id, params)) => {
                let uri = &params.text_document_position_params.text_document.uri;
                let pos = params.text_document_position_params.position;
                let result = self.goto_definition(uri, pos);
                let resp = Response::new_ok(id, result);
                connection.sender.send(Message::Response(resp))?;
                return Ok(());
            }
            Err(req) => req,
        };

        log::warn!("unhandled request: {:?}", _req.method);
        Ok(())
    }

    fn handle_notification(
        &mut self,
        connection: &Connection,
        notif: Notification,
    ) -> Result<(), Box<dyn Error + Sync + Send>> {
        let notif = match cast_notification::<DidOpenTextDocument>(notif) {
            Ok(params) => {
                let uri = params.text_document.uri;
                let text = params.text_document.text;
                self.documents.insert(uri.clone(), text.clone());
                self.publish_diagnostics(connection, &uri, &text)?;
                return Ok(());
            }
            Err(notif) => notif,
        };

        let notif = match cast_notification::<DidChangeTextDocument>(notif) {
            Ok(params) => {
                let uri = params.text_document.uri;
                if let Some(change) = params.content_changes.into_iter().last() {
                    self.documents.insert(uri.clone(), change.text.clone());
                    self.publish_diagnostics(connection, &uri, &change.text)?;
                }
                return Ok(());
            }
            Err(notif) => notif,
        };

        let _notif = match cast_notification::<DidSaveTextDocument>(notif) {
            Ok(params) => {
                let uri = params.text_document.uri;
                if let Some(text) = self.documents.get(&uri) {
                    let text = text.clone();
                    self.publish_diagnostics(connection, &uri, &text)?;
                }
                return Ok(());
            }
            Err(notif) => notif,
        };

        log::trace!("unhandled notification: {:?}", _notif.method);
        Ok(())
    }

    fn publish_diagnostics(
        &self,
        connection: &Connection,
        uri: &Url,
        text: &str,
    ) -> Result<(), Box<dyn Error + Sync + Send>> {
        let diagnostics = analysis::diagnose(text);
        let params = PublishDiagnosticsParams::new(uri.clone(), diagnostics, None);
        let notif = Notification::new(
            "textDocument/publishDiagnostics".to_string(),
            serde_json::to_value(params)?,
        );
        connection.sender.send(Message::Notification(notif))?;
        Ok(())
    }

    fn hover(&self, uri: &Url, pos: Position) -> Option<Hover> {
        let text = self.documents.get(uri)?;
        analysis::hover(text, pos)
    }

    fn goto_definition(
        &self,
        uri: &Url,
        pos: Position,
    ) -> Option<GotoDefinitionResponse> {
        let text = self.documents.get(uri)?;
        let location = analysis::goto_definition(text, uri, pos)?;
        Some(GotoDefinitionResponse::Scalar(location))
    }
}

fn cast_request<R>(req: Request) -> Result<(RequestId, R::Params), Request>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    match req.extract(R::METHOD) {
        Ok(val) => Ok(val),
        Err(e) => Err(match e {
            lsp_server::ExtractError::MethodMismatch(req) => req,
            lsp_server::ExtractError::JsonError { method: _, error: _ } => {
                // Can't recover the original request here, but this shouldn't happen
                // in practice with well-formed clients
                panic!("JSON deserialization error in request");
            }
        }),
    }
}

fn cast_notification<N>(notif: Notification) -> Result<N::Params, Notification>
where
    N: lsp_types::notification::Notification,
    N::Params: serde::de::DeserializeOwned,
{
    match notif.extract(N::METHOD) {
        Ok(val) => Ok(val),
        Err(e) => Err(match e {
            lsp_server::ExtractError::MethodMismatch(notif) => notif,
            lsp_server::ExtractError::JsonError { method: _, error: _ } => {
                panic!("JSON deserialization error in notification");
            }
        }),
    }
}
