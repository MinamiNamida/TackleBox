use chrono::Utc;

use crate::api::command::{Endpoint, ResponseError, ServerMessage, ServerPayload, SystemCommand, SystemMessage, SystemResponse, UserCommand, UserMessage, UserResponse};

pub struct MsgGen {
    from: Endpoint,
}

impl MsgGen {
    pub fn new(from: Endpoint) -> Self {
        Self { from }
    }

    pub fn user_command(&self, to: Endpoint, cmd: UserCommand) -> ServerMessage {
        ServerMessage {
            from: self.from.clone(),
            to,
            payload: ServerPayload::User(UserMessage::Command(cmd)),
            timestamp: Utc::now(),
        }
    }

    pub fn user_response(&self, to: Endpoint, resp: UserResponse) -> ServerMessage {
        ServerMessage {
            from: self.from.clone(),
            to,
            payload:ServerPayload::User(UserMessage::Response(resp)),
            timestamp: Utc::now(),
        }
    }

    pub fn user_error(&self, to: Endpoint, err: ResponseError) -> ServerMessage {
        ServerMessage {
            from: self.from.clone(),
            to,
            payload: ServerPayload::User(UserMessage::Error(err)),
            timestamp: Utc::now(),
        }
    }

    pub fn sys_command(&self, to: Endpoint, cmd: SystemCommand) -> ServerMessage {
        ServerMessage {
            from: self.from.clone(),
            to,
            payload: ServerPayload::System(SystemMessage::Command(cmd)),
            timestamp: Utc::now(),
        }
    }
    
    pub fn sys_response(&self, to: Endpoint, resp: SystemResponse) -> ServerMessage {
        ServerMessage {
            from: self.from.clone(),
            to,
            payload: ServerPayload::System(SystemMessage::Response(resp)),
            timestamp: Utc::now(),
        }
    }
    pub fn sys_error(&self, to: Endpoint, err: ResponseError) -> ServerMessage {
        ServerMessage {
            from: self.from.clone(),
            to,
            payload: ServerPayload::System(SystemMessage::Error(err)),
            timestamp: Utc::now(),
        }
    }
    
}
