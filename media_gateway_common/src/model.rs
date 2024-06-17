use savant_core::message::Message;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::ser::SerializeStruct;

#[derive(Debug)]
pub struct Media {
    pub message: Box<Message>,
    pub topic: Vec<u8>,
    pub data: Vec<Vec<u8>>,
}

impl Serialize for Media {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut result = serializer.serialize_struct("Media", 3)?;
        // TODO: implement serialization
        result.serialize_field("message", "message");
        result.serialize_field("topic",  std::str::from_utf8(&self.topic).unwrap())?;
        result.serialize_field("data", &self.data)?;
        result.end()
    }
}

impl<'de> Deserialize<'de> for Media {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        //TODO: implement deserialization
        let media = Media {
            message: Box::new(Message::unknown(String::from("message"))),
            topic: "topic".as_bytes().to_vec(),
            data: vec![],
        };
        Ok(media)
    }
}
