use serde_bytes;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Protocol {
    NamedBinary(NamedBinaryData),
    Json(#[serde(with = "serde_bytes")] Vec<u8>),
    Ping(PingData),
    Pong(PongData),
}

impl<'a> From<&'a str> for Protocol {
    fn from(s: &'a str) -> Self {
        Protocol::Json(s.as_bytes().to_vec())
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
        is_json as_json Json Vec<u8>
        is_ping as_ping Ping PingData
        is_pong as_pong Pong PongData
    }

    pub fn as_json_string(&self) -> String {
        if let Protocol::Json(data) = self {
            String::from_utf8_lossy(&data).to_string()
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
            let wire = rmp_serde::to_vec($e).unwrap();
            let res: Protocol = rmp_serde::from_slice(&wire).unwrap();
            res
        }}
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
        let json = "{\"test\": \"hello\"}".to_string();

        let res = simple_convert!(&Protocol::Json(json.as_bytes().to_vec()));

        assert!(res.is_json());

        let res = String::from_utf8_lossy(res.as_json());

        assert_eq!(json, res);
    }

    #[test]
    fn it_can_convert_ping() {
        let src = Protocol::Ping(PingData {
            sent: 42.0,
        });

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
