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

use crate::{Planner, Result};
use ast::operations::ScalarOp;
use constraints::TypeConstraint;
use metadata::DataDefinition;
use plan::{FullTableName, Plan, TableId, TableUpdates};
use protocol::{results::QueryError, Sender};
use sql_model::DEFAULT_CATALOG;
use sqlparser::ast::{Assignment, ObjectName};
use std::{collections::HashSet, convert::TryFrom, sync::Arc};

pub(crate) struct UpdatePlanner<'up> {
    table_name: &'up ObjectName,
    assignments: &'up [Assignment],
}

impl<'up> UpdatePlanner<'up> {
    pub(crate) fn new(table_name: &'up ObjectName, assignments: &'up [Assignment]) -> UpdatePlanner<'up> {
        UpdatePlanner {
            table_name,
            assignments,
        }
    }
}

impl Planner for UpdatePlanner<'_> {
    fn plan(self, data_manager: Arc<DataDefinition>, sender: Arc<dyn Sender>) -> Result<Plan> {
        match FullTableName::try_from(self.table_name) {
            Ok(full_table_name) => {
                let (schema_name, table_name) = full_table_name.as_tuple();
                match data_manager.table_exists(DEFAULT_CATALOG, &schema_name, &table_name) {
                    None => Err(()), // TODO catalog does not exists
                    Some((_, None)) => {
                        sender
                            .send(Err(QueryError::schema_does_not_exist(schema_name)))
                            .expect("To Send Query Result to Client");
                        Err(())
                    }
                    Some((_, Some((_, None)))) => {
                        sender
                            .send(Err(QueryError::table_does_not_exist(format!(
                                "{}.{}",
                                schema_name, table_name
                            ))))
                            .expect("To Send Query Result to Client");
                        Err(())
                    }
                    Some((_, Some((schema_id, Some(table_id))))) => {
                        let table_id = TableId::from((schema_id, table_id));
                        let all_columns = data_manager.table_columns(DEFAULT_CATALOG, schema_name, table_name);
                        let mut column_indices = vec![];
                        let mut input = vec![];
                        let mut has_error = false;
                        let mut columns = HashSet::new();
                        for Assignment { id, value } in self.assignments.iter() {
                            let mut found = None;
                            let column_name = id.to_string().to_lowercase();
                            for (index, column_definition) in all_columns.iter().enumerate() {
                                if column_definition.has_name(&column_name) {
                                    match ScalarOp::transform(&value) {
                                        Ok(Ok(value)) => input.push(value),
                                        Ok(Err(error)) => {
                                            has_error = true;
                                            sender
                                                .send(Err(QueryError::syntax_error(error)))
                                                .expect("To Send Result to Client");
                                        }
                                        Err(error) => {
                                            has_error = true;
                                            sender
                                                .send(Err(QueryError::feature_not_supported(error)))
                                                .expect("To Send Result to Client");
                                        }
                                    }
                                    if columns.contains(&column_name) {
                                        has_error = true;
                                        sender
                                            .send(Err(QueryError::syntax_error(format!(
                                                "multiple assignments to same column \"{}\"",
                                                column_name
                                            ))))
                                            .expect("To Send Result to Client");
                                    }
                                    columns.insert(column_name.clone());
                                    found = Some((
                                        index,
                                        column_definition.name(),
                                        column_definition.sql_type(),
                                        TypeConstraint::from(&column_definition.sql_type()),
                                    ));
                                    break;
                                }
                            }

                            match found {
                                Some(index_col) => column_indices.push(index_col),
                                None => {
                                    sender
                                        .send(Err(QueryError::column_does_not_exist(column_name)))
                                        .expect("To Send Result to Client");
                                    has_error = true;
                                }
                            }
                        }

                        if has_error {
                            return Err(());
                        }

                        Ok(Plan::Update(TableUpdates {
                            table_id,
                            column_indices,
                            input,
                        }))
                    }
                }
            }
            Err(error) => {
                sender
                    .send(Err(QueryError::syntax_error(error)))
                    .expect("To Send Query Result to Client");
                Err(())
            }
        }
    }
}
