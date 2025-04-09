use std::{collections::HashMap, sync::{Arc}};

use rocket::{futures::{lock::Mutex, select, FutureExt, SinkExt, StreamExt}, tokio::sync::broadcast::{channel, error::RecvError, Sender}, State};
use ws::{Config, Message, WebSocket};

use crate::{utils::Utils, ClientesSockets};

#[utoipa::path(
    get,  // WebSocket generalmente usa GET
    path = "/ws/{token}",
    responses(
        (status = 101, description = "Protocol Switch (WebSocket)"),
        (status = 401, description = "No autorizado")
    ),
    params(
        ("token" = String, Query, description = "Token JWT para autenticación")
    ),
    tag = "websocket"
)]
#[get("/<token>")]
pub async fn connect_ws(token: &str, ws: WebSocket, clients: &State<ClientesSockets>) -> ws::Channel<'static> {
    let clientes: Arc<Mutex<HashMap<String, Sender<String>>>> = clients.inner().clone();
    let config = Config {
        max_message_size: Some(1024),
        max_frame_size: Some(1024),
        ..Default::default()
    };
    let key = b"1234afeb";

    match decode::<Claims>(token, &DecodingKey::from_secret(key), &Validation::default()) {
        Ok(claims) => {
            let id_usuario = claims.claims.sub;
            let ws = ws.config(config);
            ws.channel(move |mut stream| {
                Box::pin(async move {
                    let (tx, mut rx) = channel(100);
                    clientes.lock().await.insert(id_usuario.clone().to_string(), tx.clone());
                    let count = clientes.lock().await.len();
                    println!("connectes {}", count);
                    let mut stream = stream.fuse();

                    loop {
                        select! {
                            msg = stream.next() => {
                                match msg {
                                    Some(Ok(msg)) => {
                                        if msg.is_text() || msg.is_binary() {
                                            println!("mmsg reccibido {:?}", msg.to_text());
                                        } else if msg.is_close() {
                                            println!("client close conn");
                                            break;
                                        }
                                    },
                                    Some(Err(e)) => {
                                        println!("err msg client {:?}", e);
                                        break;
                                    },
                                    None => {
                                        println!("stream ended");
                                        break;
                                    }
                                }
                            },
                            broadcast_message = rx.recv().fuse() => {
                                match broadcast_message {
                                    Ok(message) => {
                                        println!("msg transmitido {}", message);
                                        if let Err(e) = stream.send(Message::Text(message)).await {
                                            println!("error invalid message {:?}", e);
                                            break;
                                        }
                                    },
                                    Err(RecvError::Closed) => {
                                        println!("channel close");
                                        break;
                                    },
                                    Err(RecvError::Lagged(n)) => {
                                        println!("Recept lag {} msg", n);
                                    }
                                }
                            },
                        }//select
                    }//loop
                    clientes.lock().await.remove(&id_usuario.clone().to_string());
                    let count = clientes.lock().await.len();
                    println!("conn closed {} clients", count);
                    Ok(())
                })//box pin
            })//ws channel
        },
        Err(e) => {
            println!("error connect ws {}", e);
            // Retornamos un channel que inmediatamente cierra la conexión
            ws.config(config).channel(move |stream| {
                Box::pin(async move {
                    println!("Closing connection due to invalid token");
                    Ok(())
                })
            })
        }
    }
}
