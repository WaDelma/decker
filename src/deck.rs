use std::{fmt, marker::PhantomData};

use rand::{prelude::*, Rng, SeedableRng};
use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize,
};

pub struct Deck<T, R> {
    deck: Vec<T>,
    discard: Vec<T>,
    rng: R,
}

impl<'de, T> Deserialize<'de> for Deck<T, StdRng>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CustomVisitor<T>(PhantomData<T>);

        impl<'de, T> Visitor<'de> for CustomVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = Deck<T, StdRng>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a struct with keys 'deck' and 'discard'")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                use serde::de::Error;
                let mut deck = None;
                let mut discard = None;

                while let Some(k) = map.next_key::<String>()? {
                    if k == "deck" {
                        deck = Some(map.next_value()?);
                    } else if k == "discard" {
                        discard = Some(map.next_value()?);
                    } else {
                        return Err(Error::custom(&format!("Invalid key: {}", k)));
                    }
                }

                Ok(Deck {
                    deck: deck.ok_or(Error::custom("Missing field deck"))?,
                    discard: discard.ok_or(Error::custom("Missing field discard"))?,
                    rng: StdRng::from_entropy(),
                })
            }
        }
        deserializer.deserialize_struct(
            "Deck",
            &["deck", "discard"][..],
            CustomVisitor(PhantomData),
        )
    }
}

impl<T, R> Serialize for Deck<T, R>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("Deck", 2)?;
        s.serialize_field("deck", &self.deck)?;
        s.serialize_field("discard", &self.discard)?;
        s.end()
    }
}

impl<T> Default for Deck<T, StdRng> {
    fn default() -> Self {
        Self {
            deck: Default::default(),
            discard: Default::default(),
            rng: StdRng::from_entropy(),
        }
    }
}

impl<T, R> Deck<T, R>
where
    R: Rng,
{
    pub fn draw(&mut self) -> Option<T> {
        let next = self.deck.pop();
        if let None = next {
            self.discard.shuffle(&mut self.rng);
            self.deck.extend(self.discard.drain(..));
            self.deck.pop()
        } else {
            next
        }
    }

    pub fn discard(&mut self, t: T) {
        self.discard.push(t);
    }

    pub fn entries(&self) -> (&[T], &[T]) {
        (&self.deck, &self.discard)
    }
}

impl<T, R> Deck<T, R>
where
    R: Rng,
    T: PartialEq,
{
    pub fn contains(&self, t: &T) -> bool {
        self.deck.contains(t) || self.discard.contains(t)
    }

    pub fn remove(&mut self, t: T) {
        self.deck.retain(|e| e != &t);
        self.discard.retain(|e| e != &t);
    }
}
