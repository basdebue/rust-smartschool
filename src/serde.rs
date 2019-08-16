//! Custom [`Serialize`](serde::ser::Serialize) and
//! [`Deserialize`](serde::de::Deserialize) implementations.

use crate::mydoc::{CustomFolderId, FolderColor, FolderId};
use serde::{
    de::{self, Deserialize, Deserializer, Visitor},
    ser::{Serialize, Serializer},
};
use std::{fmt, str::FromStr};
use uuid::Uuid;

impl<'de> Deserialize<'de> for FolderId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_string(FolderIdVisitor)
    }
}

struct FolderIdVisitor;

impl<'de> Visitor<'de> for FolderIdVisitor {
    type Value = FolderId;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a valid folder id")
    }

    fn visit_str<E: de::Error>(self, s: &str) -> Result<Self::Value, E> {
        match s {
            "" => Ok(FolderId::Root),
            "favourites" => Ok(FolderId::Favorites),
            "trashed" => Ok(FolderId::Trashed),
            _ => Uuid::from_str(s)
                .map(CustomFolderId::from)
                .map(FolderId::Custom)
                .map_err(|_| {
                    de::Error::invalid_value(
                        de::Unexpected::Str(s),
                        &"``, `favourites`, `trashed` or a valid uuid",
                    )
                }),
        }
    }
}

impl Serialize for FolderId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            FolderId::Custom(uuid) => serializer.serialize_newtype_struct("Uuid", uuid),
            FolderId::Favorites => serializer.serialize_str("favourites"),
            FolderId::Root => serializer.serialize_str(""),
            FolderId::Trashed => serializer.serialize_str("trashed"),
        }
    }
}

pub enum Json<'a> {
    FolderColor(FolderColor),
    FolderId(FolderId),
    Str(&'a str),
}

impl<'a> Serialize for Json<'a> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Json::FolderColor(color) => color.serialize(serializer),
            Json::FolderId(id) => id.serialize(serializer),
            Json::Str(s) => serializer.serialize_str(s),
        }
    }
}
