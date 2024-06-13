use savant_core::message::Message;
use serde::{Deserialize, Deserializer};

#[derive(Debug)]
pub struct Media {
    pub message: Box<Message>,
    pub topic: Vec<u8>,
    pub data: Vec<Vec<u8>>,
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
