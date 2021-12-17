pub trait Serializable {
    type Error;
    fn serialize(&self) -> Result<Vec<u8>, Self::Error>;
    fn deserialize(bytes: &[u8]) -> Result<Self, Self::Error>
    where
        Self: Sized;
}
