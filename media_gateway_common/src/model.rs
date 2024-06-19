use savant_protobuf::generated::Message;

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Media {
    #[prost(message, optional, tag = "1")]
    pub message: ::core::option::Option<Message>,
    #[prost(bytes = "vec", tag = "2")]
    pub topic: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", repeated, tag = "3")]
    pub data: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}

impl Media {
    pub fn to_proto(&self) -> anyhow::Result<Vec<u8>> {
        use prost::Message as ProstMessage;
        let mut buf = Vec::new();
        self.encode(&mut buf)?;
        Ok(buf)
    }

    pub fn from_proto(bytes: &[u8]) -> anyhow::Result<Self> {
        use prost::Message as ProstMessage;
        let media = Media::decode(bytes)?;
        Ok(media)
    }
}
