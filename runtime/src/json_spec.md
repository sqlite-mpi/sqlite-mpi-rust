# JSON protocol spec

This document is a rough sketch of the JSON protocol used in SMPI.

The design is used to:
- Inform
    - Serde configuration and custom Rust <-> JSON mapping functions.
    - Rust tests.
- Client libs that send and receive these messages.

**Question: Why not use a formal API tool?**

- Client libs will encapsulate this protocol.
    - Users of SMPI will be using a language level interface, not writing JSON.
        - This JSON protocol is a private implementation detail, not a public interface.
    - Client libs will have functional tests that implicitly test the API.
- Rust tests will test the JSON schema (and the conversion of Rust values to and from it).
- Reduce dependencies on external tools.
- Get a prototype running and allow fast iteration before cementing the design. 




# Implemented

## Result

Out
```json
{
    "ok": true,
    "res_type": "txOpen",
    "res": {}
}
```

Out
```json
{
    "ok": false,
    "error": {}
}
```


### Error 

Out
```json
{
    "error_type": "/enum/path/as/key",
    "message": "",
    "data": {
        "return_status": {}
    }
}
```

- `error.data.return_status` includes the status if *any* enum variant contains it as a value?
    - Can this be done with a macro?





## Read / Write transactions

In
```json
{
    "fn": "file/(get_read_tx|get_write_tx)",
    "args": {
        "file": "/a/b/c/file.sqlite3"
    }
}
```

Out
```json
{
    "tx_id": "str"
}
```





In
```json
{
    "fn": "tx/(q|read|write)_params",
    "args": {
        "tx_id": "x",
        "q": "",
        "key_based": {
        },
        "index_based": []
    }
}
```

Out
```json
{
    "is_read_only": false,
    "is_iud": false,
    "rows_changed": 0,
    "col_names": [
        {}
    ],
    "rows": [
        [],
        []
    ]
}
```



In
```json
{
    "fn": "tx/(commit|rollback)",
    "args": {
        "tx_id": "x"
    }
}
```









# Not implemented

## Single query

```json
{
    "fn": "file/(q|read|write)_params",
    "args": {
        "file": "/a/b/c/file.sqlite3",
        "query": "",
        "key_based": {},
        "index_based": []
    }
}
```


```json
{
    "fn": "file/(close|get_open_txs)",
    "args": {
        "file": "/a/b/c/file.sqlite3"
    }
}
```


## Config

```json
{
    "fn": "( config/set | file/config/set )",
    "args": {
        "cache_connections": true,
        "timeout": 0,
        "debug": true,
        "add_ts": false
    }
}
```

## Global per runtime ops

```json
{
    "fn": "files/(close_all|get_connections|get_open_txs)",
    "args": {
        "cache_connections": true,
        "timeout": 0
    }
,}
```


## Tx status
```json
[
    {
        "/a/b/c.sqlite": {
            "read_txs": {
                "tx_id_123": {
                    "db_con": "x"
                }
            },
            "write_tx": {
                "current": {
                    "db_con": "y"
                }
            }
        }
    },
    {
        "/a/b/c.sqlite": {
            "txs": {
                "tx_id_123": {
                    "tx_id": "abc",
                    "type": "read|write",
                    "db_con": "x"
                }
            },
            "queued_write_txs": [
                "reqWithId"
            ]
        }
    }
]
```