use crate::{
    channel::Channel, messages::Message, Command, Error, Params, Result, SslMode, Version,
};
use futures::io::{self, AsyncRead, AsyncWrite};

#[derive(Debug)]
pub struct Connection<
    R: AsyncRead + Send + Sync + Unpin + 'static,
    W: AsyncWrite + Send + Sync + Unpin + 'static,
> {
    properties: (Version, Params, SslMode),
    channel: Channel<R, W>,
}

impl<
        R: AsyncRead + Send + Sync + Unpin + 'static,
        W: AsyncWrite + Send + Sync + Unpin + 'static,
    > Connection<R, W>
{
    pub fn new(properties: (Version, Params, SslMode), channel: Channel<R, W>) -> Self {
        Self {
            properties,
            channel,
        }
    }

    pub fn properties(&self) -> &(Version, Params, SslMode) {
        &(self.properties)
    }

    pub async fn send_ready_for_query(&mut self) -> io::Result<Result<()>> {
        trace!("send ready for query message");
        self.channel.send_message(Message::ReadyForQuery).await?;
        Ok(Ok(()))
    }

    pub async fn read_query(&mut self) -> io::Result<Result<Command>> {
        let tag = self.channel.read_tag().await?;
        if b'X' == tag {
            Ok(Ok(Command::Terminate))
        } else {
            let len = self.channel.read_message_len().await?;
            let sql_buff = self.channel.receive_message(len).await?;
            debug!("FOR TEST sql = {:?}", sql_buff);
            let sql = match String::from_utf8(sql_buff[..sql_buff.len() - 1].to_vec()) {
                Ok(sql) => sql,
                Err(_e) => return Ok(Err(Error)),
            };
            trace!("SQL = {}", sql);
            Ok(Ok(Command::Query(sql)))
        }
    }

    pub async fn send_row_description(&mut self, fields: Vec<Field>) -> io::Result<()> {
        self.channel
            .send_message(Message::RowDescription(
                fields
                    .into_iter()
                    .map(|f| (f.name, f.type_id, f.type_size))
                    .collect(),
            ))
            .await?;
        trace!("row description is sent");
        Ok(())
    }

    pub async fn send_row_data(&mut self, row: Vec<String>) -> io::Result<()> {
        self.channel.send_message(Message::DataRow(row)).await?;
        Ok(())
    }

    pub async fn send_command_complete(&mut self, message: Message) -> io::Result<()> {
        self.channel.send_message(message).await?;
        trace!("end of the command is sent");
        Ok(())
    }
}

impl<
        R: AsyncRead + Send + Sync + Unpin + 'static,
        W: AsyncWrite + Send + Sync + Unpin + 'static,
    > PartialEq for Connection<R, W>
{
    fn eq(&self, other: &Self) -> bool {
        self.properties().eq(other.properties())
    }
}

#[derive(Clone)]
pub struct Field {
    pub name: String,
    pub type_id: i32,
    pub type_size: i16,
}

impl Field {
    pub fn new(name: String, type_id: i32, type_size: i16) -> Self {
        Self {
            name,
            type_id,
            type_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{channel::Channel, supported_version};
    use bytes::BytesMut;
    use test_helpers::async_io;

    #[async_std::test]
    async fn send_ready_for_query() -> io::Result<()> {
        let test_case = async_io::TestCase::empty().await;
        let mut connection = Connection::new(
            (supported_version(), Params(vec![]), SslMode::Disable),
            Channel::new(test_case.clone(), test_case.clone()),
        );

        let ready_for_query = connection.send_ready_for_query().await?;

        assert_eq!(ready_for_query, Ok(()));

        let actual_content = test_case.read_result().await;
        let mut expected_content = BytesMut::new();
        expected_content.extend_from_slice(Message::ReadyForQuery.as_vec().as_slice());

        assert_eq!(actual_content, expected_content);

        Ok(())
    }

    #[cfg(test)]
    mod read_query {
        use super::*;

        #[async_std::test]
        async fn read_termination_command() -> io::Result<()> {
            let test_case = async_io::TestCase::with_content(vec![&[88], &[0, 0, 0, 4]]).await;
            let mut connection = Connection::new(
                (supported_version(), Params(vec![]), SslMode::Disable),
                Channel::new(test_case.clone(), test_case.clone()),
            );

            let query = connection.read_query().await?;

            assert_eq!(query, Ok(Command::Terminate));

            Ok(())
        }

        #[async_std::test]
        async fn read_query_successfully() -> io::Result<()> {
            let test_case =
                async_io::TestCase::with_content(vec![&[81], &[0, 0, 0, 14], b"select 1;\0"]).await;
            let mut connection = Connection::new(
                (supported_version(), Params(vec![]), SslMode::Disable),
                Channel::new(test_case.clone(), test_case.clone()),
            );

            let query = connection.read_query().await?;

            assert_eq!(query, Ok(Command::Query("select 1;".to_owned())));

            Ok(())
        }

        #[async_std::test]
        async fn unexpected_eof_when_read_type_code_of_query_request() {
            let test_case = async_io::TestCase::with_content(vec![]).await;
            let mut connection = Connection::new(
                (supported_version(), Params(vec![]), SslMode::Disable),
                Channel::new(test_case.clone(), test_case.clone()),
            );

            let query = connection.read_query().await;

            assert!(query.is_err());
        }

        #[async_std::test]
        async fn unexpected_eof_when_read_length_of_query() {
            let test_case = async_io::TestCase::with_content(vec![&[81]]).await;
            let mut connection = Connection::new(
                (supported_version(), Params(vec![]), SslMode::Disable),
                Channel::new(test_case.clone(), test_case.clone()),
            );

            let query = connection.read_query().await;

            assert!(query.is_err());
        }

        #[async_std::test]
        async fn unexpected_eof_when_query_string() {
            let test_case =
                async_io::TestCase::with_content(vec![&[81], &[0, 0, 0, 14], b"sel;\0"]).await;
            let mut connection = Connection::new(
                (supported_version(), Params(vec![]), SslMode::Disable),
                Channel::new(test_case.clone(), test_case.clone()),
            );

            let query = connection.read_query().await;

            assert!(query.is_err());
        }
    }

    #[async_std::test]
    async fn send_field_description_query() -> io::Result<()> {
        let test_case = async_io::TestCase::empty().await;
        let mut connection = Connection::new(
            (supported_version(), Params(vec![]), SslMode::Disable),
            Channel::new(test_case.clone(), test_case.clone()),
        );
        let fields = vec![
            Field::new(
                "c1".to_owned(),
                23, // int4 type code
                4,
            ),
            Field::new("c2".to_owned(), 23, 4),
        ];

        connection.send_row_description(fields.clone()).await?;

        let actual_content = test_case.read_result().await;
        let mut expected_content = BytesMut::new();
        expected_content.extend_from_slice(
            Message::RowDescription(
                fields
                    .into_iter()
                    .map(|f| (f.name, f.type_id, f.type_size))
                    .collect(),
            )
            .as_vec()
            .as_slice(),
        );

        assert_eq!(actual_content, expected_content);

        Ok(())
    }

    #[async_std::test]
    async fn send_rows_data() -> io::Result<()> {
        let test_case = async_io::TestCase::empty().await;
        let mut connection = Connection::new(
            (supported_version(), Params(vec![]), SslMode::Disable),
            Channel::new(test_case.clone(), test_case.clone()),
        );

        let rows = vec![
            vec!["1".to_owned(), "2".to_owned()],
            vec!["3".to_owned(), "4".to_owned()],
            vec!["5".to_owned(), "6".to_owned()],
        ];
        for row in rows.iter() {
            connection.send_row_data(row.clone()).await?;
        }

        let actual_content = test_case.read_result().await;
        let mut expected_content = BytesMut::new();
        for row in rows {
            expected_content.extend_from_slice(Message::DataRow(row).as_vec().as_slice());
        }

        assert_eq!(actual_content, expected_content);

        Ok(())
    }

    #[async_std::test]
    async fn send_command_complete() -> io::Result<()> {
        let test_case = async_io::TestCase::empty().await;
        let mut connection = Connection::new(
            (supported_version(), Params(vec![]), SslMode::Disable),
            Channel::new(test_case.clone(), test_case.clone()),
        );
        connection
            .send_command_complete(Message::CommandComplete("SELECT".to_owned()))
            .await?;

        let actual_content = test_case.read_result().await;
        let mut expected_content = BytesMut::new();
        expected_content.extend_from_slice(
            Message::CommandComplete("SELECT".to_owned())
                .as_vec()
                .as_slice(),
        );
        assert_eq!(actual_content, expected_content);

        Ok(())
    }
}