use crate::storage::persistent;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

pub type Projection = (Vec<String>, Vec<Vec<String>>);

pub struct RelationalStorage<P: persistent::PersistentStorage> {
    key_id_generator: usize,
    persistent: P,
}

impl Default for RelationalStorage<persistent::SledPersistentStorage> {
    fn default() -> Self {
        Self::new(persistent::SledPersistentStorage::default())
    }
}

impl<P: persistent::PersistentStorage> RelationalStorage<P> {
    pub fn new(mut persistent: P) -> Self {
        persistent.create_namespace("system").unwrap();
        Self {
            key_id_generator: 0,
            persistent,
        }
    }

    #[allow(clippy::match_wild_err_arm, clippy::map_entry)]
    pub fn create_schema(&mut self, schema_name: String) -> Result<()> {
        match self.persistent.create_namespace(schema_name.as_str()) {
            Ok(Ok(())) => Ok(()),
            Ok(Err(persistent::NamespaceAlreadyExists)) => {
                Err(Error::SchemaAlreadyExists(schema_name))
            }
            _ => unimplemented!(),
        }
    }

    pub fn drop_schema(&mut self, schema_name: String) -> Result<()> {
        match self.persistent.drop_namespace(schema_name.as_str()) {
            Ok(Ok(())) => Ok(()),
            Ok(Err(persistent::NamespaceDoesNotExist)) => {
                Err(Error::SchemaDoesNotExist(schema_name))
            }
            _ => unimplemented!(),
        }
    }

    pub fn create_table(
        &mut self,
        schema_name: String,
        table_name: String,
        column_names: Vec<String>,
    ) -> Result<()> {
        match self
            .persistent
            .create_object(schema_name.as_str(), table_name.as_str())
        {
            Ok(Ok(())) => {
                self.persistent.create_object(
                    "system",
                    (schema_name.clone() + "." + table_name.as_str()).as_str(),
                );
                self.persistent.write(
                    "system",
                    (schema_name + "." + table_name.as_str()).as_str(),
                    vec![(
                        self.key_id_generator.to_be_bytes().to_vec(),
                        column_names
                            .iter()
                            .map(|s| s.clone().into_bytes())
                            .collect(),
                    )],
                );
                self.key_id_generator += 1;
                Ok(())
            }
            Ok(Err(persistent::CreateObjectError::ObjectAlreadyExists)) => Err(
                Error::TableAlreadyExists(schema_name + "." + table_name.as_str()),
            ),
            _ => unimplemented!(),
        }
    }

    pub fn table_columns(
        &mut self,
        schema_name: String,
        table_name: String,
    ) -> Result<Vec<String>> {
        let reads = self.persistent.read(
            "system",
            (schema_name.clone() + "." + table_name.as_str()).as_str(),
        );
        match reads {
            Ok(Ok(reads)) => Ok(reads
                .map(persistent::Result::unwrap)
                .map(|(_id, columns)| {
                    columns
                        .iter()
                        .map(|c| String::from_utf8(c.to_vec()).unwrap())
                        .collect()
                })
                .next()
                .unwrap()),
            Ok(Err(persistent::OperationOnObjectError::ObjectDoesNotExist)) => Err(
                Error::TableDoesNotExist(schema_name + "." + table_name.as_str()),
            ),
            _ => unimplemented!(),
        }
    }

    pub fn drop_table(&mut self, schema_name: String, table_name: String) -> Result<()> {
        match self
            .persistent
            .drop_object(schema_name.as_str(), table_name.as_str())
        {
            Ok(Ok(())) => Ok(()),
            Ok(Err(persistent::DropObjectError::ObjectDoesNotExist)) => Err(
                Error::TableDoesNotExist(schema_name + "." + table_name.as_str()),
            ),
            _ => unimplemented!(),
        }
    }

    pub fn insert_into(
        &mut self,
        schema_name: String,
        table_name: String,
        values: Vec<Vec<String>>,
    ) -> Result<()> {
        let mut to_write = vec![];
        for value in values {
            let key = self.key_id_generator.to_be_bytes().to_vec();
            to_write.push((key, value.iter().map(|s| s.clone().into_bytes()).collect()));
            self.key_id_generator += 1;
        }
        match self
            .persistent
            .write(schema_name.as_str(), table_name.as_str(), to_write)
        {
            Ok(Ok(_size)) => Ok(()),
            Ok(Err(persistent::OperationOnObjectError::ObjectDoesNotExist)) => Err(
                Error::TableDoesNotExist(schema_name + "." + table_name.as_str()),
            ),
            _ => unimplemented!(),
        }
    }

    pub fn select_all_from(
        &mut self,
        schema_name: String,
        table_name: String,
        columns: Vec<String>,
    ) -> Result<Projection> {
        let all_columns = self.table_columns(schema_name.clone(), table_name.clone())?;
        let mut column_indexes = vec![];
        for (i, column) in columns.iter().enumerate() {
            for (index, name) in all_columns.iter().enumerate() {
                if name == column {
                    column_indexes.push((index, i));
                }
            }
        }
        Ok((
            columns,
            self.persistent
                .read(schema_name.as_str(), table_name.as_str())
                .unwrap()
                .unwrap()
                .map(persistent::Result::unwrap)
                .map(|(_key, values)| values)
                .map(|bytes| {
                    let all_values = bytes
                        .iter()
                        .map(|b| String::from_utf8(b.to_vec()).unwrap())
                        .collect::<Vec<String>>();
                    let mut values = vec![];
                    for (origin, ord) in &column_indexes {
                        for (index, value) in all_values.iter().enumerate() {
                            if index == *origin {
                                values.push((ord, value.clone()))
                            }
                        }
                    }
                    values.iter().map(|(_, value)| value.clone()).collect()
                })
                .collect(),
        ))
    }

    pub fn update_all(
        &mut self,
        schema_name: String,
        table_name: String,
        value: String,
    ) -> Result<usize> {
        let reads = self
            .persistent
            .read(schema_name.as_str(), table_name.as_str());
        match reads {
            Ok(Ok(reads)) => {
                let to_update: Vec<(Vec<u8>, Vec<Vec<u8>>)> = reads
                    .map(persistent::Result::unwrap)
                    .map(|(key, _)| (key, vec![value.clone().into_bytes()]))
                    .collect();

                let len = to_update.len();
                self.persistent
                    .write(schema_name.as_str(), table_name.as_str(), to_update)
                    .unwrap();
                Ok(len)
            }
            Ok(Err(persistent::OperationOnObjectError::ObjectDoesNotExist)) => Err(
                Error::TableDoesNotExist(schema_name + "." + table_name.as_str()),
            ),
            _ => unimplemented!(),
        }
    }

    pub fn delete_all_from(&mut self, schema_name: String, table_name: String) -> Result<usize> {
        let reads = self
            .persistent
            .read(schema_name.as_str(), table_name.as_str());

        let to_delete: Vec<Vec<u8>> = match reads {
            Ok(Ok(reads)) => reads
                .map(persistent::Result::unwrap)
                .map(|(key, _)| key)
                .collect(),
            Ok(Err(persistent::OperationOnObjectError::ObjectDoesNotExist)) => {
                return Err(Error::TableDoesNotExist(
                    schema_name + "." + table_name.as_str(),
                ))
            }
            _ => unimplemented!(),
        };

        let len = to_delete.len();
        self.persistent
            .delete(schema_name.as_str(), table_name.as_str(), to_delete);

        Ok(len)
    }
}

#[derive(Debug, PartialEq, Error)]
pub enum Error {
    #[error("schema {0} already exists")]
    SchemaAlreadyExists(String),
    #[error("table {0} already exists")]
    TableAlreadyExists(String),
    #[error("schema {0} does not exist")]
    SchemaDoesNotExist(String),
    #[error("table {0} does not exist")]
    TableDoesNotExist(String),
    #[error("not supported operation")]
    NotSupportedOperation(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_schemas_with_different_names() {
        let mut storage = RelationalStorage::default();

        assert_eq!(storage.create_schema("schema_1".to_owned()), Ok(()));
        assert_eq!(storage.create_schema("schema_2".to_owned()), Ok(()));
    }

    #[test]
    fn create_schema_with_existing_name() -> Result<()> {
        let mut storage = RelationalStorage::default();

        storage.create_schema("schema_name".to_owned())?;

        assert_eq!(
            storage.create_schema("schema_name".to_owned()),
            Err(Error::SchemaAlreadyExists("schema_name".to_owned()))
        );

        Ok(())
    }

    #[test]
    fn drop_schema() -> Result<()> {
        let mut storage = RelationalStorage::default();

        storage.create_schema("schema_name".to_owned())?;

        assert_eq!(storage.drop_schema("schema_name".to_owned()), Ok(()));
        assert_eq!(storage.create_schema("schema_name".to_owned()), Ok(()));

        Ok(())
    }

    #[test]
    fn drop_schema_that_was_not_created() {
        let mut storage = RelationalStorage::default();

        assert_eq!(
            storage.drop_schema("does_not_exists".to_owned()),
            Err(Error::SchemaDoesNotExist("does_not_exists".to_owned()))
        );
    }

    #[test]
    fn drop_schema_drops_tables_in_it() -> Result<()> {
        let mut storage = RelationalStorage::default();

        storage.create_schema("schema_name".to_owned())?;
        storage.create_table(
            "schema_name".to_owned(),
            "table_name_1".to_owned(),
            vec!["column_test".to_owned()],
        )?;
        storage.create_table(
            "schema_name".to_owned(),
            "table_name_2".to_owned(),
            vec!["column_test".to_owned()],
        )?;

        assert_eq!(storage.drop_schema("schema_name".to_owned()), Ok(()));
        assert_eq!(storage.create_schema("schema_name".to_owned()), Ok(()));
        assert_eq!(
            storage.create_table(
                "schema_name".to_owned(),
                "table_name_1".to_owned(),
                vec!["column_test".to_owned()]
            ),
            Ok(())
        );
        assert_eq!(
            storage.create_table(
                "schema_name".to_owned(),
                "table_name_2".to_owned(),
                vec!["column_test".to_owned()]
            ),
            Ok(())
        );

        Ok(())
    }

    #[test]
    fn create_tables_with_different_names() -> Result<()> {
        let mut storage = RelationalStorage::default();

        storage.create_schema("schema_name".to_owned())?;

        assert_eq!(
            storage.create_table(
                "schema_name".to_owned(),
                "table_name_1".to_owned(),
                vec!["column_test".to_owned()]
            ),
            Ok(())
        );
        assert_eq!(
            storage.create_table(
                "schema_name".to_owned(),
                "table_name_2".to_owned(),
                vec!["column_test".to_owned()]
            ),
            Ok(())
        );

        Ok(())
    }

    #[test]
    fn create_table_with_the_same_name() -> Result<()> {
        let mut storage = RelationalStorage::default();

        create_table(
            &mut storage,
            "schema_name",
            "table_name",
            vec!["column_test"],
        )?;

        assert_eq!(
            storage.create_table(
                "schema_name".to_owned(),
                "table_name".to_owned(),
                vec!["column_test".to_owned()]
            ),
            Err(Error::TableAlreadyExists(
                "schema_name.table_name".to_owned()
            ))
        );
        Ok(())
    }

    #[test]
    fn create_table_with_the_same_name_in_different_schemas() -> Result<()> {
        let mut storage = RelationalStorage::default();

        storage.create_schema("schema_name_1".to_owned())?;
        storage.create_schema("schema_name_2".to_owned())?;
        assert_eq!(
            storage.create_table(
                "schema_name_1".to_owned(),
                "table_name".to_owned(),
                vec!["column_test".to_owned()]
            ),
            Ok(())
        );
        assert_eq!(
            storage.create_table(
                "schema_name_2".to_owned(),
                "table_name".to_owned(),
                vec!["column_test".to_owned()]
            ),
            Ok(())
        );
        Ok(())
    }

    #[test]
    fn drop_table() -> Result<()> {
        let mut storage = RelationalStorage::default();

        create_table(
            &mut storage,
            "schema_name",
            "table_name",
            vec!["column_test"],
        )?;
        assert_eq!(
            storage.drop_table("schema_name".to_owned(), "table_name".to_owned()),
            Ok(())
        );
        assert_eq!(
            storage.create_table(
                "schema_name".to_owned(),
                "table_name".to_owned(),
                vec!["column_test".to_owned()]
            ),
            Ok(())
        );
        Ok(())
    }

    #[test]
    fn drop_not_created_table() -> Result<()> {
        let mut storage = RelationalStorage::default();

        storage.create_schema("schema_name".to_owned())?;
        assert_eq!(
            storage.drop_table("schema_name".to_owned(), "not_existed_table".to_owned()),
            Err(Error::TableDoesNotExist(
                "schema_name.not_existed_table".to_owned()
            ))
        );

        Ok(())
    }

    #[test]
    fn insert_row_into_table() -> Result<()> {
        let mut storage = RelationalStorage::default();

        create_table(
            &mut storage,
            "schema_name",
            "table_name",
            vec!["column_test"],
        )?;
        assert_eq!(
            storage.insert_into(
                "schema_name".to_owned(),
                "table_name".to_owned(),
                vec![vec!["123".to_owned()]],
            ),
            Ok(())
        );

        let table_columns =
            storage.table_columns("schema_name".to_owned(), "table_name".to_owned())?;

        assert_eq!(
            storage.select_all_from(
                "schema_name".to_owned(),
                "table_name".to_owned(),
                table_columns
            ),
            Ok((vec!["column_test".to_owned()], vec![vec!["123".to_owned()]]))
        );

        Ok(())
    }

    #[test]
    fn insert_many_rows_into_table() -> Result<()> {
        let mut storage = RelationalStorage::default();

        create_table(
            &mut storage,
            "schema_name",
            "table_name",
            vec!["column_test"],
        )?;
        storage.insert_into(
            "schema_name".to_owned(),
            "table_name".to_owned(),
            vec![vec!["123".to_owned()]],
        )?;
        storage.insert_into(
            "schema_name".to_owned(),
            "table_name".to_owned(),
            vec![vec!["456".to_owned()]],
        )?;

        let table_columns =
            storage.table_columns("schema_name".to_owned(), "table_name".to_owned())?;

        assert_eq!(
            storage.select_all_from(
                "schema_name".to_owned(),
                "table_name".to_owned(),
                table_columns
            ),
            Ok((
                vec!["column_test".to_owned()],
                vec![vec!["123".to_owned()], vec!["456".to_owned()]]
            ))
        );

        Ok(())
    }

    #[test]
    fn insert_into_non_existent_table() -> Result<()> {
        let mut storage = RelationalStorage::default();

        storage.create_schema("schema_name".to_owned())?;
        assert_eq!(
            storage.insert_into(
                "schema_name".to_owned(),
                "not_existed".to_owned(),
                vec![vec!["123".to_owned()]],
            ),
            Err(Error::TableDoesNotExist(
                "schema_name.not_existed".to_owned()
            ))
        );

        Ok(())
    }

    #[test]
    fn select_from_table_that_does_not_exist() -> Result<()> {
        let mut storage = RelationalStorage::default();

        storage.create_schema("schema_name".to_owned())?;
        assert_eq!(
            storage.table_columns("schema_name".to_owned(), "not_existed".to_owned()),
            Err(Error::TableDoesNotExist(
                "schema_name.not_existed".to_owned()
            ))
        );

        Ok(())
    }

    #[test]
    fn update_all_records() -> Result<()> {
        let mut storage = RelationalStorage::default();

        create_table(
            &mut storage,
            "schema_name",
            "table_name",
            vec!["column_test"],
        )?;
        storage.insert_into(
            "schema_name".to_owned(),
            "table_name".to_owned(),
            vec![vec!["123".to_owned()]],
        )?;
        storage.insert_into(
            "schema_name".to_owned(),
            "table_name".to_owned(),
            vec![vec!["456".to_owned()]],
        )?;
        storage.insert_into(
            "schema_name".to_owned(),
            "table_name".to_owned(),
            vec![vec!["789".to_owned()]],
        )?;

        assert_eq!(
            storage.update_all(
                "schema_name".to_owned(),
                "table_name".to_owned(),
                "567".to_owned()
            ),
            Ok(3)
        );

        let table_columns =
            storage.table_columns("schema_name".to_owned(), "table_name".to_owned())?;

        assert_eq!(
            storage.select_all_from(
                "schema_name".to_owned(),
                "table_name".to_owned(),
                table_columns
            ),
            Ok((
                vec!["column_test".to_owned()],
                vec![
                    vec!["567".to_owned()],
                    vec!["567".to_owned()],
                    vec!["567".to_owned()]
                ]
            ))
        );

        Ok(())
    }

    #[test]
    fn update_not_existed_table() -> Result<()> {
        let mut storage = RelationalStorage::default();

        storage.create_schema("schema_name".to_owned())?;
        assert_eq!(
            storage.update_all(
                "schema_name".to_owned(),
                "not_existed".to_owned(),
                "123".to_owned()
            ),
            Err(Error::TableDoesNotExist(
                "schema_name.not_existed".to_owned()
            ))
        );

        Ok(())
    }

    #[test]
    fn delete_all_from_table() -> Result<()> {
        let mut storage = RelationalStorage::default();

        create_table(
            &mut storage,
            "schema_name",
            "table_name",
            vec!["column_test"],
        )?;
        storage.insert_into(
            "schema_name".to_owned(),
            "table_name".to_owned(),
            vec![vec!["123".to_owned()]],
        )?;
        storage.insert_into(
            "schema_name".to_owned(),
            "table_name".to_owned(),
            vec![vec!["456".to_owned()]],
        )?;
        storage.insert_into(
            "schema_name".to_owned(),
            "table_name".to_owned(),
            vec![vec!["789".to_owned()]],
        )?;

        assert_eq!(
            storage.delete_all_from("schema_name".to_owned(), "table_name".to_owned()),
            Ok(3)
        );

        let table_columns =
            storage.table_columns("schema_name".to_owned(), "table_name".to_owned())?;

        assert_eq!(
            storage.select_all_from(
                "schema_name".to_owned(),
                "table_name".to_owned(),
                table_columns
            ),
            Ok((vec!["column_test".to_owned()], vec![]))
        );

        Ok(())
    }

    #[test]
    fn delete_all_from_not_existed_table() -> Result<()> {
        let mut storage = RelationalStorage::default();

        storage.create_schema("schema_name".to_owned())?;

        assert_eq!(
            storage.delete_all_from("schema_name".to_owned(), "table_name".to_owned()),
            Err(Error::TableDoesNotExist(
                "schema_name.table_name".to_owned()
            ))
        );

        Ok(())
    }

    #[test]
    fn select_all_from_table_with_many_columns() -> Result<()> {
        let mut storage = RelationalStorage::default();

        create_table(
            &mut storage,
            "schema_name",
            "table_name",
            vec!["column_1", "column_2", "column_3"],
        )?;
        storage.insert_into(
            "schema_name".to_owned(),
            "table_name".to_owned(),
            vec![vec!["1".to_owned(), "2".to_owned(), "3".to_owned()]],
        )?;

        let table_columns =
            storage.table_columns("schema_name".to_owned(), "table_name".to_owned())?;

        assert_eq!(
            storage.select_all_from(
                "schema_name".to_owned(),
                "table_name".to_owned(),
                table_columns
            ),
            Ok((
                vec![
                    "column_1".to_owned(),
                    "column_2".to_owned(),
                    "column_3".to_owned()
                ],
                vec![vec!["1".to_owned(), "2".to_owned(), "3".to_owned()]]
            ))
        );

        Ok(())
    }

    #[test]
    fn insert_multiple_rows() -> Result<()> {
        let mut storage = RelationalStorage::default();

        create_table(
            &mut storage,
            "schema_name",
            "table_name",
            vec!["column_1", "column_2", "column_3"],
        )?;
        storage.insert_into(
            "schema_name".to_owned(),
            "table_name".to_owned(),
            vec![
                vec!["1".to_owned(), "2".to_owned(), "3".to_owned()],
                vec!["4".to_owned(), "5".to_owned(), "6".to_owned()],
                vec!["7".to_owned(), "8".to_owned(), "9".to_owned()],
            ],
        )?;

        let table_columns =
            storage.table_columns("schema_name".to_owned(), "table_name".to_owned())?;

        assert_eq!(
            storage.select_all_from(
                "schema_name".to_owned(),
                "table_name".to_owned(),
                table_columns
            ),
            Ok((
                vec![
                    "column_1".to_owned(),
                    "column_2".to_owned(),
                    "column_3".to_owned()
                ],
                vec![
                    vec!["1".to_owned(), "2".to_owned(), "3".to_owned()],
                    vec!["4".to_owned(), "5".to_owned(), "6".to_owned()],
                    vec!["7".to_owned(), "8".to_owned(), "9".to_owned()],
                ],
            ))
        );

        Ok(())
    }

    #[test]
    fn select_first_and_last_columns_from_table_with_multiple_columns() -> Result<()> {
        let mut storage = RelationalStorage::default();

        create_table(
            &mut storage,
            "schema_name",
            "table_name",
            vec!["first", "middle", "last"],
        )?;
        storage.insert_into(
            "schema_name".to_owned(),
            "table_name".to_owned(),
            vec![
                vec!["1".to_owned(), "2".to_owned(), "3".to_owned()],
                vec!["4".to_owned(), "5".to_owned(), "6".to_owned()],
                vec!["7".to_owned(), "8".to_owned(), "9".to_owned()],
            ],
        )?;

        assert_eq!(
            storage.select_all_from(
                "schema_name".to_owned(),
                "table_name".to_owned(),
                vec!["first".to_owned(), "last".to_owned()]
            ),
            Ok((
                vec!["first".to_owned(), "last".to_owned(),],
                vec![
                    vec!["1".to_owned(), "3".to_owned()],
                    vec!["4".to_owned(), "6".to_owned()],
                    vec!["7".to_owned(), "9".to_owned()],
                ],
            ))
        );

        Ok(())
    }

    #[test]
    fn select_all_columns_reordered_from_table_with_multiple_columns() -> Result<()> {
        let mut storage = RelationalStorage::default();

        create_table(
            &mut storage,
            "schema_name",
            "table_name",
            vec!["first", "middle", "last"],
        )?;
        storage.insert_into(
            "schema_name".to_owned(),
            "table_name".to_owned(),
            vec![
                vec!["1".to_owned(), "2".to_owned(), "3".to_owned()],
                vec!["4".to_owned(), "5".to_owned(), "6".to_owned()],
                vec!["7".to_owned(), "8".to_owned(), "9".to_owned()],
            ],
        )?;

        assert_eq!(
            storage.select_all_from(
                "schema_name".to_owned(),
                "table_name".to_owned(),
                vec!["last".to_owned(), "first".to_owned(), "middle".to_owned()]
            ),
            Ok((
                vec!["last".to_owned(), "first".to_owned(), "middle".to_owned()],
                vec![
                    vec!["3".to_owned(), "1".to_owned(), "2".to_owned()],
                    vec!["6".to_owned(), "4".to_owned(), "5".to_owned()],
                    vec!["9".to_owned(), "7".to_owned(), "8".to_owned()],
                ],
            ))
        );

        Ok(())
    }

    #[test]
    fn select_with_column_name_duplication() -> Result<()> {
        let mut storage = RelationalStorage::default();

        create_table(
            &mut storage,
            "schema_name",
            "table_name",
            vec!["first", "middle", "last"],
        )?;
        storage.insert_into(
            "schema_name".to_owned(),
            "table_name".to_owned(),
            vec![
                vec!["1".to_owned(), "2".to_owned(), "3".to_owned()],
                vec!["4".to_owned(), "5".to_owned(), "6".to_owned()],
                vec!["7".to_owned(), "8".to_owned(), "9".to_owned()],
            ],
        )?;

        assert_eq!(
            storage.select_all_from(
                "schema_name".to_owned(),
                "table_name".to_owned(),
                vec![
                    "last".to_owned(),
                    "middle".to_owned(),
                    "first".to_owned(),
                    "last".to_owned(),
                    "middle".to_owned()
                ]
            ),
            Ok((
                vec![
                    "last".to_owned(),
                    "middle".to_owned(),
                    "first".to_owned(),
                    "last".to_owned(),
                    "middle".to_owned()
                ],
                vec![
                    vec![
                        "3".to_owned(),
                        "2".to_owned(),
                        "1".to_owned(),
                        "3".to_owned(),
                        "2".to_owned()
                    ],
                    vec![
                        "6".to_owned(),
                        "5".to_owned(),
                        "4".to_owned(),
                        "6".to_owned(),
                        "5".to_owned()
                    ],
                    vec![
                        "9".to_owned(),
                        "8".to_owned(),
                        "7".to_owned(),
                        "9".to_owned(),
                        "8".to_owned()
                    ],
                ],
            ))
        );

        Ok(())
    }

    fn create_table<P: persistent::PersistentStorage>(
        storage: &mut RelationalStorage<P>,
        schema_name: &str,
        table_name: &str,
        column_names: Vec<&str>,
    ) -> Result<()> {
        storage.create_schema(schema_name.to_owned())?;
        storage.create_table(
            schema_name.to_owned(),
            table_name.to_owned(),
            column_names
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>(),
        )
    }
}
