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

use crate::{binary::Binary, repr::Datum, Cursor, DataCatalog, DataTable, Key, SchemaHandle, Value};
use dashmap::DashMap;
use definition::FullTableName;
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, RwLock,
    },
};

#[derive(Default, Debug)]
struct InternalInMemoryTableHandle {
    records: RwLock<BTreeMap<Binary, Binary>>,
    record_ids: AtomicU64,
    column_ords: AtomicU64,
}

#[derive(Debug)]
pub struct InMemoryIndex {
    records: RwLock<BTreeMap<Binary, Binary>>,
    column: usize,
}

impl InMemoryIndex {
    pub(crate) fn new(column: usize) -> InMemoryIndex {
        InMemoryIndex {
            records: RwLock::default(),
            column,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn select(&self) -> Cursor {
        self.records
            .read()
            .unwrap()
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<Cursor>()
    }

    pub(crate) fn insert(&self, value: Value, key: Key) {
        self.records.write().unwrap().insert(value, key);
    }

    pub(crate) fn over(&self, column_index: usize) -> bool {
        self.column == column_index
    }
}

#[derive(Default, Debug, Clone)]
pub struct InMemoryTableHandle {
    inner: Arc<InternalInMemoryTableHandle>,
    indexes: Arc<DashMap<String, Arc<InMemoryIndex>>>,
}

impl InMemoryTableHandle {
    pub(crate) fn index(&self, index: &str) -> Arc<InMemoryIndex> {
        self.indexes.get(index).unwrap().clone()
    }

    pub(crate) fn indexes(&self) -> Vec<Arc<InMemoryIndex>> {
        self.indexes.iter().map(|entry| entry.value().clone()).collect()
    }
}

impl DataTable for InMemoryTableHandle {
    fn select(&self) -> Cursor {
        self.inner
            .records
            .read()
            .unwrap()
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect::<Cursor>()
    }

    fn insert(&self, data: Vec<Value>) -> Vec<Key> {
        let mut rw = self.inner.records.write().unwrap();
        let mut keys = vec![];
        for value in data {
            let record_id = self.inner.record_ids.fetch_add(1, Ordering::SeqCst);
            let key = Binary::pack(&[Datum::from_u64(record_id)]);
            debug_assert!(
                matches!(rw.insert(key.clone(), value), None),
                "insert operation should insert nonexistent key"
            );
            keys.push(key);
        }

        keys
    }

    fn update(&self, data: Vec<(Key, Value)>) -> usize {
        let len = data.len();
        let mut rw = self.inner.records.write().unwrap();
        for (key, value) in data {
            debug_assert!(
                matches!(rw.insert(key, value), Some(_)),
                "update operation should change already existed key"
            );
        }
        len
    }

    fn delete(&self, data: Vec<Key>) -> usize {
        let mut rw = self.inner.records.write().unwrap();
        let mut size = 0;
        let keys = rw
            .iter()
            .filter(|(key, _value)| data.contains(key))
            .map(|(key, _value)| key.clone())
            .collect::<Vec<Binary>>();
        for key in keys.iter() {
            debug_assert!(matches!(rw.remove(key), Some(_)), "delete operation delete existed key");
            size += 1;
        }
        size
    }

    fn next_column_ord(&self) -> u64 {
        self.inner.column_ords.fetch_add(1, Ordering::SeqCst)
    }

    fn create_index(&self, index_name: &str, over_column: usize) {
        self.indexes
            .insert(index_name.to_owned(), Arc::new(InMemoryIndex::new(over_column)));
    }
}

#[derive(Default, Debug)]
pub struct InMemorySchemaHandle {
    tables: DashMap<String, InMemoryTableHandle>,
}

impl SchemaHandle for InMemorySchemaHandle {
    type Table = InMemoryTableHandle;

    fn create_table(&self, table_name: &str) -> bool {
        if self.tables.contains_key(table_name) {
            log::error!("TABLE {:?} is already exist", table_name);
            false
        } else {
            self.tables
                .insert(table_name.to_owned(), InMemoryTableHandle::default());
            log::warn!("TABLE {:?} was created", table_name);
            true
        }
    }

    fn drop_table(&self, table_name: &str) -> bool {
        if !self.tables.contains_key(table_name) {
            log::warn!("TABLE {:?} does not exist", table_name);
            false
        } else {
            self.tables.remove(table_name);
            log::warn!("TABLE {:?} was removed", table_name);
            true
        }
    }

    fn empty(&self) -> bool {
        self.tables.is_empty()
    }

    fn all_tables(&self) -> Vec<String> {
        self.tables.iter().map(|entry| entry.key().clone()).collect()
    }

    fn create_index(&self, table_name: &str, index_name: &str, column_index: usize) -> bool {
        match self.tables.get(table_name) {
            None => {
                log::warn!("TABLE {:?} does not exist", table_name);
                false
            }
            Some(table) => {
                table.create_index(index_name, column_index);
                log::warn!("INDEX {:?} on TABLE {:?} was created", index_name, table_name);
                true
            }
        }
    }

    fn work_with<T, F: Fn(&Self::Table) -> T>(&self, table_name: &str, operation: F) -> Option<T> {
        self.tables.get(table_name).map(|table| operation(&*table))
    }
}

#[derive(Default)]
pub struct InMemoryCatalogHandle {
    schemas: DashMap<String, InMemorySchemaHandle>,
}

impl InMemoryCatalogHandle {
    pub(crate) fn table(&self, full_table_name: &FullTableName) -> InMemoryTableHandle {
        self.schemas
            .get(full_table_name.schema())
            .unwrap()
            .tables
            .get(full_table_name.table())
            .unwrap()
            .clone()
    }
}

impl DataCatalog for InMemoryCatalogHandle {
    type Schema = InMemorySchemaHandle;

    fn create_schema(&self, schema_name: &str) -> bool {
        if self.schemas.contains_key(schema_name) {
            false
        } else {
            self.schemas
                .insert(schema_name.to_owned(), InMemorySchemaHandle::default());
            true
        }
    }

    fn drop_schema(&self, schema_name: &str) -> bool {
        if !self.schemas.contains_key(schema_name) {
            false
        } else {
            self.schemas.remove(schema_name);
            true
        }
    }

    fn work_with<T, F: Fn(&Self::Schema) -> T>(&self, schema_name: &str, operation: F) -> Option<T> {
        self.schemas.get(schema_name).map(|schema| operation(&*schema))
    }
}

#[cfg(test)]
mod general_cases {
    use super::*;

    const SCHEMA: &str = "schema_name";
    const SCHEMA_1: &str = "schema_name_1";
    const SCHEMA_2: &str = "schema_name_2";
    const TABLE: &str = "table_name";
    const TABLE_1: &str = "table_name_1";
    const TABLE_2: &str = "table_name_2";
    const DOES_NOT_EXIST: &str = "does_not_exist";

    fn catalog() -> InMemoryCatalogHandle {
        InMemoryCatalogHandle::default()
    }

    #[cfg(test)]
    mod schemas {
        use super::*;

        #[test]
        fn create_schemas_with_different_names() {
            let catalog_handle = catalog();

            assert_eq!(catalog_handle.create_schema(SCHEMA_1), true);
            assert_eq!(catalog_handle.work_with(SCHEMA_1, |_schema| 1), Some(1));
            assert_eq!(catalog_handle.create_schema(SCHEMA_2), true);
            assert_eq!(catalog_handle.work_with(SCHEMA_2, |_schema| 2), Some(2));
        }

        #[test]
        fn drop_schema() {
            let catalog_handle = catalog();

            assert!(catalog_handle.create_schema(SCHEMA));
            assert_eq!(catalog_handle.drop_schema(SCHEMA), true);
            assert!(matches!(catalog_handle.work_with(SCHEMA, |_schema| 1), None));
            assert_eq!(catalog_handle.create_schema(SCHEMA), true);
            assert!(matches!(catalog_handle.work_with(SCHEMA, |_schema| 1), Some(1)));
        }

        #[test]
        fn dropping_schema_drops_tables_in_it() {
            let catalog_handle = catalog();

            assert_eq!(catalog_handle.create_schema(SCHEMA), true);

            assert_eq!(
                catalog_handle.work_with(SCHEMA, |schema| schema.create_table(TABLE_1)),
                Some(true)
            );
            assert_eq!(
                catalog_handle.work_with(SCHEMA, |schema| schema.create_table(TABLE_2)),
                Some(true)
            );

            assert_eq!(catalog_handle.drop_schema(SCHEMA), true);
            assert_eq!(catalog_handle.create_schema(SCHEMA), true);
            assert_eq!(
                catalog_handle.work_with(SCHEMA, |schema| schema.create_table(TABLE_1)),
                Some(true)
            );
            assert_eq!(
                catalog_handle.work_with(SCHEMA, |schema| schema.create_table(TABLE_2)),
                Some(true)
            );
        }

        #[test]
        fn create_schema_with_the_same_name() {
            let catalog_handle = catalog();

            assert_eq!(catalog_handle.create_schema(SCHEMA), true);
            assert_eq!(catalog_handle.create_schema(SCHEMA), false);
        }

        #[test]
        fn drop_schema_that_does_not_exist() {
            let catalog_handle = catalog();

            assert_eq!(catalog_handle.drop_schema(SCHEMA), false);
        }
    }

    #[cfg(test)]
    mod create_table {
        use super::*;

        #[test]
        fn create_tables_with_different_names() {
            let catalog_handle = catalog();

            assert_eq!(catalog_handle.create_schema(SCHEMA), true);

            assert_eq!(
                catalog_handle.work_with(SCHEMA, |schema| schema.create_table(TABLE_1)),
                Some(true)
            );
            assert_eq!(
                catalog_handle.work_with(SCHEMA, |schema| schema.create_table(TABLE_2)),
                Some(true)
            );
        }

        #[test]
        fn create_tables_with_the_same_name_in_the_same_schema() {
            let catalog_handle = catalog();

            assert_eq!(catalog_handle.create_schema(SCHEMA), true);
            assert_eq!(
                catalog_handle.work_with(SCHEMA, |schema| schema.create_table(TABLE)),
                Some(true)
            );
            assert_eq!(
                catalog_handle.work_with(SCHEMA, |schema| schema.create_table(TABLE)),
                Some(false)
            );
        }

        #[test]
        fn create_tables_in_non_existent_schema() {
            let catalog_handle = catalog();

            assert_eq!(
                catalog_handle.work_with(DOES_NOT_EXIST, |schema| schema.create_table(TABLE)),
                None
            );
        }

        #[test]
        fn create_table_with_the_same_name_in_different_namespaces() {
            let catalog_handle = catalog();

            assert_eq!(catalog_handle.create_schema(SCHEMA_1), true);
            assert_eq!(catalog_handle.create_schema(SCHEMA_2), true);

            assert_eq!(
                catalog_handle.work_with(SCHEMA_1, |schema| schema.create_table(TABLE)),
                Some(true)
            );
            assert_eq!(
                catalog_handle.work_with(SCHEMA_2, |schema| schema.create_table(TABLE)),
                Some(true)
            );
        }
    }

    #[cfg(test)]
    mod drop_table {
        use super::*;

        #[test]
        fn drop_table() {
            let catalog_handle = catalog();

            assert_eq!(catalog_handle.create_schema(SCHEMA), true);

            assert_eq!(
                catalog_handle.work_with(SCHEMA, |schema| schema.create_table(TABLE)),
                Some(true)
            );
            assert_eq!(
                catalog_handle.work_with(SCHEMA, |schema| schema.drop_table(TABLE)),
                Some(true)
            );
        }

        #[test]
        fn drop_table_from_schema_that_does_not_exist() {
            let catalog_handle = catalog();

            assert_eq!(
                catalog_handle.work_with(DOES_NOT_EXIST, |schema| schema.drop_table(TABLE)),
                None
            );
        }

        #[test]
        fn drop_table_that_does_not_exist() {
            let catalog_handle = catalog();

            assert_eq!(catalog_handle.create_schema(SCHEMA), true);
            assert_eq!(
                catalog_handle.work_with(SCHEMA, |s| s.drop_table(DOES_NOT_EXIST)),
                Some(false)
            );
        }
    }

    #[cfg(test)]
    mod operations_on_table {
        use super::*;

        #[test]
        fn scan_table_that_in_schema_that_does_not_exist() {
            let catalog_handle = catalog();

            assert!(matches!(
                catalog_handle.work_with(DOES_NOT_EXIST, |schema| schema.work_with(TABLE, |table| table.select())),
                None
            ));
        }

        #[test]
        fn scan_table_that_does_not_exist() {
            let catalog_handle = catalog();

            assert_eq!(catalog_handle.create_schema(SCHEMA), true);
            assert!(matches!(
                catalog_handle.work_with(SCHEMA, |schema| schema
                    .work_with(DOES_NOT_EXIST, |table| table.select())),
                Some(None)
            ));
        }

        #[test]
        fn insert_a_row_into_table_in_schema_that_does_not_exist() {
            let catalog_handle = catalog();

            assert_eq!(
                catalog_handle.work_with(SCHEMA, |schema| schema.work_with(TABLE, |table| table.insert(vec![]))),
                None
            );
        }

        #[test]
        fn insert_a_row_into_table_that_does_not_exist() {
            let catalog_handle = catalog();

            assert_eq!(catalog_handle.create_schema(SCHEMA), true);

            assert_eq!(
                catalog_handle.work_with(SCHEMA, |schema| schema.work_with(TABLE, |table| table.insert(vec![]))),
                Some(None)
            );
        }

        #[test]
        fn insert_row_into_table_and_scan() {
            let catalog_handle = catalog();

            assert_eq!(catalog_handle.create_schema(SCHEMA), true);
            assert_eq!(
                catalog_handle.work_with(SCHEMA, |schema| schema.create_table(TABLE)),
                Some(true)
            );

            assert_eq!(
                catalog_handle.work_with(SCHEMA, |schema| schema
                    .work_with(TABLE, |table| table.insert(vec![Binary::pack(&[Datum::from_u64(1)])]))),
                Some(Some(vec![Binary::pack(&[Datum::from_u64(0)])]))
            );

            assert_eq!(
                catalog_handle
                    .work_with(SCHEMA, |schema| schema.work_with(TABLE, |table| table.select()))
                    .unwrap()
                    .unwrap()
                    .collect::<Vec<(Key, Value)>>(),
                vec![(Binary::pack(&[Datum::from_u64(0)]), Binary::pack(&[Datum::from_u64(1)]))]
            );
        }

        #[test]
        fn insert_many_rows_into_table_and_scan() {
            let catalog_handle = catalog();

            assert_eq!(catalog_handle.create_schema(SCHEMA), true);
            assert_eq!(
                catalog_handle.work_with(SCHEMA, |schema| schema.create_table(TABLE)),
                Some(true)
            );

            assert_eq!(
                catalog_handle.work_with(SCHEMA, |schema| schema.work_with(TABLE, |table| table.insert(vec![
                    Binary::pack(&[Datum::from_u64(1)]),
                    Binary::pack(&[Datum::from_u64(2)])
                ]))),
                Some(Some(vec![
                    Binary::pack(&[Datum::from_u64(0)]),
                    Binary::pack(&[Datum::from_u64(1)])
                ]))
            );

            assert_eq!(
                catalog_handle
                    .work_with(SCHEMA, |schema| schema.work_with(TABLE, |table| table.select()))
                    .unwrap()
                    .unwrap()
                    .collect::<Vec<(Key, Value)>>(),
                vec![
                    (Binary::pack(&[Datum::from_u64(0)]), Binary::pack(&[Datum::from_u64(1)])),
                    (Binary::pack(&[Datum::from_u64(1)]), Binary::pack(&[Datum::from_u64(2)]))
                ]
            );
        }

        #[test]
        fn delete_from_table_that_in_schema_that_does_not_exist() {
            let catalog_handle = catalog();

            assert_eq!(
                catalog_handle.work_with(DOES_NOT_EXIST, |schema| schema
                    .work_with(TABLE, |table| table.delete(vec![]))),
                None
            );
        }

        #[test]
        fn delete_from_table_that_does_not_exist() {
            let catalog_handle = catalog();

            assert_eq!(catalog_handle.create_schema(SCHEMA), true);
            assert_eq!(
                catalog_handle.work_with(SCHEMA, |schema| schema
                    .work_with(DOES_NOT_EXIST, |table| table.delete(vec![]))),
                Some(None)
            );
        }

        #[test]
        fn insert_delete_scan_records_from_table() {
            let catalog_handle = catalog();

            assert_eq!(catalog_handle.create_schema(SCHEMA), true);
            assert_eq!(
                catalog_handle.work_with(SCHEMA, |schema| schema.create_table(TABLE)),
                Some(true)
            );

            assert_eq!(
                catalog_handle.work_with(SCHEMA, |schema| schema.work_with(TABLE, |table| table.insert(vec![
                    Binary::pack(&[Datum::from_u64(1)]),
                    Binary::pack(&[Datum::from_u64(2)])
                ]))),
                Some(Some(vec![
                    Binary::pack(&[Datum::from_u64(0)]),
                    Binary::pack(&[Datum::from_u64(1)])
                ]))
            );

            assert_eq!(
                catalog_handle.work_with(SCHEMA, |schema| schema
                    .work_with(TABLE, |table| table.delete(vec![Binary::pack(&[Datum::from_u64(1)])]))),
                Some(Some(1))
            );

            assert_eq!(
                catalog_handle
                    .work_with(SCHEMA, |schema| schema.work_with(TABLE, |table| table.select()))
                    .unwrap()
                    .unwrap()
                    .collect::<Vec<(Key, Value)>>(),
                vec![(Binary::pack(&[Datum::from_u64(0)]), Binary::pack(&[Datum::from_u64(1)]))]
            );
        }

        #[test]
        fn update_table_that_in_schema_that_does_not_exist() {
            let catalog_handle = catalog();

            assert_eq!(
                catalog_handle.work_with(DOES_NOT_EXIST, |schema| schema
                    .work_with(TABLE, |table| table.update(vec![]))),
                None
            );
        }

        #[test]
        fn update_table_that_does_not_exist() {
            let catalog_handle = catalog();

            assert_eq!(catalog_handle.create_schema(SCHEMA), true);
            assert_eq!(
                catalog_handle.work_with(SCHEMA, |schema| schema
                    .work_with(DOES_NOT_EXIST, |table| table.update(vec![]))),
                Some(None)
            );
        }

        #[test]
        fn insert_update_scan_records_from_table() {
            let catalog_handle = catalog();

            assert_eq!(catalog_handle.create_schema(SCHEMA), true);
            assert_eq!(
                catalog_handle.work_with(SCHEMA, |schema| schema.create_table(TABLE)),
                Some(true)
            );

            assert_eq!(
                catalog_handle.work_with(SCHEMA, |schema| schema.work_with(TABLE, |table| table.insert(vec![
                    Binary::pack(&[Datum::from_u64(1)]),
                    Binary::pack(&[Datum::from_u64(2)])
                ]))),
                Some(Some(vec![
                    Binary::pack(&[Datum::from_u64(0)]),
                    Binary::pack(&[Datum::from_u64(1)])
                ]))
            );

            assert_eq!(
                catalog_handle.work_with(SCHEMA, |schema| schema.work_with(TABLE, |table| table.update(vec![(
                    Binary::pack(&[Datum::from_u64(1)]),
                    Binary::pack(&[Datum::from_u64(4)])
                )]))),
                Some(Some(1))
            );

            assert_eq!(
                catalog_handle
                    .work_with(SCHEMA, |schema| schema.work_with(TABLE, |table| table.select()))
                    .unwrap()
                    .unwrap()
                    .collect::<Vec<(Key, Value)>>(),
                vec![
                    (Binary::pack(&[Datum::from_u64(0)]), Binary::pack(&[Datum::from_u64(1)])),
                    (Binary::pack(&[Datum::from_u64(1)]), Binary::pack(&[Datum::from_u64(4)])),
                ]
            );
        }
    }
}
