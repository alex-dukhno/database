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
fn schema_does_not_exist() {
    let db = Database::new("");
    let analyzer = QueryAnalyzer::from(db.transaction());
    assert_eq!(
        analyzer.analyze(select("non_existent_schema", TABLE)),
        Err(AnalysisError::schema_does_not_exist(&"non_existent_schema"))
    );
}

#[test]
fn table_does_not_exist() {
    let db = Database::new("");
    let transaction = db.transaction();
    let catalog = CatalogHandler::from(transaction.clone());
    catalog.apply(create_schema_ops(SCHEMA)).unwrap();

    let analyzer = QueryAnalyzer::from(transaction);
    assert_eq!(
        analyzer.analyze(select(SCHEMA, "non_existent_table")),
        Err(AnalysisError::table_does_not_exist(&format!("{}.{}", SCHEMA, "non_existent_table")))
    );
}
