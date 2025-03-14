use std::sync::{Arc, Mutex};
use std::net::SocketAddr;
use axum::{
    body::Bytes,
    extract::{
        ws::{close_code, CloseFrame, Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
        State,
        connect_info::ConnectInfo
    }, response::IntoResponse, routing::any, Json, Router
};
use serde_json::json;
use axum_extra::TypedHeader;
use std::ops::ControlFlow;
use futures::{sink::SinkExt, stream::StreamExt};

use crate::AppState;

pub fn peers_router(state: Arc<Mutex<AppState>>) -> Router<Arc<Mutex<AppState>>> {
    Router::new()
        .route("/", any(handle_get_peer_list))
        .route("/ws", any(handle_ws_upgrade))
        .with_state(state)
}

async fn handle_get_peer_list(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let peers = state.lock().unwrap().peers.lock().unwrap().clone();

    Json(json!(peers))
}

async fn handle_ws_upgrade(
    State(state): State<Arc<Mutex<AppState>>>,
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    println!("Peers handshake");
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };

    println!("`{user_agent}` at {addr} connected.");

    return ws.on_upgrade(move |socket| {
        let peers: Arc<Mutex<Vec<SocketAddr>>> = state.lock().unwrap().peers.clone();
        handle_socket(socket, addr, peers)
    })
}

async fn handle_socket(mut socket: WebSocket, who: SocketAddr, peers: Arc<Mutex<Vec<SocketAddr>>>) {
    if socket
        .send(Message::Ping(Bytes::from_static(&[1, 2, 3])))
        .await
        .is_ok()
    {
        peers.lock().unwrap().push(who);
        println!("Pinged {who}");
    } else {
        println!("Could not send ping to {who}");
        return;
    }

    if let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            if process_message(msg, &who).is_break() {
                return;
            }
        } else {
            println!("client {who} abruptly disconnected");
            return;
        }
    }

    for i in 1..5 {
        if socket
            .send(Message::Text(format!("Hi {i} times!").into()))
            .await
            .is_err()
        {
            println!("client {who} abruptly disconnected");
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    let (mut sender, mut receiver) = socket.split();
    let mut send_task = tokio::spawn(async move {
        let n_msg = 20;
        // for i in 0..n_msg {
        let mut i = 0;
        loop {
            i = i + 1;
            if sender
                .send(Message::Text(format!("Server message {i} ...").into()))
                .await
                .is_err()
            {
                return i;
            }

            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }

        // println!("Sending close to {who}...");
        // if let Err(e) = sender
        //     .send(Message::Close(Some(CloseFrame {
        //         code: close_code::NORMAL,
        //         reason: Utf8Bytes::from_static("Goodbye"),
        //     })))
        //     .await
        // {
        //     println!("Could not send Close due to {e}, probably it is ok?");
        // }
        // n_msg
    });

    let mut recv_task = tokio::spawn(async move {
        let mut cnt = 0;
        while let Some(Ok(msg)) = receiver.next().await {
            cnt += 1;
            if process_message(msg, &who).is_break() {
                break;
            }
        }
        cnt
    });

    tokio::select! {
        rv_a = (&mut send_task) => {
            match rv_a {
                Ok(a) => println!("{a} messages sent to {who}"),
                Err(a) => println!("Error sending messages {a:?}")
            }
            recv_task.abort();
        },
        rv_b = (&mut recv_task) => {
            match rv_b {
                Ok(b) => println!("Received {b} messages"),
                Err(b) => println!("Error receiving messages {b:?}")
            }
            send_task.abort();
        }
    }

    let index = peers.lock().unwrap().iter().position(|x| *x == who).unwrap();
    peers.lock().unwrap().remove(index);
    println!("Websocket context {who} destroyed");
}

fn process_message(msg: Message, who: &SocketAddr) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            println!(">>> {who} sent str: {t:?}");
        }
        Message::Binary(d) => {
            println!(">>> {} sent {} bytes: {:?}", who, d.len(), d);
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                println!(
                    ">>> {} sent close with code {} and reason `{}`",
                    who, cf.code, cf.reason
                );
            } else {
                println!(">>> {who} somehow sent close message without CloseFrame");
            }
            return ControlFlow::Break(());
        }
        Message::Pong(v) => {
            println!(">>> {who} sent pong with {v:?}");
        }
        Message::Ping(v) => {
            println!(">>> {who} sent ping with {v:?}");
        }
    }
    ControlFlow::Continue(())
}