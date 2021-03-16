// Copyright 2020 - present Alex Dukhno
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{
    connection::{
        async_native_tls::TlsStream,
        network::{Network, Stream},
        Channel, ClientRequest, Connection, RequestReceiver, ResponseSender,
    },
    pg_model::{ConnSupervisor, Encryption, ProtocolConfiguration},
};
use async_mutex::Mutex as AsyncMutex;
use blocking::Unblock;
use futures_lite::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use postgres::wire_protocol::{BackendMessage, HandShakeProcess, HandShakeStatus};
use std::{fs::File, io, sync::Arc};

pub struct ConnectionManager {
    network: Network,
    protocol_config: ProtocolConfiguration,
    conn_supervisor: ConnSupervisor,
}

impl ConnectionManager {
    pub fn new(
        network: Network,
        protocol_config: ProtocolConfiguration,
        conn_supervisor: ConnSupervisor,
    ) -> ConnectionManager {
        ConnectionManager {
            network,
            protocol_config,
            conn_supervisor,
        }
    }

    pub async fn accept(&self) -> io::Result<Result<ClientRequest, ()>> {
        match self.network.accept().await {
            Ok((stream, _address)) => {
                match accept_client_request(stream, &self.protocol_config, self.conn_supervisor.clone()).await? {
                    Ok(ok) => Ok(Ok(ok)),
                    Err(()) => Ok(Err(())),
                }
            }
            Err(io_error) => Err(io_error),
        }
    }
}

/// Perform `PostgreSql` wire protocol to accept request and establish
/// connection with a client based on `config` parameters and using `stream` as
/// a medium to communicate
/// As a result of operation returns tuple of `Receiver` and `Sender`
/// that have to be used to communicate with the client on performing commands
async fn accept_client_request(
    stream: Stream,
    config: &ProtocolConfiguration,
    conn_supervisor: ConnSupervisor,
) -> io::Result<Result<ClientRequest, ()>> {
    let mut channel = Channel::Plain(stream);
    let mut process = HandShakeProcess::start();
    let mut current: Option<Vec<u8>> = None;
    loop {
        match process.next_stage(current.as_deref()) {
            Ok(HandShakeStatus::RequestingBytes(len)) => {
                let mut local = vec![b'0'; len];
                local = channel.read_exact(&mut local).await.map(|_| local)?;
                current = Some(local);
            }
            Ok(HandShakeStatus::UpdatingToSecureWithReadingBytes(len)) => {
                channel = match channel {
                    Channel::Plain(mut channel) if config.ssl_support() => {
                        log::warn!("ALEX SECURE CHANNEL");
                        channel.write_all(Encryption::AcceptSsl.into()).await?;
                        Channel::Secure(tls_channel(channel, config).await?)
                    }
                    _ => {
                        channel.write_all(Encryption::RejectSsl.into()).await?;
                        channel
                    }
                };
                let mut local = vec![b'0'; len];
                local = channel.read_exact(&mut local).await.map(|_| local)?;
                log::warn!("ALEX SECURE CHANNEL READ");
                current = Some(local);
            }
            Ok(HandShakeStatus::Cancel(conn_id, secret_key)) => {
                return if conn_supervisor.verify(conn_id, secret_key) {
                    Ok(Ok(ClientRequest::QueryCancellation(conn_id)))
                } else {
                    Ok(Err(()))
                }
            }
            Ok(HandShakeStatus::Done(props)) => {
                channel
                    .write_all(BackendMessage::AuthenticationCleartextPassword.as_vec().as_slice())
                    .await?;
                channel.flush().await?;
                let mut tag_buffer = [0u8; 1];
                let tag = channel.read_exact(&mut tag_buffer).await.map(|_| tag_buffer[0]);
                log::debug!("client message response tag {:?}", tag);
                log::debug!("waiting for authentication response");
                let mut len_buffer = [0u8; 4];
                let len = channel
                    .read_exact(&mut len_buffer)
                    .await
                    .map(|_| u32::from_be_bytes(len_buffer) as usize)?;
                let len = len - 4;
                let mut message_buffer = Vec::with_capacity(len);
                message_buffer.resize(len, b'0');
                let _message = channel.read_exact(&mut message_buffer).await.map(|_| message_buffer)?;
                channel
                    .write_all(BackendMessage::AuthenticationOk.as_vec().as_slice())
                    .await?;

                channel
                    .write_all(
                        BackendMessage::ParameterStatus("client_encoding".to_owned(), "UTF8".to_owned())
                            .as_vec()
                            .as_slice(),
                    )
                    .await?;

                channel
                    .write_all(
                        BackendMessage::ParameterStatus("DateStyle".to_owned(), "ISO".to_owned())
                            .as_vec()
                            .as_slice(),
                    )
                    .await?;

                channel
                    .write_all(
                        BackendMessage::ParameterStatus("integer_datetimes".to_owned(), "off".to_owned())
                            .as_vec()
                            .as_slice(),
                    )
                    .await?;

                channel
                    .write_all(
                        BackendMessage::ParameterStatus("server_version".to_owned(), "12.4".to_owned())
                            .as_vec()
                            .as_slice(),
                    )
                    .await?;

                let (conn_id, secret_key) = match conn_supervisor.alloc() {
                    Ok((c, s)) => (c, s),
                    Err(_error) => {
                        log::error!("ERROR");
                        return Ok(Err(()));
                    }
                };

                log::debug!("start service on connection-{}", conn_id);
                channel
                    .write_all(BackendMessage::BackendKeyData(conn_id, secret_key).as_vec().as_slice())
                    .await?;

                log::debug!("send ready_for_query message");
                channel
                    .write_all(BackendMessage::ReadyForQuery.as_vec().as_slice())
                    .await?;

                let channel = Arc::new(AsyncMutex::new(channel));
                return Ok(Ok(ClientRequest::Connect(Connection {
                    receiver: Box::new(RequestReceiver::new(
                        conn_id,
                        props.clone(),
                        channel.clone(),
                        conn_supervisor,
                    )),
                    sender: Arc::new(ResponseSender::new(props, channel)),
                })));
            }
            Err(error) => {
                log::error!("{}", error);
                return Ok(Err(()));
            }
        }
    }
}

async fn tls_channel<RW>(tcp_channel: RW, config: &ProtocolConfiguration) -> io::Result<TlsStream<RW>>
where
    RW: AsyncRead + AsyncWrite + Unpin,
{
    match config.ssl_config() {
        Some((path, password)) => {
            match super::async_native_tls::accept(Unblock::new(File::open(path)?), password, tcp_channel).await {
                Ok(socket) => Ok(socket),
                Err(_err) => Err(io::Error::from(io::ErrorKind::ConnectionAborted)),
            }
        }
        None => Err(io::Error::from(io::ErrorKind::ConnectionAborted)),
    }
}
