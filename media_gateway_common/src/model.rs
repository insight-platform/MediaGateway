//! Models for media gateway client-server communication.
//!
//! The module provides [`Media`] struct that can be converted from/to
//! [protocol buffers](https://protobuf.dev/).
use savant_protobuf::generated::Message;

/// A struct that contains all information required to forward a message.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Media {
    /// A message to be forwarded
    #[prost(message, optional, tag = "1")]
    pub message: ::core::option::Option<Message>,
    /// A topic
    #[prost(bytes = "vec", tag = "2")]
    pub topic: ::prost::alloc::vec::Vec<u8>,
    /// Extra data sent with the message
    #[prost(bytes = "vec", repeated, tag = "3")]
    pub data: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}

impl Media {
    /// Serializes the struct to protocol buffers.
    pub fn to_proto(&self) -> anyhow::Result<Vec<u8>> {
        use prost::Message as ProstMessage;
        let mut buf = Vec::new();
        self.encode(&mut buf)?;
        Ok(buf)
    }

    /// Deserializes the struct from protocol buffers.
    pub fn from_proto(bytes: &[u8]) -> anyhow::Result<Self> {
        use prost::Message as ProstMessage;
        let media = Media::decode(bytes)?;
        Ok(media)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use savant_protobuf::generated::message::Content;
    use savant_protobuf::generated::{Message, Unknown};

    use crate::model::Media;

    #[test]
    fn to_from_proto() {
        let message = Message {
            protocol_version: "protocol_version".to_string(),
            routing_labels: vec!["label1".to_string(), "label2".to_string()],
            propagated_context: HashMap::from([("key".to_string(), "value".to_string())]),
            seq_id: 100,
            content: Option::from(Content::Unknown(Unknown {
                message: "message".to_string(),
            })),
        };
        let original_media = Media {
            message: Option::from(message),
            topic: "topic".as_bytes().to_vec(),
            data: vec![vec![1]],
        };
        let bytes = original_media.to_proto().expect("to_proto failed");
        let result_media = Media::from_proto(&bytes).expect("from_proto failed");
        assert_eq!(original_media, result_media);
    }
}
