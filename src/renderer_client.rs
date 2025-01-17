use std::sync::Arc;

use ipc_channel::ipc::IpcError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::{mpsc, Mutex, RwLock};

use crate::ipc_protocol::{ClientRequest, ClientSender, ConnectionError, ServerResponse};
use crate::renderer_server::RendererServer;

/// Signals that the IPC connection has been disconnected and therefore the window was probably
/// closed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("Cannot continue to run turtle commands after window is closed. This panic stops the thread, but is not necessarily an error.")]
struct Disconnected;

/// A unique ID used to dispatch responses on the client side
///
/// Treated as an opaque value on the server that is returned back to the client with the response
/// to a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClientId(usize);

/// Spawns the server, and dispatches the messages received from it
///
/// Responses are dispatched back to the correct client based on the received client ID.
#[derive(Debug)]
struct ClientDispatcher {
    /// The server task/process
    ///
    /// When dropped, this will block until the server process has quit. This field is explicitly
    /// owned by this struct and not reference counted in order to guarantee that this happens.
    #[allow(dead_code)]
    server: RendererServer,

    /// A channel for sending responses from the server to each client, indexed by `ClientId`
    ///
    /// Using `RwLock` allows sending multiple times concurrently using `RwLock::read()` and also
    /// allows more clients to be added using `RwLock::write()`.
    clients: Arc<RwLock<Vec<mpsc::UnboundedSender<Result<ServerResponse, Disconnected>>>>>,
}

impl ClientDispatcher {
    async fn new() -> Result<(Self, ClientSender), ConnectionError> {
        let (server, sender, server_responses) = RendererServer::spawn().await?;
        let clients = Arc::new(RwLock::new(Vec::<mpsc::UnboundedSender<_>>::new()));

        let task_clients = clients.clone();
        tokio::spawn(async move {
            loop {
                let (id, response) = match server_responses.recv().await {
                    Ok((id, response)) => (id, Ok(response)),

                    Err(IpcError::Disconnected) => {
                        // Alert all the clients of the disconnection
                        let clients = task_clients.read().await;
                        for client in &*clients {
                            // Ignoring the error since it just means that this particular client
                            // connection must have gotten dropped
                            client.send(Err(Disconnected)).unwrap_or(());
                        }
                        break;
                    }

                    Err(err) => panic!("Error while receiving IPC message: {:?}", err),
                };

                let clients = task_clients.read().await;

                let ClientId(index) = id;
                // Ignoring the error since it just means that this particular client
                // connection must have gotten dropped
                clients[index].send(response).unwrap_or(());
            }
        });

        Ok((Self { server, clients }, sender))
    }

    async fn add_client(&self) -> (ClientId, mpsc::UnboundedReceiver<Result<ServerResponse, Disconnected>>) {
        let mut clients = self.clients.write().await;

        let id = ClientId(clients.len());
        let (sender, receiver) = mpsc::unbounded_channel();
        clients.push(sender);

        (id, receiver)
    }
}

/// Represents a single connection to the server
#[derive(Debug)]
pub struct RendererClient {
    dispatcher: Arc<ClientDispatcher>,
    id: ClientId,
    sender: ClientSender,
    receiver: Mutex<mpsc::UnboundedReceiver<Result<ServerResponse, Disconnected>>>,
}

impl RendererClient {
    /// Spawns a new server process and creates a connection to it
    pub async fn new() -> Result<Self, ConnectionError> {
        let (dispatcher, sender) = ClientDispatcher::new().await?;
        let dispatcher = Arc::new(dispatcher);
        let (id, receiver) = dispatcher.add_client().await;
        let receiver = Mutex::new(receiver);

        Ok(Self {
            dispatcher,
            id,
            sender,
            receiver,
        })
    }

    /// Creates a new renderer client that can also communicate to the same server
    pub async fn split(&self) -> Self {
        let dispatcher = self.dispatcher.clone();
        let (id, receiver) = dispatcher.add_client().await;
        let sender = self.sender.clone();
        let receiver = Mutex::new(receiver);

        Self {
            dispatcher,
            id,
            sender,
            receiver,
        }
    }

    /// Sends a message to the server process
    ///
    /// When possible, prefer using methods from `ProtocolClient` instead of using this directly
    pub fn send(&self, req: ClientRequest) {
        // The error produced by send is a serialization error, so it signals a bug in this code,
        // not something that should be propagated to be handled elsewhere.
        self.sender
            .send(self.id, req)
            .expect("bug: error while sending message through IPC")
    }

    /// Receives a response from the server process
    ///
    /// Note that if the same client sends multiple requests, there is no guarantee that the
    /// responses will be returned in the same order as the requests. It is up to the client to
    /// ensure that ordering or otherwise prevent multiple requests from being sent simultaneously.
    ///
    /// When possible, prefer using methods from `ProtocolClient` instead of using this directly
    pub async fn recv(&self) -> ServerResponse {
        let mut receiver = self.receiver.lock().await;
        receiver
            .recv()
            .await
            // Since this struct keeps a ref-counted copy of the senders, they can't have possibly
            // been dropped at this point.
            .expect("bug: client senders should not be dropped yet")
            // This panic causes the program to exit if turtle commands continue after the window
            // closes
            .unwrap_or_else(|err| panic!("IPC response not received: {}", err))
    }
}
