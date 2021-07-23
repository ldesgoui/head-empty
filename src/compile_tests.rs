/// ```
/// #[derive(Debug, PartialEq, Eq, serde::Deserialize)]
/// struct Mysql {
///     host: String,
///     database: String,
///     user: String,
///     password: String,
/// }
///
/// #[derive(Debug, PartialEq, Eq, serde::Deserialize)]
/// struct ListenPort(u16);
///
/// head_empty::register!{
///     mysql: Mysql,
///     listen_port: ListenPort,
/// }
///
/// let deserializer = serde_json::json!({
///     "mysql": {
///         "host": "localhost:5432",
///         "database": "test",
///         "user": "root",
///         "password": "toor",
///     },
///     "listen_port": 8080,
/// });
///
/// head_empty::init(deserializer).unwrap();
///
/// let mysql: &'static Mysql = Mysql::configured();
/// let listen_port: &'static ListenPort = ListenPort::configured();
///
/// assert_eq!(
///     mysql,
///     &Mysql {
///         host: "localhost:5432".into(),
///         database: "test".into(),
///         user: "root".into(),
///         password: "toor".into()
///     }
/// );
///
/// assert_eq!(listen_port, &ListenPort(8080));
/// ```
fn _smoke_test() {}

/// ```
/// #[derive(serde::Deserialize)] struct A;
/// #[derive(serde::Deserialize)] struct B;
/// #[derive(serde::Deserialize)] struct C;
/// #[derive(serde::Deserialize)] struct D;
/// #[derive(serde::Deserialize)] struct E;
/// #[derive(serde::Deserialize)] struct F;
///
/// head_empty::register!{ a: A }
/// head_empty::register!{ b: B, }
/// head_empty::register!{ c: C, d: D }
/// head_empty::register!{ e: E, f: F, }
/// ```
fn _register_commas() {}

/// ```compile_fail
/// head_empty::register!{ }
/// ```
fn _register_empty_call() {}

/// ```compile_fail
/// #[derive(serde::Deserialize)]
/// struct A;
///
/// head_empty::register!{
///     a: A,
///     b: A,
/// }
/// ```
fn _cannot_registrer_the_same_type_twice() {}

/// ```compile_fail
/// head_empty::register!{ a: u64 }
/// ```
fn _cannot_register_external_types() {}

/// ```
/// use ::head_empty as aliased;
///
/// mod head_empty {}
/// mod erased_serde {}
/// mod linkme {}
/// mod paste {}
/// mod std {}
///
/// enum Result {
///     Ok(i32),
///     Err(i32),
/// }
///
/// fn Ok() {}
///
/// struct Box<T>(T);
///
/// macro_rules! stringify {
///     () => {}
/// }
///
/// #[derive(serde::Deserialize)]
/// struct A;
///
/// aliased::register! { a: A }
/// ```
fn _register_path_test() {}