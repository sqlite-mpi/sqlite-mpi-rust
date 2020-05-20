//use std::mem::{drop};


// @see https://stackoverflow.com/a/39009227/4949386 (separate tests into their own files).
use super::*;
use placeholder::PlaceholderTypes;
use serde_json::json;
use crate::stmt::ErrorBind;

// Place outside src so that file writes do not trigger `cargo watch`.
static TEST_OUTPUT_DIR: &'static str = "/tmp";

#[test]
fn test_is_valid_handle() {
    // Note: sqlite3 FFI does not provide a `sqlite3_is_valid_handle(db)` fn.
    // - Just trust return status instead of reading memory?
    // - `sqlite3` object is "opaque", meaning only a pointer is given to the FFI client.
    // @see https://en.wikipedia.org/wiki/Opaque_data_type
}


fn get_test_file() -> String {
    let now: DateTime<Utc> = Utc::now();
    format!("{}/del-test-{:?}.sqlite3", TEST_OUTPUT_DIR, now)
}


#[test]
fn test_get_db_handle() {
    let file = get_test_file();

    // First iteration creates the file, later iterations connect to the same empty file.
    // At the end of the block `drop` will call `sqlite3_close` successfully, or will `panic`.
    for _ in 0..5 {
        let h = DbHandle::new(file.clone()).unwrap();
    }
}

#[test]
fn test_get_stmt_handle_err() {
    let file = get_test_file();
    let h = DbHandle::new(file).unwrap();

    for _ in 0..5 {
        let s = StmtHandle::new(&h, "SELECT * FROM table_does_not_exist");

        match s {
            Err(e) => {
                match e.primary.id {
                    PrimaryRC::SQLITE_ERROR => {
                        match e.err_msg {
                            Some(m) => assert!(m.len() > 0, "Should be a msg"),
                            None => assert!(false, "Should have msg")
                        }
                    }
                    _ => assert!(false, "Should be SQLITE_ERROR")
                }
            }
            Ok(_) => assert!(false, "Should be Err")
        }
    }
}


#[test]
fn test_get_stmt_handle_ok() {
    let file = get_test_file();
    let h = DbHandle::new(file).unwrap();

    // @todo/low test closing DB early and trying to use handle.

    // Create table
    {
        let s = StmtHandle::new(&h, "CREATE TABLE t1(a PRIMARY KEY, b);").expect("Syntax Ok");
        let rset = s.run().expect("Creates table");
        assert!(!rset.is_read_only, "Q modifies db file");
    }


    // Create the same table again returns error *when creating stmt* (not at VM step runtime).
    {
        let s = StmtHandle::new(&h, "CREATE TABLE t1(a PRIMARY KEY, b);");
        match s {
            Err(e) => {
                if let PrimaryRC::SQLITE_ERROR = e.primary.id {
                    assert!(true, "Correct error ID");
                } else {
                    assert!(false, "Incorrect error ID");
                }
            }
            Ok(_) => assert!(false)
        }
    }


    // @todo/important Test extended error codes.
}


fn assert_bind_kv_fails_with(s: &StmtHandle, kind: ErrorBindType) {
    let data: HashMap<String, Val> = [
        ("abc".to_string(), Val::I64(1)),
    ].iter().cloned().collect();
    let kv = KeyVal { data };


    let e = s.bind_kv(&kv);
    assert!(e.is_err());

    match e {
        Err(eb) => {
            assert!(&eb.kind == &kind);
        }
        _ => {}
    }
}


fn assert_bind_err_eq(e: &Result<(), ErrorBind>, kind: ErrorBindType) {
    assert!(e.is_err());

    match e {
        Err(eb) => {
            assert!(&eb.kind == &kind);
        }
        _ => {}
    }
}

fn assert_bind_index_fails_with(s: &StmtHandle, kind: ErrorBindType) {
    let e = s.bind_index(&vec![]);
    assert_bind_err_eq(&e, kind);
}

#[test]
fn test_get_stmt_params_kv_ok() {
    {
        let h = DbHandle::new(":memory:".to_string()).unwrap();


        // Assert: Key prefix character is ignored. Extra keys in KV input are ignored.
        {
            let q = r#"
                SELECT :keyA, @keyA, $keyA
            "#;
            let s = StmtHandle::new(&h, q).expect("Syntax Ok");
            let t = &s.placeholder_meta.types_used;
            assert_eq!(PlaceholderTypes::Key, *t);
            assert_bind_kv_fails_with(&s, ErrorBindType::MissingKeysInData);

            // Assumption: `stmt` has 0 values bound.
            // @todo/low Enforce a single data binding per stmt, or use reset.

            let data: HashMap<String, Val> = [
                // This will be used to bind all placeholders.
                ("keyA".to_string(), Val::I64(1)),

                // Assert: Prefix of placeholders is not used.
                (":keyA".to_string(), Val::I64(12)),
                ("@keyA".to_string(), Val::I64(13)),
                ("$keyA".to_string(), Val::I64(14)),
                ("extraKeyIsIgnored".to_string(), Val::I64(15))
            ].iter().cloned().collect();
            let kv = KeyVal { data };

            assert!(&s.bind_kv(&kv).is_ok());

            if let Ok(r) = &s.run() {
                // Alternative:
                // let target = json!([[1, 1, 1]]);
                // assert_eq!(target, serde_json::to_value(&r.rows.data).unwrap());

                let target = Rows {
                    data: vec![
                        vec![Val::I64(1), Val::I64(1), Val::I64(1)],
                    ]
                };

                assert_eq!(target, r.rows);
            } else {
                assert!(false);
            }
        }

        // Assert: Numeric named placeholders are treated like string keys, NOT index values (like ?123).
        {
            let q = r#"
                SELECT
                    :111,
                    @222,
                    $333
            "#;
            let s = StmtHandle::new(&h, q).expect("Syntax Ok");
            let t = &s.placeholder_meta.types_used;
            assert_eq!(PlaceholderTypes::Key, *t);

            let data: HashMap<String, Val> = [
                ("111".to_string(), Val::I64(111)),
                ("222".to_string(), Val::I64(222)),
                ("333".to_string(), Val::I64(333))
            ].iter().cloned().collect();
            let kv = KeyVal { data };

            assert!(&s.bind_kv(&kv).is_ok());

            if let Ok(r) = &s.run() {
                // Alternative:
                // let target = json!([[1, 1, 1]]);
                // assert_eq!(target, serde_json::to_value(&r.rows.data).unwrap());

                let target = Rows {
                    data: vec![
                        vec![Val::I64(111), Val::I64(222), Val::I64(333)],
                    ]
                };

                assert_eq!(target, r.rows);
            } else {
                assert!(false);
            }
        }

        // Assert: Bound input values are returned as the same type in the output rows.
        {
            let q = r#"
                SELECT
                    :i64,
                    :f64,
                    :string,
                    :null,
                    :blob

            "#;
            let s = StmtHandle::new(&h, q).expect("Syntax Ok");
            let t = &s.placeholder_meta.types_used;
            assert_eq!(PlaceholderTypes::Key, *t);

            let data: HashMap<String, Val> = [
                ("i64".to_string(), Val::I64(123)),
                ("f64".to_string(), Val::F64(567.567)),
                ("string".to_string(), Val::Null),
                ("null".to_string(), Val::String("example string".to_string())),
                ("blob".to_string(), Val::Blob("A string as bytes".to_string().into_bytes()))
            ].iter().cloned().collect();
            let kv = KeyVal { data };

            assert!(&s.bind_kv(&kv).is_ok());

            if let Ok(r) = &s.run() {
                let target = Rows {
                    data: vec![
                        vec![
                            Val::I64(123),
                            Val::F64(567.567),
                            Val::Null,
                            Val::String("example string".to_string()),
                            Val::Blob("A string as bytes".to_string().into_bytes())
                        ],
                    ]
                };

                assert_eq!(target, r.rows);
            } else {
                assert!(false);
            }
        }
    }
}


#[test]
fn test_get_stmt_params_index_ok() {
    {
        let h = DbHandle::new(":memory:".to_string()).unwrap();


        // Assert: Simple index based params.
        {
            let q = r#"
                SELECT ?, ?, ?
            "#;
            let s = StmtHandle::new(&h, q).expect("Syntax Ok");
            let t = &s.placeholder_meta.types_used;
            assert_eq!(PlaceholderTypes::Index, *t);
            assert_bind_kv_fails_with(&s, ErrorBindType::PlaceholderDataTypeNotCompatible);

            let vals: Vec<Val> = vec![
                Val::I64(1),
                Val::I64(2),
                Val::I64(3)
            ];

            assert!(&s.bind_index(&vals).is_ok());

            if let Ok(r) = &s.run() {
                let target = Rows {
                    data: vec![
                        vec![Val::I64(1), Val::I64(2), Val::I64(3)],
                    ]
                };

                assert_eq!(target, r.rows);
            } else {
                assert!(false);
            }
        }


        // Assert: Bound input values are returned as the same type in the output rows.
        {
            let q = r#"
                SELECT ?, ?, ?, ?, ?
            "#;
            let s = StmtHandle::new(&h, q).expect("Syntax Ok");

            let vals: Vec<Val> = vec![
                Val::I64(123),
                Val::F64(567.567),
                Val::Null,
                Val::String("example string".to_string()),
                Val::Blob("A string as bytes".to_string().into_bytes())
            ];

            assert!(&s.bind_index(&vals).is_ok());

            if let Ok(r) = &s.run() {
                let target = Rows {
                    data: vec![
                        vec![
                            Val::I64(123),
                            Val::F64(567.567),
                            Val::Null,
                            Val::String("example string".to_string()),
                            Val::Blob("A string as bytes".to_string().into_bytes())
                        ],
                    ]
                };

                assert_eq!(target, r.rows);
            } else {
                assert!(false);
            }
        }
    }
}

#[test]
fn test_get_stmt_params_index_error() {
    let h = DbHandle::new(":memory:".to_string()).unwrap();


    // Assert: Placeholder params must all be bound.
    {
        let q = r#"
                SELECT ?, ?, ?
            "#;
        let s = StmtHandle::new(&h, q).expect("Syntax Ok");

        let vals: Vec<Val> = vec![
            Val::I64(1),
            Val::I64(2),
        ];

        assert_bind_err_eq(&s.bind_index(&vals), ErrorBindType::MissingIndexesInData);
    }

    // Assert: Placeholder params must all be bound.
    {
        let q = r#"
                SELECT ?99
            "#;
        let s = StmtHandle::new(&h, q).expect("Syntax Ok");

        let vals: Vec<Val> = vec![
            Val::I64(1),
            Val::I64(2),
        ];

        assert_bind_err_eq(&s.bind_index(&vals), ErrorBindType::MissingIndexesInData);
    }
}


#[test]
fn test_get_stmt_params_error() {
    {
        let h = DbHandle::new(":memory:".to_string()).unwrap();


        {
            let q = r#"
                SELECT 1
            "#;
            let s = StmtHandle::new(&h, q).expect("Syntax Ok");
            let t = &s.placeholder_meta.types_used;

            assert_eq!(PlaceholderTypes::None, *t);
            assert_bind_kv_fails_with(&s, ErrorBindType::PlaceholderDataTypeNotCompatible);
            assert_bind_index_fails_with(&s, ErrorBindType::PlaceholderDataTypeNotCompatible);
        }

        {
            let q = r#"
                SELECT ?, ?10
            "#;
            let s = StmtHandle::new(&h, q).expect("Syntax Ok");
            let t = &s.placeholder_meta.types_used;
            assert_eq!(PlaceholderTypes::Index, *t);
            assert_bind_kv_fails_with(&s, ErrorBindType::PlaceholderDataTypeNotCompatible);
        }


        {
            let q = r#"
                SELECT ?, ?
            "#;
            let s = StmtHandle::new(&h, q).expect("Syntax Ok");
            let t = &s.placeholder_meta.types_used;
            assert_eq!(PlaceholderTypes::Index, *t);
            assert_bind_kv_fails_with(&s, ErrorBindType::PlaceholderDataTypeNotCompatible);
        }

        {
            let q = r#"
                SELECT ?10, ?20
            "#;
            let s = StmtHandle::new(&h, q).expect("Syntax Ok");
            let t = &s.placeholder_meta.types_used;
            assert_eq!(PlaceholderTypes::Index, *t);
            assert_bind_kv_fails_with(&s, ErrorBindType::PlaceholderDataTypeNotCompatible);
        }


        {
            // `?five` will not show up in the placeholder names (`?123` would), but it still takes an index space?
            let q = r#"
                SELECT ?five
            "#;
            let s = StmtHandle::new(&h, q).expect("Syntax Ok");
            let t = &s.placeholder_meta.types_used;
            assert_eq!(PlaceholderTypes::Index, *t);
            assert_bind_kv_fails_with(&s, ErrorBindType::PlaceholderDataTypeNotCompatible);
        }


        {
            let q = r#"
                SELECT :keyA, @keyA, $keyA, ?
            "#;
            let s = StmtHandle::new(&h, q).expect("Syntax Ok");
            let t = &s.placeholder_meta.types_used;
            assert_eq!(PlaceholderTypes::IndexAndKey, *t);

            assert_bind_kv_fails_with(&s, ErrorBindType::PlaceholderDataTypeNotCompatible);
            assert_bind_index_fails_with(&s, ErrorBindType::PlaceholderDataTypeNotCompatible);
        }

        {
            let q = r#"
                SELECT :keyA, @keyA, $keyA, ?10
            "#;
            let s = StmtHandle::new(&h, q).expect("Syntax Ok");
            let t = &s.placeholder_meta.types_used;
            assert_eq!(PlaceholderTypes::IndexAndKey, *t);
            assert_bind_kv_fails_with(&s, ErrorBindType::PlaceholderDataTypeNotCompatible);
            assert_bind_index_fails_with(&s, ErrorBindType::PlaceholderDataTypeNotCompatible);
        }


        {
            let q = r#"
                SELECT ?, ?, ?, :keyA
            "#;
            let s = StmtHandle::new(&h, q).expect("Syntax Ok");
            let t = &s.placeholder_meta.types_used;
            assert_eq!(PlaceholderTypes::IndexAndKey, *t);
            assert_bind_kv_fails_with(&s, ErrorBindType::PlaceholderDataTypeNotCompatible);
            assert_bind_index_fails_with(&s, ErrorBindType::PlaceholderDataTypeNotCompatible);
        }


        {
            let q = r#"
                SELECT :keyA, ?, ?, ?, :keyA
            "#;
            let s = StmtHandle::new(&h, q).expect("Syntax Ok");
            let t = &s.placeholder_meta.types_used;
            assert_eq!(PlaceholderTypes::IndexAndKey, *t);
            assert_bind_kv_fails_with(&s, ErrorBindType::PlaceholderDataTypeNotCompatible);
            assert_bind_index_fails_with(&s, ErrorBindType::PlaceholderDataTypeNotCompatible);
        }

        {
            let q = r#"
                SELECT :keyA, ?10, :keyA, ?
            "#;
            let s = StmtHandle::new(&h, q).expect("Syntax Ok");
            let t = &s.placeholder_meta.types_used;
            assert_eq!(PlaceholderTypes::IndexAndKey, *t);
            assert_bind_kv_fails_with(&s, ErrorBindType::PlaceholderDataTypeNotCompatible);
            assert_bind_index_fails_with(&s, ErrorBindType::PlaceholderDataTypeNotCompatible);
        }
    }
}


// @todo/low Test meta data such as: column names, `is_read_only`, `is_iud`. Allow these to be read before run?



