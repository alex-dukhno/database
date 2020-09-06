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

use crate::plan::{Plan, SchemaCreationInfo};
use crate::{planner::Planner, planner::Result, SchemaName};
use data_manager::DataManager;
use protocol::results::QueryError;
use protocol::Sender;
use sqlparser::ast::ObjectName;
use std::convert::TryFrom;
use std::sync::Arc;

pub(crate) struct CreateSchemaPlanner {
    schema_name: ObjectName,
}

impl CreateSchemaPlanner {
    pub(crate) fn new(schema_name: ObjectName) -> CreateSchemaPlanner {
        CreateSchemaPlanner { schema_name }
    }
}

impl Planner for CreateSchemaPlanner {
    fn plan(self, data_manager: Arc<DataManager>, sender: Arc<dyn Sender>) -> Result<Plan> {
        match SchemaName::try_from(self.schema_name) {
            Ok(schema_name) => match data_manager.schema_exists(schema_name.name()) {
                Some(_) => {
                    sender
                        .send(Err(QueryError::schema_already_exists(schema_name)))
                        .expect("To Send Query Result to Client");
                    Err(())
                }
                None => Ok(Plan::CreateSchema(SchemaCreationInfo::new(schema_name))),
            },
            Err(error) => {
                sender
                    .send(Err(QueryError::syntax_error(error)))
                    .expect("To Send Query Result to Client");
                Err(())
            }
        }
    }
}
