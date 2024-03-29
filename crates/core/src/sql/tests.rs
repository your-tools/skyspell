use crate::operations::Ignore as IgnoreOperation;
use crate::sql::schema::operations;
use crate::test_ignore_store;
use crate::test_repository;
use crate::IgnoreStore;
use crate::Operation;
use crate::Repository;
use crate::SQLRepository;

use diesel::dsl::count_star;
use diesel::prelude::*;

#[test]
fn test_delete_old_operations_when_more_than_100_operations_are_stored() {
    let mut sql_repository = SQLRepository::new_for_tests().unwrap();
    let values: Vec<_> = (1..=103)
        .map(|i| {
            let word = format!("foo-{}", i);
            let operation = Operation::Ignore(IgnoreOperation { word });
            let json = serde_json::to_string(&operation).unwrap();
            (
                operations::json.eq(json),
                operations::timestamp.eq(i + 10_000),
            )
        })
        .collect();
    diesel::insert_into(operations::table)
        .values(values)
        .execute(&mut sql_repository.connection)
        .unwrap();

    let last = sql_repository.pop_last_operation().unwrap();
    assert!(last.is_some());

    let actual_count: i64 = operations::table
        .select(count_star())
        .first(&mut sql_repository.connection)
        .unwrap();

    assert_eq!(actual_count, 101);
}

#[test]
fn test_keep_old_operations_when_less_than_100_operations_are_stored() {
    let mut sql_repository = SQLRepository::new_for_tests().unwrap();
    let values: Vec<_> = (1..=50)
        .map(|i| {
            let word = format!("foo-{}", i);
            let operation = Operation::Ignore(IgnoreOperation { word });
            let json = serde_json::to_string(&operation).unwrap();
            (
                operations::json.eq(json),
                operations::timestamp.eq(i + 10_000),
            )
        })
        .collect();
    diesel::insert_into(operations::table)
        .values(values)
        .execute(&mut sql_repository.connection)
        .unwrap();

    let last = sql_repository.pop_last_operation().unwrap();
    assert!(last.is_some());

    let actual_count: i64 = operations::table
        .select(count_star())
        .first(&mut sql_repository.connection)
        .unwrap();

    assert_eq!(actual_count, 49);
}

test_ignore_store!(SQLRepository);
test_repository!(SQLRepository);
