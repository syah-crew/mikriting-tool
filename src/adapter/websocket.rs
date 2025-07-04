use actix::prelude::*;
use actix_web_actors::ws;
use async_trait::async_trait;
use log::{debug, error, info};
use std::collections::HashMap;

use crate::domain::{
    models::{VpnUser, LatencyUpdate, WebSocketMessage, DomainError},
    traits::EventPublisher,
};

// WebSocket Actor
pub struct WebSocketActor {
    id: u64,
    manager: Addr<WebSocketManager>,
}

impl WebSocketActor {
    pub fn new(manager: Addr<WebSocketManager>) -> Self {
        Self {
            id: rand::random(),
            manager,
        }
    }
}

impl Actor for WebSocketActor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("WebSocket connection {} started", self.id);
        self.manager.do_send(Connect {
            id: self.id,
            addr: ctx.address(),
        });
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        debug!("WebSocket connection {} stopping", self.id);
        self.manager.do_send(Disconnect { id: self.id });
        Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                // Pong response, no action needed
            }
            Ok(ws::Message::Text(text)) => {
                debug!("Received text message: {}", text);
                // Handle incoming text messages if needed
            }
            Ok(ws::Message::Binary(bin)) => {
                debug!("Received binary message: {} bytes", bin.len());
                // Handle incoming binary messages if needed
            }
            Ok(ws::Message::Close(reason)) => {
                debug!("WebSocket closing: {:?}", reason);
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {
                // Handle continuation frames
                debug!("Received continuation frame");
            }
            Ok(ws::Message::Nop) => {
                // No operation
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                ctx.stop();
            }
        }
    }
}

// WebSocket Messages
#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub id: u64,
    pub addr: Addr<WebSocketActor>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: u64,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastMessage {
    pub message: String,
}

impl Handler<BroadcastMessage> for WebSocketActor {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, ctx: &mut Self::Context) {
        ctx.text(msg.message);
    }
}

// WebSocket Manager
pub struct WebSocketManager {
    connections: HashMap<u64, Addr<WebSocketActor>>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    fn broadcast(&self, message: &str) {
        debug!("Broadcasting message to {} connections", self.connections.len());
        
        for (_id, addr) in &self.connections {
            addr.do_send(BroadcastMessage {
                message: message.to_string(),
            });
        }
    }
}

impl Default for WebSocketManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Actor for WebSocketManager {
    type Context = Context<Self>;

    fn started(&mut self, _: &mut Self::Context) {
        info!("WebSocket manager started");
    }
}

impl Handler<Connect> for WebSocketManager {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Self::Context) {
        debug!("WebSocket connection {} registered", msg.id);
        self.connections.insert(msg.id, msg.addr);
    }
}

impl Handler<Disconnect> for WebSocketManager {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) {
        debug!("WebSocket connection {} disconnected", msg.id);
        self.connections.remove(&msg.id);
    }
}

impl Handler<BroadcastMessage> for WebSocketManager {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, _: &mut Self::Context) {
        self.broadcast(&msg.message);
    }
}

// Event Publisher Implementation
pub struct WebSocketEventPublisher {
    manager: Addr<WebSocketManager>,
}

impl WebSocketEventPublisher {
    pub fn new(manager: Addr<WebSocketManager>) -> Self {
        Self { manager }
    }
}

#[async_trait]
impl EventPublisher for WebSocketEventPublisher {
    async fn publish_vpn_users_update(&self, users: Vec<VpnUser>) -> Result<(), DomainError> {
        let message = WebSocketMessage::vpn_users_update(users);
        let json = serde_json::to_string(&message)
            .map_err(|e| DomainError::SerializationError(e.to_string()))?;
        
        self.manager.do_send(BroadcastMessage { message: json });
        Ok(())
    }

    async fn publish_latency_update(&self, update: LatencyUpdate) -> Result<(), DomainError> {
        let message = WebSocketMessage::latency_update(update.user_name, update.latency);
        let json = serde_json::to_string(&message)
            .map_err(|e| DomainError::SerializationError(e.to_string()))?;
        
        self.manager.do_send(BroadcastMessage { message: json });
        Ok(())
    }
}
