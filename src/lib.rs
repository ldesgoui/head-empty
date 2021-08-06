#![doc = include_str!("../README.md")]
#![deny(unsafe_code)]

mod de;

#[cfg_attr(feature = "internal-doc-hidden", doc(hidden))]
pub use erased_serde;
#[cfg_attr(feature = "internal-doc-hidden", doc(hidden))]
pub use linkme;
#[cfg_attr(feature = "internal-doc-hidden", doc(hidden))]
pub use paste;

use erased_serde as erased;
use once_cell::race::OnceBox;
use std::any::Any;
use std::collections::HashMap;

type Store = HashMap<&'static str, Box<dyn Any + Send + Sync>>;

static STORE: OnceBox<Store> = OnceBox::new();

#[cfg_attr(feature = "internal-doc-hidden", doc(hidden))]
pub fn store_get<T>(field: &'static str) -> &'static T {
    STORE
        .get()
        .expect("`head_empty::init` was not called soon enough")
        .get(field)
        .unwrap()
        .downcast_ref()
        .unwrap()
}

#[cfg_attr(feature = "internal-doc-hidden", doc(hidden))]
pub struct Registration {
    pub field: &'static str,
    pub deserialize: DeserializeFn,
}

type DeserializeFn =
    fn(&mut dyn erased::Deserializer) -> erased::Result<Box<dyn Any + Send + Sync>>;

#[cfg_attr(feature = "internal-doc-hidden", doc(hidden))]
#[linkme::distributed_slice]
pub static REGISTRATIONS: [Registration] = [..];

/// Initialize configuration by deserializing it from a [`serde::Deserializer`]
///
/// # Panics
///
/// This will panic if it has already succeeded prior
///
/// This will panic if the same field name has been registered multiple times
pub fn init<'de, Der>(der: Der) -> Result<(), Der::Error>
where
    Der: serde::Deserializer<'de>,
{
    let store = de::deserialize(&REGISTRATIONS, der)?;

    if STORE.set(Box::new(store)).is_err() {
        panic!("`head_empty::init` was called once too many times");
    }

    Ok(())
}

/// Register fields to be deserialized during [`init`]
///
/// ```rust
/// head_empty::register! {
///     host: String,
///     port: u8,
/// }
///
/// // Will define:
/// // fn configured_host() -> &'static String;
/// // fn configured_port() -> &'static u8;
/// ```
///
/// A field name can only be registered once
///
/// ```rust,should_panic
/// head_empty::register! {
///     same_field: String,
/// # }
/// # mod another {
/// # head_empty::register! {
///     same_field: u16,
/// }
/// # }
/// # let deserializer = serde_json::json!({
/// #     "same_field": "woops!",
/// # });
///
/// head_empty::init(deserializer); // panic!
/// ```
///
/// The trailing comma is optional
///
/// ```rust
/// head_empty::register! {
///     host: String,
///     port: u8
/// }
/// ```
#[macro_export]
macro_rules! register {
    ( $( $field:ident: $type:ty ),* ) => {
        $crate::register! { $( $field: $type, )* }
    };

    ( $( $field:ident: $type:ty , )+ ) => {
        $crate::paste::paste! {
            $(
                /// Get the configured value for the field
                #[doc = "`"]
                #[doc = ::std::stringify!($field)]
                #[doc = "`"]
                ///
                /// # Panics
                ///
                /// This will panic if [`head_empty::init`] has not successfully ran prior
                fn [< configured_ $field >]() -> &'static $type {
                    use $crate::{
                        erased_serde, linkme::distributed_slice, store_get, Registration,
                        REGISTRATIONS,
                    };
                    use ::std::{boxed::Box, result::Result::Ok, stringify};

                    #[distributed_slice(REGISTRATIONS)]
                    static REGISTRATION: Registration = Registration {
                        field: stringify!($field),
                        deserialize: |d| Ok(Box::new(erased_serde::deserialize::<$type>(d)?)),
                    };

                    store_get(stringify!($field))
                }
            )+
        }
    };
}
