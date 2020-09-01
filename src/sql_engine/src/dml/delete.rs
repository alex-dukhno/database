// Copyright 2020 Alex Dukhno
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

use crate::catalog_manager::CatalogManager;
use kernel::SystemResult;
use protocol::results::QueryError;
use protocol::{results::QueryEvent, Sender};
use sqlparser::ast::ObjectName;
use std::sync::Arc;

pub(crate) struct DeleteCommand {
    name: ObjectName,
    storage: Arc<CatalogManager>,
    session: Arc<dyn Sender>,
}

impl DeleteCommand {
    pub(crate) fn new(name: ObjectName, storage: Arc<CatalogManager>, session: Arc<dyn Sender>) -> DeleteCommand {
        DeleteCommand { name, storage, session }
    }

    pub(crate) fn execute(&mut self) -> SystemResult<()> {
        let schema_name = self.name.0[0].to_string();
        let table_name = self.name.0[1].to_string();

        if !self.storage.schema_exists(&schema_name) {
            self.session
                .send(Err(QueryError::schema_does_not_exist(schema_name)))
                .expect("To Send Result to Client");
            return Ok(());
        }

        if !self.storage.table_exists(&schema_name, &table_name) {
            self.session
                .send(Err(QueryError::table_does_not_exist(
                    schema_name + "." + table_name.as_str(),
                )))
                .expect("To Send Result to Client");
            return Ok(());
        }

        match self.storage.delete_all_from(&schema_name, &table_name) {
            Err(e) => Err(e),
            Ok(records_number) => {
                self.session
                    .send(Ok(QueryEvent::RecordsDeleted(records_number)))
                    .expect("To Send Query Result to Client");
                Ok(())
            }
        }
    }
}
