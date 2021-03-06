// Copyright 2020 - 2021 Alex Dukhno
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

use super::*;
use crate::transaction_manager::TransactionManager;
use postgre_sql::{
    query_ast::Request,
    query_parser::QueryParser,
    query_response::QueryError,
    wire_protocol::payload::{BIGINT, CHAR, INT, SMALLINT, VARCHAR},
};
use storage::Database;

#[cfg(test)]
mod delete;
#[cfg(test)]
mod insert;
#[cfg(test)]
mod prepared_statement;
#[cfg(test)]
mod schema;
#[cfg(test)]
mod select;
#[cfg(test)]
mod table;
#[cfg(test)]
mod type_constraints;
#[cfg(test)]
mod update;

fn small_int(value: i16) -> String {
    value.to_string()
}

fn integer(value: i32) -> String {
    value.to_string()
}

fn big_int(value: i64) -> String {
    value.to_string()
}

fn string(value: &str) -> String {
    value.to_owned()
}

fn assert_cached_statement(query_plan_cache: &mut QueryPlanCache, txn: &TransactionContext, sql: &str, expected: Vec<OutboundMessage>) {
    let executor = QueryExecutor;
    match QueryParser.parse(sql) {
        Ok(Request::Statement(statement)) => {
            assert_eq!(executor.execute_statement(statement, txn, query_plan_cache), expected);
        }
        other => panic!("expected DDL query but was {:?}", other),
    }
}

fn assert_statement(txn: &TransactionContext, sql: &str, expected: Vec<OutboundMessage>) {
    let executor = QueryExecutor;
    let mut query_plan_cache = QueryPlanCache::default();
    match QueryParser.parse(sql) {
        Ok(Request::Statement(statement)) => {
            assert_eq!(executor.execute_statement(statement, txn, &mut query_plan_cache), expected);
        }
        other => panic!("expected DDL query but was {:?}", other),
    }
}

#[rstest::fixture]
fn with_schema() -> TransactionManager {
    let database = Database::new("IN_MEMORY");
    let transaction_manager = TransactionManager::new(database);

    let txn = transaction_manager.start_transaction();
    assert_statement(
        &txn,
        "create schema schema_name",
        vec![OutboundMessage::SchemaCreated, OutboundMessage::ReadyForQuery],
    );
    txn.commit();
    transaction_manager
}
