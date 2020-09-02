use crate::query::scalar::{ScalarOp};
use crate::{ColumnDefinition, TableDefinition};
use protocol::results::{QueryErrorBuilder, QueryResult};
use protocol::Sender;
use representation::{Datum, EvalError, ScalarType};
use sqlparser::ast::{Assignment, BinaryOperator, DataType, Expr, UnaryOperator, Value};
use std::convert::TryFrom;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;
use sql_types::SqlType;
use kernel::SystemErrorKind::SqlEngineBug;

pub(crate) struct ExpressionEvaluation {
    session: Arc<dyn Sender>,
    table_info: Vec<TableDefinition>,
}

impl ExpressionEvaluation {
    pub(crate) fn new(session: Arc<dyn Sender>, table_info: Vec<TableDefinition>) -> ExpressionEvaluation {
        ExpressionEvaluation { session, table_info }
    }

    pub(crate) fn eval(&self, expr: &Expr) -> Result<ScalarOp, ()> {
        self.inner_eval(expr)
    }

    fn inner_eval(&self, expr: &Expr) -> Result<ScalarOp, ()> {
        match expr {
            Expr::Cast { expr, data_type } => match (&**expr, data_type) {
                (Expr::Value(Value::SingleQuotedString(v)), DataType::Boolean) => {
                    Ok(ScalarOp::Literal(Datum::from_bool(bool::from_str(v).unwrap())))
                }
                (Expr::Value(Value::Boolean(val)), DataType::Boolean) => Ok(ScalarOp::Literal(Datum::from_bool(*val))),
                _ => {
                    self.session
                        .send(Err(QueryErrorBuilder::new()
                            .syntax_error(format!(
                                "Cast from {:?} to {:?} is not currently supported",
                                expr, data_type
                            ))
                            .build()))
                        .expect("To Send Query Result to Client");
                    return Err(());
                }
            },
            Expr::UnaryOp { op, expr } => {
                // let operand = self.inner_eval(expr.deref())?;
                match (op, expr.deref()) {
                    (UnaryOperator::Minus, Expr::Value(Value::Number(value))) => {
                        match Datum::try_from(&Value::Number(-value)) {
                            Ok(datum) => Ok(ScalarOp::Literal(datum)),
                            Err(e) => {
                                let err = match e {
                                    EvalError::UnsupportedDatum(ty) => QueryErrorBuilder::new()
                                        .feature_not_supported(format!("Data type not supported: {}", ty))
                                        .build(),
                                    EvalError::OutOfRangeNumeric(ty) => {
                                        let mut builder = QueryErrorBuilder::new();
                                        builder.out_of_range(ty.to_pg_types(), String::new(), 0);
                                        builder.build()
                                    }
                                    EvalError::UnsupportedOperation => QueryErrorBuilder::new()
                                        .feature_not_supported("Use of unsupported expression feature".to_string())
                                        .build(),
                                };

                                self.session.send(Err(err)).expect("To Send Query Result to Client");
                                Err(())
                            }
                        }
                    }
                    (op, operand) => {
                        self.session
                            .send(Err(QueryErrorBuilder::new()
                                .syntax_error(op.to_string() + expr.to_string().as_str())
                                .build()))
                            .expect("To Send Query Result to Client");
                        // EvalScalarOp::eval_unary_literal_expr(op, *op, operand)?;
                        return Err(());
                    }
                }
            }
            Expr::BinaryOp { op, left, right } => {
                let lhs = self.inner_eval(left.deref())?;
                let rhs = self.inner_eval(right.deref())?;
                if let Some(ty) = self.compatible_types_for_op(op.clone(), lhs.scalar_type(), rhs.scalar_type()) {
                    match (lhs, rhs) {
                        (ScalarOp::Literal(left), ScalarOp::Literal(right)) => {
                            EvalScalarOp::eval_binary_literal_expr(self.session.as_ref(), op.clone(), left, right)
                                .map(ScalarOp::Literal)
                        }
                        (left, right) => {
                            Ok(ScalarOp::Binary(op.clone(), Box::new(left), Box::new(right), ty))
                        }
                    }
                }
                else {
                    let kind = QueryErrorBuilder::new()
                        .undefined_function(
                            op.to_string(),
                            lhs.scalar_type().to_string(),
                            rhs.scalar_type().to_string(),
                        )
                        .build();
                    self.session.send(Err(kind)).expect("To Senc Query Result to Client");
                    Err(())
                }
            }
            Expr::Value(value) => match Datum::try_from(value) {
                Ok(datum) => Ok(ScalarOp::Literal(datum)),
                Err(e) => {
                    let err = match e {
                        EvalError::UnsupportedDatum(ty) => QueryErrorBuilder::new()
                            .feature_not_supported(format!("Data type not supported: {}", ty))
                            .build(),
                        EvalError::OutOfRangeNumeric(ty) => {
                            let mut builder = QueryErrorBuilder::new();
                            builder.out_of_range(ty.to_pg_types(), String::new(), 0);
                            builder.build()
                        }
                        EvalError::UnsupportedOperation => QueryErrorBuilder::new()
                            .feature_not_supported("Use of unsupported expression feature".to_string())
                            .build(),
                    };

                    self.session.send(Err(err)).expect("To Send Query Result to Client");
                    Err(())
                }
            },
            Expr::Identifier(ident) => {
                if let Some((idx, column_def)) = self.find_column_by_name(ident.value.as_str())? {
                    let scalar_type = column_def.sql_type();
                    Ok(ScalarOp::Column(idx, Self::convert_sql_type(scalar_type)))
                } else {
                    self.session
                        .send(Err(QueryErrorBuilder::new()
                            .undefined_column(ident.value.clone())
                            .build()))
                        .expect("To Send Query Result to Client");
                    Err(())
                }
            }
            Expr::CompoundIdentifier(idents) => {
                self.session
                    .send(Err(QueryErrorBuilder::new().syntax_error(String::new()).build()))
                    .expect("To Send Query Result to Client");
                Err(())
            }
            _ => {
                self.session
                    .send(Err(QueryErrorBuilder::new().syntax_error(expr.to_string()).build()))
                    .expect("To Send Query Result to Client");
                Err(())
            }
        }
    }

    pub fn eval_assignment(&self, assignment: &Assignment) -> Result<ScalarOp, ()> {
        let Assignment { id, value } = assignment;
        let (destination, column_def) = if let Some((idx, def)) = self.find_column_by_name(id.value.as_str())? {
            (idx, def)
        } else {
            let kind = QueryErrorBuilder::new().undefined_column(id.value.clone()).build();
            self.session.send(Err(kind)).expect("To Send Query Result to Client");
            return Err(());
        };

        let value = self.eval(value)?;
        let ty = value.scalar_type();

        Ok(ScalarOp::Assignment {
            destination,
            value: Box::new(value),
            ty
        })
    }

    pub fn find_column_by_name(&self, name: &str) -> Result<Option<(usize, ColumnDefinition)>, ()> {
        let mut found = None;
        for table_info in self.table_info.to_vec() {
            if let Some((idx, column)) = table_info.column_by_name_with_index(name) {
                if found.is_some() {
                    let kind = QueryErrorBuilder::new().ambiguous_column(name.to_string()).build();
                    self.session.send(Err(kind)).expect("To Send Query Result to Client");
                    return Err(());
                } else {
                    found = Some((idx, column));
                }
            }
        }
        Ok(found)
    }

    pub fn compatible_types_for_op(&self, op: BinaryOperator, lhs_type: ScalarType, rhs_type: ScalarType) -> Option<ScalarType> {
        if lhs_type == rhs_type {
            if lhs_type.is_integer() {
                match op {
                    BinaryOperator::Plus |
                    BinaryOperator::Minus |
                    BinaryOperator::Multiply |
                    BinaryOperator::Divide |
                    BinaryOperator::Modulus |
                    BinaryOperator::BitwiseAnd |
                    BinaryOperator::BitwiseOr => Some(lhs_type),
                    BinaryOperator::StringConcat => Some(ScalarType::String),
                    _ => None
                }
            }
            else if lhs_type.is_float() {
                match op {
                    BinaryOperator::Plus |
                    BinaryOperator::Minus |
                    BinaryOperator::Multiply |
                    BinaryOperator::Divide => Some(lhs_type),
                    _ => None,
                }
            }
            else if lhs_type.is_string() {
                match op {
                    BinaryOperator::StringConcat => Some(ScalarType::String),
                    _ => None,
                }
            }
            else {
                None
            }
        }
        else if lhs_type.is_string() && rhs_type.is_integer() {
            match op {
                BinaryOperator::StringConcat => Some(ScalarType::String),
                _ => None
            }
        }
        else {
            None
        }
    }

    fn convert_sql_type(sql_type: SqlType) -> ScalarType {
        match sql_type {
            SqlType::Bool => ScalarType::Boolean,
            SqlType::Char(_) |
            SqlType::VarChar(_) => ScalarType::String,
            SqlType::SmallInt(_) => ScalarType::Int16,
            SqlType::Integer(_) => ScalarType::Int32,
            SqlType::BigInt(_) => ScalarType::Int64,
            SqlType::Real => ScalarType::Float32,
            SqlType::DoublePrecision => ScalarType::Float64,
            SqlType::Time |
            SqlType::TimeWithTimeZone |
            SqlType::Timestamp |
            SqlType::TimestampWithTimeZone |
            SqlType::Date |
            SqlType::Interval |
            SqlType::Decimal => panic!(),
        }
    }

}

pub struct EvalScalarOp;

impl EvalScalarOp {
    pub fn eval<'a, 'b: 'a>(session: &dyn Sender, row: &[Datum<'a>], eval: &ScalarOp) -> Result<Datum<'a>, ()> {
        match eval {
            ScalarOp::Column(idx, _) => Ok(row[*idx].clone()),
            ScalarOp::Literal(datum) => Ok(datum.clone()),
            ScalarOp::Binary(op, lhs, rhs, _) => {
                let left = Self::eval(session, row, lhs.as_ref())?;
                let right = Self::eval(session, row, rhs.as_ref())?;
                Self::eval_binary_literal_expr(session, op.clone(), left, right)
            }
            ScalarOp::Unary(op, operand, _) => {
                let operand = Self::eval(session, row, operand.as_ref())?;
                Self::eval_unary_literal_expr(session, op.clone(), operand)
            }
            ScalarOp::Assignment { .. } => {
                panic!("EvalScalarOp:eval should not be evaluated on a ScalarOp::Assignment")
            }
        }
    }

    pub fn eval_on_row(session: &dyn Sender, row: &mut [Datum], eval: &ScalarOp) -> Result<(), ()> {
        match eval {
            ScalarOp::Assignment { destination, value, ty: _} => {
                let value = Self::eval(session, row, value.as_ref())?;
                row[*destination] = value;
            }
            _ => {
                panic!("EvalScalarOp:eval_on_row should only be evaluated on a ScalarOp::Assignment");
            }
        }
        Ok(())
    }

    pub fn eval_binary_literal_expr<'b>(
        session: &dyn Sender,
        op: BinaryOperator,
        left: Datum<'b>,
        right: Datum<'b>,
    ) -> Result<Datum<'b>, ()> {
        if left.is_integer() && right.is_integer() {
            match op {
                BinaryOperator::Plus => Ok(left + right),
                BinaryOperator::Minus => Ok(left - right),
                BinaryOperator::Multiply => Ok(left * right),
                BinaryOperator::Divide => Ok(left / right),
                BinaryOperator::Modulus => Ok(left % right),
                BinaryOperator::BitwiseAnd => Ok(left & right),
                BinaryOperator::BitwiseOr => Ok(left | right),
                BinaryOperator::StringConcat => {
                    let kind = QueryErrorBuilder::new()
                        .undefined_function(op.to_string(), "NUMBER".to_owned(), "NUMBER".to_owned())
                        .build();
                    session.send(Err(kind)).expect("To Send Query Result to Client");
                    return Err(());
                }
                _ => panic!(),
            }
        } else if left.is_float() && right.is_float() {
            match op {
                BinaryOperator::Plus => Ok(left + right),
                BinaryOperator::Minus => Ok(left - right),
                BinaryOperator::Multiply => Ok(left * right),
                BinaryOperator::Divide => Ok(left / right),
                BinaryOperator::StringConcat => {
                    let kind = QueryErrorBuilder::new()
                        .undefined_function(op.to_string(), "NUMBER".to_owned(), "NUMBER".to_owned())
                        .build();
                    session.send(Err(kind)).expect("To Send Query Result to Client");
                    return Err(());
                }
                _ => panic!(),
            }
        } else if left.is_string() || right.is_string() {
            match op {
                BinaryOperator::StringConcat => {
                    let value = format!("{}{}", left.to_string(), right.to_string());
                    Ok(Datum::OwnedString(value))
                }
                _ => {
                    let kind = QueryErrorBuilder::new()
                        .undefined_function(op.to_string(), "STRING".to_owned(), "STRING".to_owned())
                        .build();
                    session.send(Err(kind)).expect("To Send Query Result to Client");
                    return Err(());
                }
            }
        } else {
            session
                .send(Err(QueryErrorBuilder::new()
                    .syntax_error(format!("{} {} {}", left.to_string(), op.to_string(), right.to_string()))
                    .build()))
                .expect("To Send Query Result to Client");
            Err(())
        }
    }

    pub fn eval_unary_literal_expr<'b>(
        session: &dyn Sender,
        op: UnaryOperator,
        operand: Datum,
    ) -> Result<Datum<'b>, ()> {
        unimplemented!()
    }
}
