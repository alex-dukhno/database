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

use types::SqlType;

pub type Id = u64;
pub type ParameterName = String;

#[derive(Debug, PartialEq, Clone)]
pub struct DeprecatedColumnDefinition {
    name: String,
    sql_type: SqlType,
}

impl DeprecatedColumnDefinition {
    pub fn new(name: &str, sql_type: SqlType) -> Self {
        Self {
            name: name.to_lowercase(),
            sql_type,
        }
    }

    pub fn sql_type(&self) -> SqlType {
        self.sql_type
    }

    pub fn has_name(&self, other_name: &str) -> bool {
        self.name == other_name
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }
}
