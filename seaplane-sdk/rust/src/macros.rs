/// Implements Deserialize using FromStr
macro_rules! impl_deser_from_str {
    ($t:ty) => {
        impl<'de> ::serde::Deserialize<'de> for $t {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: ::serde::de::Deserializer<'de>,
            {
                String::deserialize(deserializer)?
                    .parse()
                    .map_err(::serde::de::Error::custom)
            }
        }
    };
}

/// Implements Serialize using to_string
#[cfg(all(feature = "unstable", feature = "compute_api_v2"))]
macro_rules! impl_ser_to_str {
    ($t:ty) => {
        impl ::serde::Serialize for $t {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: ::serde::ser::Serializer,
            {
                serializer.collect_str(self)
            }
        }
    };
}

/// Implements De/Serialize using to_string
#[cfg(all(feature = "unstable", feature = "compute_api_v2"))]
macro_rules! impl_serde_str {
    ($t:ty) => {
        impl_ser_to_str!($t);
        impl_deser_from_str!($t);
    };
}
