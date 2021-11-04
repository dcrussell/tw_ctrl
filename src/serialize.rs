pub trait Serializable {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize<T, E>(bytes: Vec<u8>)
}
