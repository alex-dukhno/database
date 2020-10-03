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
use data_manager::{DataManager, MetadataView};
use plan::{FullTableName, Plan, TableId, TableInserts};
use protocol::{results::QueryError, Sender};
use sqlparser::ast::{Ident, ObjectName, Query, SetExpr};
use std::{collections::HashSet, convert::TryFrom, sync::Arc};

pub(crate) struct InsertPlanner<'ip> {
    table_name: &'ip ObjectName,
    columns: &'ip [Ident],
    source: &'ip Query,
}

impl<'ip> InsertPlanner<'ip> {
    pub(crate) fn new(table_name: &'ip ObjectName, columns: &'ip [Ident], source: &'ip Query) -> InsertPlanner<'ip> {
        InsertPlanner {
            table_name,
            columns,
            source,
        }
    }
}

impl Planner for InsertPlanner<'_> {
    fn plan(self, data_manager: Arc<DataManager>, sender: Arc<dyn Sender>) -> Result<Plan> {
        match FullTableName::try_from(self.table_name) {
            Ok(full_table_name) => {
                let (schema_name, table_name) = full_table_name.as_tuple();
                match data_manager.table_exists(&schema_name, &table_name) {
                    None => {
                        sender
                            .send(Err(QueryError::schema_does_not_exist(schema_name.to_owned())))
                            .expect("To Send Query Result to Client");
                        Err(())
                    }
                    Some((_, None)) => {
                        sender
                            .send(Err(QueryError::table_does_not_exist(format!(
                                "{}.{}",
                                schema_name, table_name
                            ))))
                            .expect("To Send Query Result to Client");
                        Err(())
                    }
                    Some((schema_id, Some(table_id))) => {
                        let Query { body, .. } = &self.source;
                        match body {
                            SetExpr::Values(values) => {
                                let table_id = TableId::from((schema_id, table_id));
                                let mut input = vec![];
                                for row in values.0.iter() {
                                    let mut scalar_values = vec![];
                                    for value in row {
                                        match ScalarOp::transform(&value) {
                                            Ok(Ok(value)) => scalar_values.push(value),
                                            Ok(Err(error)) => {
                                                sender
                                                    .send(Err(QueryError::syntax_error(error)))
                                                    .expect("To Send Result to Client");
                                                return Err(());
                                            }
                                            Err(error) => {
                                                sender
                                                    .send(Err(QueryError::feature_not_supported(error)))
                                                    .expect("To Send Result to Client");
                                                return Err(());
                                            }
                                        }
                                    }
                                    input.push(scalar_values);
                                }
                                let all_columns = data_manager.table_columns(&table_id).expect("Ok");
                                let column_indices = if self.columns.is_empty() {
                                    all_columns
                                        .iter()
                                        .cloned()
                                        .enumerate()
                                        .map(|(index, col_def)| {
                                            (
                                                index,
                                                col_def.name(),
                                                col_def.sql_type(),
                                                TypeConstraint::from(&col_def.sql_type()),
                                            )
                                        })
                                        .collect::<Vec<_>>()
                                } else {
                                    let mut columns = HashSet::new();
                                    let mut index_cols = vec![];
                                    let mut has_error = false;
                                    for col_name in self.columns.iter().map(|id| id.value.as_str()) {
                                        let column_name = col_name.to_lowercase();
                                        let mut found = None;
                                        for (index, column_definition) in all_columns.iter().enumerate() {
                                            if column_definition.has_name(&column_name) {
                                                if columns.contains(&column_name) {
                                                    sender
                                                        .send(Err(QueryError::duplicate_column(&column_name)))
                                                        .expect("To Send Result to Client");
                                                    has_error = true;
                                                }
                                                columns.insert(column_name.clone());
                                                found = Some((
                                                    index,
                                                    column_name.clone(),
                                                    column_definition.sql_type(),
                                                    TypeConstraint::from(&column_definition.sql_type()),
                                                ));
                                                break;
                                            }
                                        }

                                        match found {
                                            Some(index_col) => index_cols.push(index_col),
                                            None => {
                                                sender
                                                    .send(Err(QueryError::column_does_not_exist(column_name.clone())))
                                                    .expect("To Send Result to Client");
                                                has_error = true;
                                            }
                                        }
                                    }

                                    if has_error {
                                        return Err(());
                                    }

                                    index_cols
                                };
                                Ok(Plan::Insert(TableInserts {
                                    table_id,
                                    column_indices,
                                    input,
                                }))
                            }
                            set_expr => {
                                sender
                                    .send(Err(QueryError::syntax_error(format!("{} is not supported", set_expr))))
                                    .expect("To Send Query Result to Client");
                                Err(())
                            }
                        }
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
