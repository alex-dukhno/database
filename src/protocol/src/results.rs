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

use crate::{sql_types, sql_types::PostgreSqlType, Message};
use std::fmt::{self, Display, Formatter};

/// Represents result of SQL query execution
pub type QueryResult = std::result::Result<QueryEvent, QueryError>;
/// Represents selected data from tables
pub type Projection = (Vec<(String, sql_types::PostgreSqlType)>, Vec<Vec<String>>);

/// Represents successful events that can happen in server backend
#[derive(Debug, PartialEq)]
pub enum QueryEvent {
    /// Schema successfully created
    SchemaCreated,
    /// Schema successfully dropped
    SchemaDropped,
    /// Table successfully created
    TableCreated,
    /// Table successfully dropped
    TableDropped,
    /// Variable successfully set
    VariableSet,
    /// Transaction is started
    TransactionStarted,
    /// Number of records inserted into a table
    RecordsInserted(usize),
    /// Records selected from database
    RecordsSelected(Projection),
    /// Number of records updated into a table
    RecordsUpdated(usize),
    /// Number of records deleted into a table
    RecordsDeleted(usize),
}

/// Message severities
/// Reference: defined in https://www.postgresql.org/docs/12/protocol-error-fields.html
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum Severity {
    Error,
    Fatal,
    Panic,
    Warning,
    Notice,
    Debug,
    Info,
    Log,
}

// easy conversion into a string.
impl Into<String> for Severity {
    fn into(self) -> String {
        match self {
            Self::Error => "ERROR".to_string(),
            Self::Fatal => "FATAL".to_string(),
            Self::Panic => "PANIC".to_string(),
            Self::Warning => "WARNING".to_string(),
            Self::Notice => "NOTICE".to_string(),
            Self::Debug => "DEBUG".to_string(),
            Self::Info => "INFO".to_string(),
            Self::Log => "LOG".to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum QueryErrorKind {
    SchemaAlreadyExists(String),
    TableAlreadyExists(String),
    SchemaDoesNotExist(String),
    TableDoesNotExist(String),
    ColumnDoesNotExist(Vec<String>),
    NotSupportedOperation(String),
    TooManyInsertExpressions,

    NumericTypeOutOfRange(PostgreSqlType),
    DataTypeMismatch(PostgreSqlType, String),
    StringTypeLengthMismatch(PostgreSqlType, u64),
}

impl Display for QueryErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::SchemaAlreadyExists(schema_name) => write!(f, "schema \"{}\" already exists", schema_name),
            Self::TableAlreadyExists(table_name) => write!(f, "table \"{}\" already exists", table_name),
            Self::SchemaDoesNotExist(schema_name) => write!(f, "schema \"{}\" does not exist", schema_name),
            Self::TableDoesNotExist(table_name) => write!(f, "table \"{}\" does not exist", table_name),
            Self::ColumnDoesNotExist(columns) => {
                if columns.len() > 1 {
                    write!(f, "columns {} do not exist", columns.join(", "))
                } else {
                    write!(f, "column {} does not exist", columns[0])
                }
            }
            Self::NotSupportedOperation(raw_sql_query) => {
                write!(f, "Currently, Query '{}' can't be executed", raw_sql_query)
            }
            Self::TooManyInsertExpressions => write!(f, "INSERT has more epxressions then target columns"),
            Self::NumericTypeOutOfRange(pg_type) => write!(f, "{} out of range", pg_type),
            Self::DataTypeMismatch(pg_type, value) => {
                write!(f, "invalid input syntax for type {}: \"{}\"", pg_type, value)
            }
            Self::StringTypeLengthMismatch(pg_type, len) => write!(f, "value too long for type {}({})", pg_type, len),
        }
    }
}

/// Represents error during query execution
#[derive(Debug, PartialEq)]
pub(crate) struct QueryErrorInner {
    severity: Severity,
    code: String,
    kind: QueryErrorKind,
}

impl QueryErrorInner {
    fn code(&self) -> Option<String> {
        Some(self.code.clone())
    }

    fn severity(&self) -> Option<String> {
        Some(self.severity.into())
    }

    fn message(&self) -> Option<String> {
        Some(format!("{}", self.kind))
    }
}

/// a container of errors that occured during query execution
#[derive(Debug, PartialEq)]
pub struct QueryError {
    errors: Vec<QueryErrorInner>,
}

impl QueryError {
    pub(crate) fn new(errors: Vec<QueryErrorInner>) -> Self {
        Self { errors }
    }

    pub(crate) fn into_messages(self) -> Vec<Message> {
        self.errors
            .into_iter()
            .map(|inner| Message::ErrorResponse(inner.severity(), inner.code(), inner.message()))
            .collect::<Vec<_>>()
    }
}

/// a structure for building a QueryError
#[derive(Default, Debug)]
pub struct QueryErrorBuilder {
    errors: Vec<QueryErrorInner>,
}

impl QueryErrorBuilder {
    /// constructs a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    // I am not sure this is a good idea.
    /// helper for building errors in one line.
    pub fn build_with<Errs: FnMut(&mut Self)>(mut errs: Errs) -> QueryError {
        let mut builder = Self::new();
        errs(&mut builder);
        builder.build()
    }

    /// builds a QueryError containing all of the error generated
    pub fn build(self) -> QueryError {
        QueryError::new(self.errors)
    }

    // these error will stop the execution of the query; therefore there will only
    // ever be one.

    /// schema already exists error constructor
    pub fn schema_already_exists(mut self, schema_name: String) -> Self {
        self.errors.push(QueryErrorInner {
            severity: Severity::Error,
            code: "42P06".to_owned(),
            kind: QueryErrorKind::SchemaAlreadyExists(schema_name),
        });
        self
    }

    /// schema does not exist error constructor
    pub fn schema_does_not_exist(mut self, schema_name: String) -> Self {
        self.errors.push(QueryErrorInner {
            severity: Severity::Error,
            code: "3F000".to_owned(),
            kind: QueryErrorKind::SchemaDoesNotExist(schema_name),
        });
        self
    }

    /// table already exists error constructor
    pub fn table_already_exists(mut self, table_name: String) -> Self {
        self.errors.push(QueryErrorInner {
            severity: Severity::Error,
            code: "42P07".to_owned(),
            kind: QueryErrorKind::TableAlreadyExists(table_name),
        });
        self
    }

    /// table does not exist error constructor
    pub fn table_does_not_exist(mut self, table_name: String) -> Self {
        self.errors.push(QueryErrorInner {
            severity: Severity::Error,
            code: "42P01".to_owned(),
            kind: QueryErrorKind::TableDoesNotExist(table_name),
        });
        self
    }

    /// column does not exists error constructor
    pub fn column_does_not_exist(mut self, non_existing_columns: Vec<String>) -> Self {
        self.errors.push(QueryErrorInner {
            severity: Severity::Error,
            code: "42703".to_owned(),
            kind: QueryErrorKind::ColumnDoesNotExist(non_existing_columns),
        });
        self
    }

    /// not supported operation error constructor
    pub fn not_supported_operation(mut self, raw_sql_query: String) -> Self {
        self.errors.push(QueryErrorInner {
            severity: Severity::Error,
            code: "42601".to_owned(),
            kind: QueryErrorKind::NotSupportedOperation(raw_sql_query),
        });
        self
    }

    /// too many insert expressions errors constructors
    pub fn too_many_insert_expressions(mut self) -> Self {
        self.errors.push(QueryErrorInner {
            severity: Severity::Error,
            code: "42601".to_owned(),
            kind: QueryErrorKind::TooManyInsertExpressions,
        });
        self
    }

    // These errors can be generated multiple at a time which is why they are &mut self
    // and the rest are mut self.

    /// numeric out of range constructor
    pub fn out_of_range(&mut self, pg_type: PostgreSqlType) {
        self.errors.push(QueryErrorInner {
            severity: Severity::Error,
            code: "22003".to_owned(),
            kind: QueryErrorKind::NumericTypeOutOfRange(pg_type),
        });
    }

    /// type mismatch constructor
    pub fn type_mismatch(&mut self, value: &str, pg_type: PostgreSqlType) {
        self.errors.push(QueryErrorInner {
            severity: Severity::Error,
            code: "2200G".to_owned(),
            kind: QueryErrorKind::DataTypeMismatch(pg_type, value.to_owned()),
        });
    }

    /// length of string types do not match constructor
    pub fn string_length_mismatch(&mut self, pg_type: PostgreSqlType, len: u64) {
        self.errors.push(QueryErrorInner {
            severity: Severity::Error,
            code: "22026".to_owned(),
            kind: QueryErrorKind::StringTypeLengthMismatch(pg_type, len),
        });
    }
}
