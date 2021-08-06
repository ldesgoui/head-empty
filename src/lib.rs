#![doc = include_str!("../README.md")]
#![deny(unsafe_code)]

mod compile_tests;
mod de;

#[cfg_attr(feature = "internal-doc-hidden", doc(hidden))]
pub use erased_serde;
#[cfg_attr(feature = "internal-doc-hidden", doc(hidden))]
pub use linkme;
#[cfg_attr(feature = "internal-doc-hidden", doc(hidden))]
pub use paste;

use erased_serde as erased;
use once_cell::race::OnceBox;
use std::any::{Any, TypeId};
use std::collections::HashMap;

type Store = HashMap<TypeId, Box<dyn Any + Send + Sync>>;

static STORE: OnceBox<Store> = OnceBox::new();

#[cfg_attr(feature = "internal-doc-hidden", doc(hidden))]
pub fn store_get<T>() -> &'static T {
    STORE
        .get()
        .expect("`head_empty::init` was not called soon enough")
        .get(&TypeId::of::<T>())
        .unwrap()
        .downcast_ref()
        .unwrap()
}

#[cfg_attr(feature = "internal-doc-hidden", doc(hidden))]
pub struct Registration {
    pub field: &'static str,
    pub type_id: fn() -> TypeId,
    pub deserialize: DeserializeFn,
}

type DeserializeFn =
    fn(&mut dyn erased::Deserializer) -> erased::Result<Box<dyn Any + Send + Sync>>;

#[cfg_attr(feature = "internal-doc-hidden", doc(hidden))]
#[linkme::distributed_slice]
pub static REGISTRATIONS: [Registration] = [..];

#[cfg_attr(feature = "internal-doc-hidden", doc(hidden))]
pub trait CanOnlyRegisterOnce {}

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

/// Register a type to be deserialized during [`init`]
///
/// Types or field names may only be registered once
///
/// Only crate-local types may be used
///
/// The trailing comma is optional
///
/// ```
/// # #[derive(serde::Deserialize)] struct Type;
/// # #[derive(serde::Deserialize)] struct Type2;
/// #
/// head_empty::register! {
///     name: Type,
///     name2: Type2,
/// }
/// ```
#[macro_export]
macro_rules! register {
    ( $( $field:ident: $type:ty ),* ) => {
        $crate::register! { $( $field: $type, )* }
    };

    ( $( $field:ident: $type:ty , )+ ) => {
        $(
            impl $crate::CanOnlyRegisterOnce for $type {}

            $crate::paste::paste! {
                #[$crate::linkme::distributed_slice($crate::REGISTRATIONS)]
                static [< REGISTRATION_FOR_ $field >]: $crate::Registration = $crate::Registration {
                    field: ::std::stringify!($field),
                    type_id: || ::std::any::TypeId::of::<$type>(),
                    deserialize: |d| {
                        ::std::result::Result::Ok(::std::boxed::Box::new(
                            $crate::erased_serde::deserialize::<$type>(d)?,
                        ))
                    },
                };
            }

            impl $type {
                /// # Panics
                ///
                /// This will panic if [`head_empty::init`] has not successfully ran prior
                fn configured() -> &'static Self {
                    $crate::store_get()
                }
            }
        )+
    };
}
