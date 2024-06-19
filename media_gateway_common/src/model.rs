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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use savant_protobuf::generated::{Message, Unknown};
    use savant_protobuf::generated::message::Content;

    use crate::model::Media;

    #[test]
    fn to_from_proto() {
        let message = Message {
            protocol_version: "protocol_version".to_string(),
            routing_labels: vec!["label1".to_string(), "label2".to_string()],
            propagated_context: HashMap::from([("key".to_string(), "value".to_string())]),
            seq_id: 100,
            content: Option::from(Content::Unknown(Unknown { message: "message".to_string() })),
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
