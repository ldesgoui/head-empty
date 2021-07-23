use crate::{DeserializeFn, Registration, Store};
use erased_serde as erased;
use serde::de::{self, DeserializeSeed, Deserializer, MapAccess, Visitor};
use std::any::Any;
use std::collections::HashMap;
use std::fmt;

pub(crate) fn deserialize<'de, Der>(regs: &[Registration], der: Der) -> Result<Store, Der::Error>
where
    Der: Deserializer<'de>,
{
    Seed::new(regs).deserialize(der)
}

struct Seed<'a> {
    store: Store,
    regs: HashMap<&'static str, &'a Registration>,
}

impl<'a> Seed<'a> {
    fn new(slice: &'a [Registration]) -> Self {
        let mut regs = HashMap::with_capacity(slice.len());

        for reg in slice {
            if regs.insert(reg.field, reg).is_some() {
                panic!("The field '{}' was once too many times", reg.field);
            }
        }

        Self {
            store: Store::with_capacity(slice.len()),
            regs,
        }
    }
}

impl<'de> DeserializeSeed<'de> for Seed<'_> {
    type Value = Store;

    fn deserialize<Der>(self, der: Der) -> Result<Self::Value, Der::Error>
    where
        Der: Deserializer<'de>,
    {
        Ok(der.deserialize_map(self)?.store)
    }
}

impl<'de, 'a> Visitor<'de> for Seed<'a> {
    type Value = Seed<'a>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Configuration")
    }

    fn visit_map<A>(mut self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        while let Some(key) = map.next_key::<String>()? {
            if let Some(reg) = self.regs.get(&*key) {
                let x = map.next_value_seed(Wrapper(&reg.deserialize))?;
                let type_id = (reg.type_id)();

                if self.store.insert(type_id, x).is_some() {
                    return Err(de::Error::duplicate_field(reg.field));
                }
            }
        }

        Ok(self)
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
        (self.0)(&mut erased).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use erased_serde as erased;
    use serde_json as json;
    use std::any::TypeId;

    #[derive(Debug, PartialEq, Eq, serde::Deserialize)]
    struct Mysql {
        host: String,
        database: String,
        user: String,
        password: String,
    }

    #[derive(Debug, PartialEq, Eq, serde::Deserialize)]
    struct ListenPort(u16);

    const REGISTRATIONS: [Registration; 2] = [
        Registration {
            field: "mysql",
            type_id: || TypeId::of::<Mysql>(),
            deserialize: |d| Ok(Box::new(erased::deserialize::<Mysql>(d)?)),
        },
        Registration {
            field: "listen_port",
            type_id: || TypeId::of::<ListenPort>(),
            deserialize: |d| Ok(Box::new(erased::deserialize::<ListenPort>(d)?)),
        },
    ];

    fn get<'a, T: 'static>(store: &'a Store) -> &'a T {
        store
            .get(&TypeId::of::<T>())
            .unwrap()
            .downcast_ref()
            .unwrap()
    }

    #[test]
    fn hello() {
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
            get::<Mysql>(&store),
            &Mysql {
                host: "localhost:5432".into(),
                database: "test".into(),
                user: "root".into(),
                password: "toor".into()
            }
        );

        assert_eq!(get::<ListenPort>(&store), &ListenPort(8080));
    }
}
