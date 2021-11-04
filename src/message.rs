use std::fmt;
#[repr(u8)]
#[derive(Debug)]
pub enum MessageId {
    CmdReset,
    CmdTph,
    CmdTemp,
    CmdPress,
    CmdHum,
    RspTph,
    RspTemp,
    RspPress,
    RspHum,
}
// TODO: Implement fmt::Display
pub struct IdError;

impl MessageId {
    pub fn value(&self) -> u8 {
        match *self {
            MessageId::CmdReset => 0x01,
            MessageId::CmdTph => 0x02,
            MessageId::CmdTemp => 0x03,
            MessageId::CmdPress => 0x04,
            MessageId::CmdHum => 0x05,
            MessageId::RspTph => 0x06,
            MessageId::RspTemp => 0x07,
            MessageId::RspPress => 0x08,
            MessageId::RspHum => 0x09,
        }
    }

    pub fn from_value(value: &u8) -> Result<MessageId, IdError> {
        match value {
            0x01 => Ok(MessageId::CmdReset),
            0x02 => Ok(MessageId::CmdTph),
            0x03 => Ok(MessageId::CmdTemp),
            0x04 => Ok(MessageId::CmdPress),
            0x05 => Ok(MessageId::CmdHum),
            0x06 => Ok(MessageId::RspTph),
            0x07 => Ok(MessageId::RspTemp),
            0x08 => Ok(MessageId::RspPress),
            0x09 => Ok(MessageId::RspHum),
            _ => Err(IdError),
        }
    }
}

pub struct Message {
    id: MessageId,
    payload: Vec<u8>,
}

pub struct MessageError;
impl Message {
    fn new(id: MessageId) -> Message {
        Message {
            id,
            payload: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.payload.len() + 1
    }

    fn id(&self) -> &MessageId {
        &self.id
    }

    fn deserialize(bytes: Vec<u8>) -> Result<Message, MessageError> {
        Ok(Message {
            id: match bytes.get(0) {
                Some(id) => match MessageId::from_value(id) {
                    Ok(msgid) => msgid,
                    Err(e) => return Err(MessageError),
                },
                None => return Err(MessageError),
            },
            payload: bytes[1..].to_vec(),
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.push(self.id.value());
        for n in &self.payload {
            v.push(*n);
        }
        v
    }

    fn payload(&self) -> &Vec<u8> {
        &self.payload
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_id(n: &u8, exp: MessageId) {
        let id = match MessageId::from_value(&n) {
            Ok(t) => t,
            Err(e) => panic!("Failed to convert value to id"),
        };

        match id {
            exp => return,
            _ => panic!("Converted to wrong id"),
        }
    }
    // New messages are empty
    #[test]
    fn test_new_size() {
        let msg = Message::new(MessageId::CmdTph);

        assert_eq!(1, msg.len());
    }

    #[test]
    fn test_get_msgid() {
        let msg = Message::new(MessageId::CmdTph);
        let r = match msg.id() {
            MessageId::CmdTph => true,
            _ => false,
        };

        assert!(r);
    }

    #[test]
    #[should_panic]
    fn test_msgid_bad_value() {
        let n = 0x45;
        let id = {
            let this = MessageId::from_value(&n);
            match this {
                Ok(t) => t,
                Err(e) => panic!("Failed to convert value to id"),
            }
        };
    }
    #[test]
    fn test_msgid_from_value() {
        let mut n = 0x01;
        check_id(&n, MessageId::CmdReset);
        n = 0x02;
        check_id(&n, MessageId::CmdTph);
        n = 0x03;
        check_id(&n, MessageId::CmdTemp);
        n = 0x04;
        check_id(&n, MessageId::CmdPress);
        n = 0x05;
        check_id(&n, MessageId::CmdHum);
        n = 0x06;
        check_id(&n, MessageId::RspTph);
        n = 0x07;
        check_id(&n, MessageId::RspTemp);
        n = 0x08;
        check_id(&n, MessageId::RspPress);
        n = 0x09;
        check_id(&n, MessageId::RspHum);
    }

    #[test]
    fn test_deserialize() {
        let v: Vec<u8> = vec![0x06, 0x02];
        let msg = match Message::deserialize(v) {
            Ok(m) => m,
            Err(_) => panic!("Failed to deserialize"),
        };
        assert_eq!(0x06, msg.id().value());
        assert_eq!(0x02, *msg.payload().get(0).unwrap());
    }
    #[test]
    fn test_serialize() {
        let msg = Message::new(MessageId::CmdTph);
        let v = msg.serialize();
        assert_eq!(0x02, *v.get(0).unwrap());
        assert_eq!(0x02, msg.id().value());
    }
}
