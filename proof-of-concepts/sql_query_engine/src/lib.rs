extern crate types;

use std::collections::{BTreeMap, HashMap};
use std::convert::TryFrom;
use std::fmt::{self, Debug, Display};
use std::ops::Deref;

use sqlparser::ast::{
    Assignment, BinaryOperator, Expr, Query, Select, SetExpr, Statement, TableFactor,
    TableWithJoins,
};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

use num_bigint::BigInt;
use serde::export::Formatter;
use types::{Type, TypeError};

pub type ExecutionResult = Result<EngineEvent, ErrorEvent>;

#[derive(Debug, PartialEq)]
pub enum EngineEvent {
    TableCreated(String),
    RecordInserted,
    RecordsSelected(Vec<Vec<u8>>),
    RecordsUpdated,
    RecordsDeleted,
}

#[derive(Debug, PartialEq)]
pub enum ErrorEvent {
    TableAlreadyExists(String),
    UnimplementedBranch(String),
    TableDoesNotExist(String),
}

impl Display for ErrorEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ErrorEvent::TableAlreadyExists(table_name) => write!(f, "{}", table_name),
            ErrorEvent::UnimplementedBranch(error) => write!(f, "{}", error),
            ErrorEvent::TableDoesNotExist(table_name) => write!(f, "{}", table_name),
        }
    }
}

pub struct Engine {
    dialect: GenericDialect,
    tables: HashMap<String, BTreeMap<BigInt, Vec<u8>>>,
}

impl Engine {
    #[allow(clippy::cognitive_complexity)]
    pub fn execute(&mut self, sql: String) -> ExecutionResult {
        let mut statements = match Parser::parse_sql(&self.dialect, sql) {
            Ok(ok) => ok,
            Err(error) => return Err(ErrorEvent::UnimplementedBranch(format!("{:?}", error))),
        };
        match statements.pop() {
            Some(Statement::CreateTable { name, .. }) => {
                let table_name = name.to_string();
                if self.tables.contains_key(&table_name) {
                    Err(ErrorEvent::TableAlreadyExists(table_name))
                } else {
                    self.tables.insert(table_name.clone(), BTreeMap::new());
                    Ok(EngineEvent::TableCreated(table_name))
                }
            }
            Some(Statement::Insert {
                table_name, source, ..
            }) => {
                let table_name = table_name.to_string();
                match self.tables.get_mut(&table_name) {
                    None => Err(ErrorEvent::TableDoesNotExist(table_name)),
                    Some(table) => {
                        let Query { body, .. } = &*source;
                        if let SetExpr::Values(values) = &body {
                            let values = &values.0;
                            if let Expr::Value(value) = &values[0][0] {
                                if let Ok(Type::Int(value)) = Type::try_from(value.clone()) {
                                    let encoded = bincode::serialize(&value).unwrap();
                                    table.insert(value, encoded);
                                    Ok(EngineEvent::RecordInserted)
                                } else {
                                    Err(
                                        ErrorEvent::UnimplementedBranch(
                                            format!(
                                                "UNIMPLEMENTED HANDLING OF STRING PARSING \n{:?}\n IN \"INSERT INTO <table> VALUES (v)\"",
                                                value
                                            )
                                        )
                                    )
                                }
                            } else {
                                Err(
                                    ErrorEvent::UnimplementedBranch(
                                        format!(
                                            "UNIMPLEMENTED HANDLING OF PARSING \n{:?}\n IN \"INSERT INTO <table> VALUES (v)\"",
                                            values
                                        )
                                    )
                                )
                            }
                        } else {
                            Err(ErrorEvent::UnimplementedBranch(format!(
                                "UNIMPLEMENTED HANDLING OF VALUES INSERTION \n{:?}\n",
                                source
                            )))
                        }
                    }
                }
            }
            Some(Statement::Update {
                table_name,
                assignments,
                selection,
            }) => {
                let table_name = table_name.to_string();
                match self.tables.get_mut(&table_name) {
                    None => Err(ErrorEvent::TableDoesNotExist(table_name)),
                    Some(table) => {
                        let keys = match selection {
                            Some(Expr::BinaryOp { right, .. }) => {
                                if let Expr::Value(value) = right.deref() {
                                    match Type::try_from(value.clone()) {
                                        Ok(Type::Int(value)) => vec![value],
                                        Ok(sql_type) => {
                                            return Err(ErrorEvent::UnimplementedBranch(format!(
                                                "{:?} is not supported yet",
                                                sql_type
                                            )))
                                        }
                                        Err(TypeError::Unsupported(message)) => {
                                            return Err(ErrorEvent::UnimplementedBranch(message))
                                        }
                                    }
                                } else {
                                    return Err(ErrorEvent::UnimplementedBranch(format!(
                                        "Non value RHS type {:?} is not supported",
                                        right
                                    )));
                                }
                            }
                            None => table.keys().cloned().collect::<Vec<BigInt>>(),
                            selection => {
                                return Err(ErrorEvent::UnimplementedBranch(format!(
                                    "UNIMPLEMENTED HANDLING OF \n{:?}\n WHERE CLAUSE!",
                                    selection
                                )))
                            }
                        };
                        let Assignment { value, .. } = &assignments[0];
                        let value = if let Expr::Value(value) = value {
                            match Type::try_from(value.clone()) {
                                Ok(Type::Int(value)) => value,
                                Ok(sql_type) => {
                                    return Err(ErrorEvent::UnimplementedBranch(format!(
                                        "{:?} is not supported yet",
                                        sql_type
                                    )))
                                }
                                Err(TypeError::Unsupported(message)) => {
                                    return Err(ErrorEvent::UnimplementedBranch(message))
                                }
                            }
                        } else {
                            return Err(ErrorEvent::UnimplementedBranch(format!(
                                "Non value RHS type {:?} is not supported",
                                value
                            )));
                        };
                        for key in keys {
                            if let Some(old_value) = table.get_mut(&key) {
                                *old_value = bincode::serialize(&value).unwrap();
                            }
                        }
                        Ok(EngineEvent::RecordsUpdated)
                    }
                }
            }
            Some(Statement::Delete {
                table_name,
                selection,
            }) => {
                let table_name = table_name.to_string();
                match self.tables.get_mut(&table_name) {
                    None => Err(ErrorEvent::TableDoesNotExist(table_name.to_string())),
                    Some(table) => {
                        let keys = match selection {
                            Some(Expr::BinaryOp { right, .. }) => {
                                if let Expr::Value(value) = right.deref() {
                                    match Type::try_from(value.clone()) {
                                        Ok(Type::Int(value)) => vec![value],
                                        Ok(sql_type) => {
                                            return Err(ErrorEvent::UnimplementedBranch(format!(
                                                "{:?} is not supported yet",
                                                sql_type
                                            )))
                                        }
                                        Err(TypeError::Unsupported(message)) => {
                                            return Err(ErrorEvent::UnimplementedBranch(message))
                                        }
                                    }
                                } else {
                                    return Err(ErrorEvent::UnimplementedBranch(format!(
                                        "Non value RHS type {:?} is not supported",
                                        right
                                    )));
                                }
                            }
                            None => table.keys().cloned().collect::<Vec<BigInt>>(),
                            selection => {
                                return Err(ErrorEvent::UnimplementedBranch(format!(
                                    "UNIMPLEMENTED HANDLING OF \n{:?}\n WHERE CLAUSE!",
                                    selection
                                )))
                            }
                        };
                        for key in keys {
                            table.remove(&key);
                        }
                        Ok(EngineEvent::RecordsDeleted)
                    }
                }
            }
            Some(Statement::Query(query)) => {
                let Query { body, .. } = &*query;
                if let SetExpr::Select(select) = &body {
                    let Select {
                        selection, from, ..
                    } = select.deref();
                    let TableWithJoins { relation, .. } = &from[0];
                    let table_name = match relation {
                        TableFactor::Table { name, .. } => name.to_string(),
                        _ => {
                            return Err(ErrorEvent::UnimplementedBranch(format!(
                                "UNIMPLEMENTED SELECTION FROM MULTIPLE TABLES \n{:?}\n",
                                relation
                            )))
                        }
                    };
                    match self.tables.get(&table_name) {
                        None => Err(ErrorEvent::TableDoesNotExist(table_name)),
                        Some(table) => match selection {
                            Some(Expr::BinaryOp { left: _, op, right }) => match op {
                                BinaryOperator::Eq => {
                                    if let Expr::Value(value) = right.deref() {
                                        if let Ok(Type::Int(value)) = Type::try_from(value.clone())
                                        {
                                            table.get(&value)
                                                        .ok_or_else(|| ErrorEvent::UnimplementedBranch("UNIMPLEMENTED HANDLING OF NO INSERTED VALUE".to_owned()))
                                                        .map(|record| EngineEvent::RecordsSelected(vec![record.clone()]))
                                        } else {
                                            return Err(
                                                        ErrorEvent::UnimplementedBranch(
                                                            format!(
                                                                "UNIMPLEMENTED HANDLING OF STRING PARSING \n{:?}\n IN WHERE X = RIGHT!",
                                                                right
                                                            )
                                                        )
                                                    );
                                        }
                                    } else {
                                        return Err(
                                                    ErrorEvent::UnimplementedBranch(
                                                        format!("UNIMPLEMENTED HANDLING OF \n{:?}\n IN WHERE X = RIGHT!", right)
                                                    )
                                                );
                                    }
                                }
                                operator => {
                                    return Err(ErrorEvent::UnimplementedBranch(format!(
                                    "UNIMPLEMENTED HANDLING OF OPERATOR \n{:?}\n IN WHERE CLAUSE",
                                    operator
                                )))
                                }
                            },
                            Some(Expr::Between {
                                negated, low, high, ..
                            }) => {
                                if let (Expr::Value(low), Expr::Value(high)) =
                                    (low.deref(), high.deref())
                                {
                                    if let (Ok(Type::Int(low)), Ok(Type::Int(high))) =
                                        (Type::try_from(low.clone()), Type::try_from(high.clone()))
                                    {
                                        if *negated {
                                            Ok(EngineEvent::RecordsSelected(
                                                table
                                                    .range(..low)
                                                    .chain(table.range(high..).skip(1))
                                                    .map(|(_key, value)| value)
                                                    .cloned()
                                                    .collect(),
                                            ))
                                        } else {
                                            Ok(EngineEvent::RecordsSelected(
                                                table
                                                    .range(low..=high)
                                                    .map(|(_key, value)| value)
                                                    .cloned()
                                                    .collect(),
                                            ))
                                        }
                                    } else {
                                        return Err(
                                                ErrorEvent::UnimplementedBranch(
                                                    format!(
                                                        "UNIMPLEMENTED HANDLING OF STRING PARSING \n IN WHERE BETWEEN {:?} AND {:?}",
                                                        low, high
                                                    )
                                                )
                                            );
                                    }
                                } else {
                                    return Err(
                                            ErrorEvent::UnimplementedBranch(
                                                format!("UNIMPLEMENTED HANDLING OF \n IN WHERE BETWEEN {:?} AND {:?}", low, high)
                                            )
                                        );
                                }
                            }
                            Some(Expr::InList { list, negated, .. }) => {
                                let mut records = vec![];
                                let mut set = Vec::new();
                                for item in list {
                                    if let Expr::Value(value) = item {
                                        if let Ok(Type::Int(value)) = Type::try_from(value.clone())
                                        {
                                            set.push(value)
                                        } else {
                                            return Err(
                                                    ErrorEvent::UnimplementedBranch(
                                                        format!("UNIMPLEMENTED HANDLING OF STRING PARSING IN WHERE 'IN (x, y, z)' for {:?}", value)
                                                    )
                                                );
                                        }
                                    } else {
                                        return Err(
                                                ErrorEvent::UnimplementedBranch(
                                                    format!("UNIMPLEMENTED HANDLING OF VALUES PARSING IN WHERE 'IN (x, y, z)' for {:?}", item)
                                                )
                                            );
                                    }
                                }
                                for (key, record) in table.iter() {
                                    if !*negated && set.contains(key) {
                                        records.push(record.clone())
                                    }
                                    if *negated && !set.contains(key) {
                                        records.push(record.clone())
                                    }
                                }
                                Ok(EngineEvent::RecordsSelected(records))
                            }
                            None => {
                                let copy = table.values().cloned().collect();
                                Ok(EngineEvent::RecordsSelected(copy))
                            }
                            selection => {
                                return Err(ErrorEvent::UnimplementedBranch(format!(
                                    "UNIMPLEMENTED HANDLING OF \n{:?}\n WHERE CLAUSE!",
                                    selection
                                )))
                            }
                        },
                    }
                } else {
                    return Err(ErrorEvent::UnimplementedBranch(format!(
                        "UNIMPLEMENTED HANDLING OF \n{:?}\n SELECT QUERY!",
                        query
                    )));
                }
            }
            statement => {
                return Err(ErrorEvent::UnimplementedBranch(format!(
                    "UNIMPLEMENTED HANDLING OF \n{:?}\n STATEMENT!",
                    statement
                )))
            }
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            dialect: GenericDialect {},
            tables: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod data_definition_language {
        use super::*;

        #[test]
        fn create_two_tables() {
            let mut engine = Engine::default();

            assert_eq!(
                engine.execute(
                    "CREATE TABLE simple_table (\n\
              int_column INT,\n\
          );"
                    .to_owned()
                ),
                Ok(EngineEvent::TableCreated("simple_table".to_owned()))
            );

            assert_eq!(
                engine.execute(
                    "CREATE TABLE another_table (\n\
              int_column INT,\n\
          );"
                    .to_owned()
                ),
                Ok(EngineEvent::TableCreated("another_table".to_owned()))
            );
        }

        #[test]
        fn error_when_trying_to_create_table_with_existing_name() {
            let mut engine = Engine::default();

            assert_eq!(
                engine.execute(
                    "CREATE TABLE simple_table (\n\
            int_column INT,\n\
          );"
                    .to_owned()
                ),
                Ok(EngineEvent::TableCreated("simple_table".to_owned()))
            );

            assert_eq!(
                engine.execute(
                    "CREATE TABLE simple_table (\n\
            int_column INT,\n\
          );"
                    .to_owned()
                ),
                Err(ErrorEvent::TableAlreadyExists("simple_table".to_owned()))
            );
        }

        #[test]
        fn create_table_if_not_exists() {
            let mut engine = Engine::default();

            assert_eq!(
                engine.execute(
                    "CREATE TABLE IF NOT EXISTS simple_table (\n\
            int_column INT,\n\
          );"
                    .to_owned()
                ),
                Ok(EngineEvent::TableCreated("simple_table".to_owned()))
            );
        }
    }

    #[cfg(test)]
    mod data_manipulation_language {
        use std::fmt::Display;

        use num_bigint::BigInt;

        use super::*;

        const TABLE_NAME: &'static str = "simple_table";
        const COLUMN_NAME: &'static str = "int_column";

        #[allow(unused_must_use)]
        fn create_table(engine: &mut Engine) {
            engine.execute(format!(
                "CREATE TABLE {} ({} INT);",
                TABLE_NAME, COLUMN_NAME
            ));
        }

        fn insert_value<V: Display>(engine: &mut Engine, value: V) -> ExecutionResult {
            engine.execute(format!("INSERT INTO {} VALUES ({});", TABLE_NAME, value))
        }

        fn select_value<V: Display>(engine: &mut Engine, value: V) -> ExecutionResult {
            engine.execute(format!(
                "SELECT {0} FROM {1} WHERE {0} = {2};",
                COLUMN_NAME, TABLE_NAME, value
            ))
        }

        fn select_all(engine: &mut Engine) -> ExecutionResult {
            engine.execute(format!("SELECT {} FROM {};", COLUMN_NAME, TABLE_NAME))
        }

        fn select_between<V: Display>(engine: &mut Engine, from: V, to: V) -> ExecutionResult {
            engine.execute(format!(
                "SELECT {0} FROM {1} WHERE {0} BETWEEN {2} AND {3}",
                COLUMN_NAME, TABLE_NAME, from, to
            ))
        }

        fn select_not_between<V: Display>(engine: &mut Engine, from: V, to: V) -> ExecutionResult {
            engine.execute(format!(
                "SELECT {0} FROM {1} WHERE {0} NOT BETWEEN {2} AND {3}",
                COLUMN_NAME, TABLE_NAME, from, to
            ))
        }

        fn select_in<V: Display>(engine: &mut Engine, one: V, two: V, three: V) -> ExecutionResult {
            engine.execute(format!(
                "SELECT {0} FROM {1} WHERE {0} IN ({2}, {3}, {4});",
                COLUMN_NAME, TABLE_NAME, one, two, three
            ))
        }

        fn select_not_in<V: Display>(
            engine: &mut Engine,
            one: V,
            two: V,
            three: V,
        ) -> ExecutionResult {
            engine.execute(format!(
                "SELECT {0} FROM {1} WHERE {0} NOT IN ({2}, {3}, {4});",
                COLUMN_NAME, TABLE_NAME, one, two, three
            ))
        }

        fn select_with_and<V: Display>(engine: &mut Engine, one: V, two: V) -> ExecutionResult {
            engine.execute(format!(
                "SELECT {0} FROM {1} WHERE {0} = {2} AND {0} = {3};",
                COLUMN_NAME, TABLE_NAME, one, two
            ))
        }

        fn select_with_or<V: Display>(engine: &mut Engine, one: V, two: V) -> ExecutionResult {
            engine.execute(format!(
                "SELECT {0} FROM {1} WHERE {0} = {2} OR {0} = {3};",
                COLUMN_NAME, TABLE_NAME, one, two
            ))
        }

        fn update_value<V: Display>(engine: &mut Engine, from: V, to: V) -> ExecutionResult {
            engine.execute(format!(
                "UPDATE {0} SET {1} = {2} where {1} = {3}",
                TABLE_NAME, COLUMN_NAME, from, to
            ))
        }

        fn update_all(engine: &mut Engine) -> ExecutionResult {
            engine.execute(format!("UPDATE {0} SET {1} = 100", TABLE_NAME, COLUMN_NAME))
        }

        fn delete_value<V: Display>(engine: &mut Engine, value: V) -> ExecutionResult {
            engine.execute(format!(
                "DELETE FROM {} WHERE {} = {}",
                TABLE_NAME, COLUMN_NAME, value
            ))
        }

        fn delete_all(engine: &mut Engine) -> ExecutionResult {
            engine.execute(format!("DELETE FROM {}", TABLE_NAME))
        }

        fn int(val: i32) -> Vec<u8> {
            bincode::serialize(&BigInt::from(val)).unwrap()
        }

        #[test]
        fn insert_into_not_existed_table() {
            let mut engine = Engine::default();

            assert_eq!(
                insert_value(&mut engine, 1),
                Err(ErrorEvent::TableDoesNotExist(TABLE_NAME.to_owned()))
            )
        }

        #[test]
        fn select_from_not_existed_table() {
            let mut engine = Engine::default();

            assert_eq!(
                select_all(&mut engine),
                Err(ErrorEvent::TableDoesNotExist(TABLE_NAME.to_owned()))
            )
        }

        #[test]
        fn update_from_not_existed_table() {
            let mut engine = Engine::default();

            assert_eq!(
                update_all(&mut engine),
                Err(ErrorEvent::TableDoesNotExist(TABLE_NAME.to_owned()))
            )
        }

        #[test]
        fn delete_from_not_existed_table() {
            let mut engine = Engine::default();

            assert_eq!(
                delete_all(&mut engine),
                Err(ErrorEvent::TableDoesNotExist(TABLE_NAME.to_owned()))
            )
        }

        #[test]
        fn insert_select_single_record() {
            let mut engine = Engine::default();
            create_table(&mut engine);

            assert_eq!(
                insert_value(&mut engine, 1),
                Ok(EngineEvent::RecordInserted)
            );

            assert_eq!(
                select_value(&mut engine, 1),
                Ok(EngineEvent::RecordsSelected(vec![int(1)]))
            );
        }

        #[test]
        fn insert_many_select_single_record() {
            let mut engine = Engine::default();
            create_table(&mut engine);

            assert_eq!(
                insert_value(&mut engine, 1),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 2),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 3),
                Ok(EngineEvent::RecordInserted)
            );

            assert_eq!(
                select_value(&mut engine, 2),
                Ok(EngineEvent::RecordsSelected(vec![int(2)]))
            );
        }

        #[test]
        fn insert_many_select_all_records() {
            let mut engine = Engine::default();
            create_table(&mut engine);

            assert_eq!(
                insert_value(&mut engine, 1),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 2),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 3),
                Ok(EngineEvent::RecordInserted)
            );

            assert_eq!(
                select_all(&mut engine),
                Ok(EngineEvent::RecordsSelected(vec![int(1), int(2), int(3)]))
            );
        }

        #[test]
        fn update_single_value() {
            let mut engine = Engine::default();
            create_table(&mut engine);

            assert_eq!(
                insert_value(&mut engine, 1),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 2),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 3),
                Ok(EngineEvent::RecordInserted)
            );

            assert_eq!(
                update_value(&mut engine, 4, 2),
                Ok(EngineEvent::RecordsUpdated)
            );
            assert_eq!(
                select_all(&mut engine),
                Ok(EngineEvent::RecordsSelected(vec![int(1), int(4), int(3)]))
            );
        }

        #[test]
        fn update_all_values() {
            let mut engine = Engine::default();
            create_table(&mut engine);

            assert_eq!(
                insert_value(&mut engine, 1),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 2),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 3),
                Ok(EngineEvent::RecordInserted)
            );

            assert_eq!(update_all(&mut engine), Ok(EngineEvent::RecordsUpdated));
            assert_eq!(
                select_all(&mut engine),
                Ok(EngineEvent::RecordsSelected(vec![
                    int(100),
                    int(100),
                    int(100)
                ]))
            );
        }

        #[test]
        fn delete_single_value() {
            let mut engine = Engine::default();
            create_table(&mut engine);

            assert_eq!(
                insert_value(&mut engine, 1),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 2),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 3),
                Ok(EngineEvent::RecordInserted)
            );

            assert_eq!(
                delete_value(&mut engine, 2),
                Ok(EngineEvent::RecordsDeleted)
            );
            assert_eq!(
                select_all(&mut engine),
                Ok(EngineEvent::RecordsSelected(vec![int(1), int(3)]))
            );
        }

        #[test]
        fn delete_all_values() {
            let mut engine = Engine::default();
            create_table(&mut engine);

            assert_eq!(
                insert_value(&mut engine, 1),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 2),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 3),
                Ok(EngineEvent::RecordInserted)
            );

            assert_eq!(delete_all(&mut engine), Ok(EngineEvent::RecordsDeleted));
            assert_eq!(
                select_all(&mut engine),
                Ok(EngineEvent::RecordsSelected(vec![]))
            );
        }

        #[test]
        fn select_in_range() {
            let mut engine = Engine::default();
            create_table(&mut engine);

            assert_eq!(
                insert_value(&mut engine, 1),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 2),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 3),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 4),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 5),
                Ok(EngineEvent::RecordInserted)
            );

            assert_eq!(
                select_between(&mut engine, 2, 4),
                Ok(EngineEvent::RecordsSelected(vec![int(2), int(3), int(4)]))
            );
        }

        #[test]
        fn select_out_range() {
            let mut engine = Engine::default();
            create_table(&mut engine);

            assert_eq!(
                insert_value(&mut engine, 1),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 2),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 3),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 4),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 5),
                Ok(EngineEvent::RecordInserted)
            );

            assert_eq!(
                select_not_between(&mut engine, 2, 4),
                Ok(EngineEvent::RecordsSelected(vec![int(1), int(5)]))
            );
        }

        #[test]
        fn select_in_enumeration() {
            let mut engine = Engine::default();
            create_table(&mut engine);

            assert_eq!(
                insert_value(&mut engine, 1),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 2),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 3),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 4),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 5),
                Ok(EngineEvent::RecordInserted)
            );

            assert_eq!(
                select_in(&mut engine, 1, 3, 5),
                Ok(EngineEvent::RecordsSelected(vec![int(1), int(3), int(5)]))
            )
        }

        #[test]
        fn select_out_of_enumeration() {
            let mut engine = Engine::default();
            create_table(&mut engine);

            assert_eq!(
                insert_value(&mut engine, 1),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 2),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 3),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 4),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 5),
                Ok(EngineEvent::RecordInserted)
            );

            assert_eq!(
                select_not_in(&mut engine, 1, 3, 5),
                Ok(EngineEvent::RecordsSelected(vec![int(2), int(4)]))
            )
        }

        #[ignore]
        #[test]
        fn select_with_and_predicate() {
            let mut engine = Engine::default();
            create_table(&mut engine);

            assert_eq!(
                insert_value(&mut engine, 1),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 2),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 3),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 4),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 5),
                Ok(EngineEvent::RecordInserted)
            );

            assert_eq!(
                select_with_and(&mut engine, 1, 3),
                Ok(EngineEvent::RecordsSelected(vec![]))
            )
        }

        #[ignore]
        #[test]
        fn select_with_or_predicate() {
            let mut engine = Engine::default();
            create_table(&mut engine);

            assert_eq!(
                insert_value(&mut engine, 1),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 2),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 3),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 4),
                Ok(EngineEvent::RecordInserted)
            );
            assert_eq!(
                insert_value(&mut engine, 5),
                Ok(EngineEvent::RecordInserted)
            );

            assert_eq!(
                select_with_or(&mut engine, 1, 3),
                Ok(EngineEvent::RecordsSelected(vec![int(1), int(3)]))
            )
        }
    }
}
