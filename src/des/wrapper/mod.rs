macro_rules! to_inner_des_method {
    ($method:ident) => {
        #[inline]
        fn $method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: de::Visitor<'de>,
        {
            self.des.$method(visitor)
        }
    };
}

mod init;
mod map_key;
mod saved_map_key;
mod saved_seq;

pub use self::init::InitDeserializer;
pub(crate) use self::map_key::MapKeyDeserializer;
pub(crate) use self::saved_map_key::SavedMapKeyDeserializer;
pub(crate) use self::saved_seq::SavedSeqDeserializer;
