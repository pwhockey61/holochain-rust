use rmp_serde;
use serde_bytes;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct JsonString(String);

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    NamedBinary(NamedBinaryData),
    Json(JsonString),
    Ping(PingData),
    Pong(PongData),
}

impl<'a> From<&'a Protocol> for NamedBinaryData {
    fn from(p: &'a Protocol) -> Self {
        match p {
            Protocol::NamedBinary(nb) => NamedBinaryData {
                name: b"namedBinary".to_vec(),
                data: rmp_serde::to_vec_named(nb).unwrap(),
            },
            Protocol::Json(j) => NamedBinaryData {
                name: b"json".to_vec(),
                data: j.0.as_bytes().to_vec(),
            },
            Protocol::Ping(p) => NamedBinaryData {
                name: b"ping".to_vec(),
                data: rmp_serde::to_vec_named(p).unwrap(),
            },
            Protocol::Pong(p) => NamedBinaryData {
                name: b"pong".to_vec(),
                data: rmp_serde::to_vec_named(p).unwrap(),
            },
        }
    }
}

impl From<Protocol> for NamedBinaryData {
    fn from(p: Protocol) -> Self {
        (&p).into()
    }
}

impl<'a> From<&'a NamedBinaryData> for Protocol {
    fn from(nb: &'a NamedBinaryData) -> Self {
        match nb.name.as_slice() {
            b"namedBinary" => {
                let sub: NamedBinaryData = rmp_serde::from_slice(&nb.data).unwrap();
                Protocol::NamedBinary(sub)
            }
            b"json" => Protocol::Json(JsonString(String::from_utf8_lossy(&nb.data).to_string())),
            b"ping" => {
                let sub: PingData = rmp_serde::from_slice(&nb.data).unwrap();
                Protocol::Ping(sub)
            }
            b"pong" => {
                let sub: PongData = rmp_serde::from_slice(&nb.data).unwrap();
                Protocol::Pong(sub)
            }
            _ => panic!("bad Protocol type: {}", String::from_utf8_lossy(&nb.name)),
        }
    }
}

impl From<NamedBinaryData> for Protocol {
    fn from(nb: NamedBinaryData) -> Self {
        (&nb).into()
    }
}

impl<'a> From<&'a str> for Protocol {
    fn from(s: &'a str) -> Self {
        Protocol::Json(JsonString(s.to_string()))
    }
}

impl From<String> for Protocol {
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

impl From<Protocol> for String {
    fn from(p: Protocol) -> String {
        p.as_json_string()
    }
}

macro_rules! simple_access {
    ($($is:ident $as:ident $d:ident $t:ty)*) => {
        $(
            pub fn $is(&self) -> bool {
                if let Protocol::$d(_) = self {
                    true
                } else {
                    false
                }
            }

            pub fn $as<'a>(&'a self) -> &'a $t {
                if let Protocol::$d(data) = self {
                    &data
                } else {
                    panic!(concat!(stringify!($as), " called with bad type"));
                }
            }
        )*
    }
}

impl Protocol {
    simple_access! {
        is_named_binary as_named_binary NamedBinary NamedBinaryData
        is_json as_json Json JsonString
        is_ping as_ping Ping PingData
        is_pong as_pong Pong PongData
    }

    pub fn as_json_string(&self) -> String {
        if let Protocol::Json(data) = self {
            data.0.clone()
        } else {
            panic!("as_json_string called with bad type");
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NamedBinaryData {
    #[serde(with = "serde_bytes")]
    pub name: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PingData {
    pub sent: f64,
}

/*
impl Serialize for PingData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("sent", &self.sent)?;
        map.end()
    }
}
*/

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PongData {
    pub orig: f64,
    pub recv: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmp_serde;

    macro_rules! simple_convert {
        ($e:expr) => {{
            let wire: NamedBinaryData = $e.into();
            let res: Protocol = wire.into();
            res
        }};
    }

    #[test]
    fn it_can_convert_named_binary() {
        let nb_src = Protocol::NamedBinary(NamedBinaryData {
            name: b"test".to_vec(),
            data: b"hello".to_vec(),
        });

        let res = simple_convert!(&nb_src);

        assert!(res.is_named_binary());

        let res = res.as_named_binary();

        assert_eq!(b"test".to_vec(), res.name);
        assert_eq!(b"hello".to_vec(), res.data);
    }

    #[test]
    fn it_can_convert_json() {
        let json_str = "{\"test\": \"hello\"}";
        let json: Protocol = json_str.into();

        let res = simple_convert!(&json);

        assert!(res.is_json());

        let res = res.as_json_string();

        assert_eq!(json_str, res);
    }

    #[test]
    fn it_can_convert_ping() {
        let src = Protocol::Ping(PingData { sent: 42.0 });

        let res = simple_convert!(&src);

        assert!(res.is_ping());

        let res = res.as_ping();

        assert_eq!(42.0, res.sent);
    }

    #[test]
    fn it_can_convert_pong() {
        let src = Protocol::Pong(PongData {
            orig: 42.0,
            recv: 88.0,
        });

        let res = simple_convert!(&src);

        assert!(res.is_pong());

        let res = res.as_pong();

        assert_eq!(42.0, res.orig);
        assert_eq!(88.0, res.recv);
    }
}
