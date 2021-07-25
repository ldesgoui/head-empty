# head-empty

## Define parts of your configuration schema throughout your codebase

### Example

```rust
// mysql.rs

#[derive(Debug, PartialEq, Eq, serde::Deserialize)]
struct Mysql {
    host: String,
    database: String,
    user: String,
    password: String,
}

head_empty::register! {
    mysql: Mysql,
}

// main.rs

#[derive(Debug, PartialEq, Eq, serde::Deserialize)]
struct Debug(bool);

#[derive(Debug, PartialEq, Eq, serde::Deserialize)]
struct ListenPort(u16);

head_empty::register! {
    debug: Debug,
    listen_port: ListenPort,
}

let deserializer = serde_json::json!({
    "mysql": {
        "host": "localhost:5432",
        "database": "test",
        "user": "root",
        "password": "toor",
    },
    "debug": true,
    "listen_port": 8080,
});

head_empty::init(deserializer).expect("deserializing configuration failed");

let mysql: &'static Mysql = Mysql::configured();
assert_eq!(
    mysql,
    &Mysql {
        host: "localhost:5432".into(),
        database: "test".into(),
        user: "root".into(),
        password: "toor".into()
    }
);

let debug: &'static Debug = Debug::configured();
assert_eq!(debug, &Debug(true));

let listen_port: &'static ListenPort = ListenPort::configured();
assert_eq!(listen_port, &ListenPort(8080));
```
