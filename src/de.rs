use crate::{DeserializeFn, Registration, Store};
use erased_serde as erased;
use serde::de::{self, DeserializeSeed, Deserializer, MapAccess, Visitor};
use std::any::Any;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fmt;

pub(crate) fn deserialize<'de, Der>(regs: &[Registration], der: Der) -> Result<Store, Der::Error>
where
    Der: Deserializer<'de>,
{
    Seed::new(regs).deserialize(der)
}

struct Seed<'a> {
    regs: HashMap<&'static str, &'a Registration>,
}

impl<'a> Seed<'a> {
    fn new(slice: &'a [Registration]) -> Self {
        let mut regs = HashMap::with_capacity(slice.len());

        for reg in slice {
            if regs.insert(reg.field, reg).is_some() {
                panic!(
                    "The field '{}' was registered once too many times",
                    reg.field
                );
            }
        }

        Self { regs }
    }
}

impl<'de> DeserializeSeed<'de> for Seed<'_> {
    type Value = Store;

    fn deserialize<Der>(self, der: Der) -> Result<Self::Value, Der::Error>
    where
        Der: Deserializer<'de>,
    {
        der.deserialize_map(self)
    }
}

impl<'de, 'a> Visitor<'de> for Seed<'a> {
    type Value = Store;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Configuration")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut store = Store::with_capacity(self.regs.len());
        let mut visited = HashSet::with_capacity(self.regs.len());

        while let Some(key) = map.next_key::<Cow<str>>()? {
            if let Some(reg) = self.regs.get(key.as_ref()) {
                if !visited.insert(key) {
                    return Err(de::Error::duplicate_field(reg.field));
                }

                let boxed = map.next_value_seed(Wrapper(&reg.deserialize))?;

                store.insert(reg.field, boxed);
            } else {
                // TODO: deny unknown fields
            }
        }

        for reg in self.regs.values() {
            if visited.contains(reg.field) {
                continue;
            }

            // TODO: default

            return Err(de::Error::missing_field(reg.field));
        }

        Ok(store)
    }
}

struct Wrapper<T>(T);

impl<'de> DeserializeSeed<'de> for Wrapper<&DeserializeFn> {
    type Value = Box<dyn Any + Send + Sync>;

    fn deserialize<D>(self, der: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut erased = <dyn erased::Deserializer>::erase(der);
        (self.0)(&mut erased).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use erased_serde as erased;
    use serde_json as json;

    #[derive(Debug, PartialEq, Eq, serde::Deserialize)]
    struct Mysql {
        host: String,
        database: String,
        user: String,
        password: String,
    }

    const REGISTRATIONS: [Registration; 2] = [
        Registration {
            field: "mysql",
            deserialize: |d| Ok(Box::new(erased::deserialize::<Mysql>(d)?)),
        },
        Registration {
            field: "listen_port",
            deserialize: |d| Ok(Box::new(erased::deserialize::<u16>(d)?)),
        },
    ];

    fn get<'a, T: 'static>(store: &'a Store, field: &'static str) -> &'a T {
        store.get(field).unwrap().downcast_ref().unwrap()
    }

    #[test]
    fn missing_field() {
        let de = json::json!({
            "listen_port": 8080,
        });

        let registrations = REGISTRATIONS;

        assert!(super::deserialize(&registrations, de).is_err());
    }

    #[test]
    fn smoke_test() {
        let de = json::json!({
            "mysql": {
                "host": "localhost:5432",
                "database": "test",
                "user": "root",
                "password": "toor",
            },
            "listen_port": 8080,
        });

        let registrations = REGISTRATIONS;
        let store = super::deserialize(&registrations, de).unwrap();

        assert_eq!(store.len(), registrations.len());

        assert_eq!(
            get::<Mysql>(&store, "mysql"),
            &Mysql {
                host: "localhost:5432".into(),
                database: "test".into(),
                user: "root".into(),
                password: "toor".into()
            }
        );

        assert_eq!(get::<u16>(&store, "listen_port"), &8080);
    }
}
