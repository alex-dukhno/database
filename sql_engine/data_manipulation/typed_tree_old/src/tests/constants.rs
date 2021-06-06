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

#[test]
fn small_int() {
    assert_eq!(
        TypedTreeOld::Item(TypedItemOld::Const(TypedValueOld::Num {
            value: BigDecimal::from(0),
            type_family: SqlTypeFamilyOld::SmallInt,
        }))
        .eval(&[], &[]),
        Ok(ScalarValue::Num {
            value: BigDecimal::from(0),
            type_family: SqlTypeFamilyOld::SmallInt
        })
    );
}

#[test]
fn integer() {
    assert_eq!(
        TypedTreeOld::Item(TypedItemOld::Const(TypedValueOld::Num {
            value: BigDecimal::from(0),
            type_family: SqlTypeFamilyOld::Integer,
        }))
        .eval(&[], &[]),
        Ok(ScalarValue::Num {
            value: BigDecimal::from(0),
            type_family: SqlTypeFamilyOld::Integer
        })
    );
}

#[test]
fn big_int() {
    assert_eq!(
        TypedTreeOld::Item(TypedItemOld::Const(TypedValueOld::Num {
            value: BigDecimal::from(0),
            type_family: SqlTypeFamilyOld::BigInt,
        }))
        .eval(&[], &[]),
        Ok(ScalarValue::Num {
            value: BigDecimal::from(0),
            type_family: SqlTypeFamilyOld::BigInt
        })
    );
}

#[test]
fn bool() {
    assert_eq!(
        TypedTreeOld::Item(TypedItemOld::Const(TypedValueOld::Bool(true))).eval(&[], &[]),
        Ok(ScalarValue::Bool(true))
    );
}

#[test]
fn string() {
    assert_eq!(
        TypedTreeOld::Item(TypedItemOld::Const(TypedValueOld::String("str".to_owned()))).eval(&[], &[]),
        Ok(ScalarValue::String("str".to_owned()))
    );
}