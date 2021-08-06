Define parts of your configuration schema throughout your codebase

### Example

```rust
head_empty::register! {
    debug: bool,
    listen_port: u16,
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

head_empty::init(deserializer)
    .expect("deserializing configuration failed");

assert_eq!(configured_debug(), &true);
assert_eq!(configured_listen_port(), &8080);

mysql::run();

// In a completely different part of your codebase
mod mysql {
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

    pub(crate) fn run() {
        let mysql: &'static Mysql = configured_mysql();

        assert_eq!(
            mysql,
            &Mysql {
                host: "localhost:5432".into(),
                database: "test".into(),
                user: "root".into(),
                password: "toor".into()
            }
        );
    }
}

```
