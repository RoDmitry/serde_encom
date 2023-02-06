mod init;
mod map_key;
mod scratch;
mod scratch_map_key;

pub use self::init::InitDeserializer;
pub(crate) use self::map_key::MapKeyDeserializer;
pub(crate) use self::scratch::ScratchDeserializer;
pub(crate) use self::scratch_map_key::ScratchMapKeyDeserializer;

#[macro_export]
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
