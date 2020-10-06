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

use super::*;

#[cfg(test)]
mod persistence;
#[cfg(test)]
mod queries;
#[cfg(test)]
mod schema;
#[cfg(test)]
mod table;

const SCHEMA: &str = "schema_name";
const SCHEMA_1: &str = "schema_name_1";
const SCHEMA_2: &str = "schema_name_2";

type InMemory = DataManager<InMemoryDatabase>;

#[rstest::fixture]
fn data_manager() -> InMemory {
    DataManager::default()
}

#[rstest::fixture]
fn data_manager_with_schema(data_manager: InMemory) -> InMemory {
    data_manager.create_schema(&SCHEMA).expect("schema is created");
    data_manager
}
