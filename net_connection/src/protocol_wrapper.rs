use serde_json;

use super::protocol::Protocol;

#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolWrapper {
    RawProtocol(Protocol),

    RequestState,
    State(StateData),

    RequestDefaultConfig,
    DefaultConfig(ConfigData),

    SetConfig(ConfigData),

    Connect(ConnectData),

    PeerConnected(PeerConnectData),

    SendMessage(SendData),
    HandleSend(HandleSendData),
}

impl<'a> From<&'a Protocol> for ProtocolWrapper {
    fn from(p: &'a Protocol) -> Self {
        if let Protocol::Json(json) = p {
            let json: serde_json::Value = json.into();
            let method = &json["method"];

            if method == "requestState" {
                return ProtocolWrapper::RequestState;
            } else if method == "state" {
                return ProtocolWrapper::State(StateData {
                    state: match json["state"].as_str() {
                        Some(s) => s.to_string(),
                        None => "undefined".to_string(),
                    },
                    id: match json["id"].as_str() {
                        Some(s) => s.to_string(),
                        None => "undefined".to_string(),
                    },
                    bindings: match json["bindings"].as_array() {
                        Some(arr) => arr
                            .iter()
                            .map(|i| match i.as_str() {
                                Some(b) => b.to_string(),
                                None => "undefined".to_string(),
                            })
                            .collect(),
                        None => Vec::new(),
                    },
                });
            } else if method == "requestDefaultConfig" {
                return ProtocolWrapper::RequestDefaultConfig;
            } else if method == "defaultConfig" {
                assert!(json["config"].is_string());
                return ProtocolWrapper::DefaultConfig(ConfigData {
                    config: json["config"].as_str().unwrap().to_string(),
                });
            } else if method == "setConfig" {
                assert!(json["config"].is_string());
                return ProtocolWrapper::SetConfig(ConfigData {
                    config: json["config"].as_str().unwrap().to_string(),
                });
            } else if method == "connect" {
                assert!(json["address"].is_string());
                return ProtocolWrapper::Connect(ConnectData {
                    address: json["address"].as_str().unwrap().to_string(),
                });
            } else if method == "peerConnected" {
                assert!(json["id"].is_string());
                return ProtocolWrapper::PeerConnected(PeerConnectData {
                    id: json["id"].as_str().unwrap().to_string(),
                });
            } else if method == "send" {
                assert!(json["_id"].is_string());
                assert!(json["toAddress"].is_string());
                return ProtocolWrapper::SendMessage(SendData {
                    msg_id: json["_id"].as_str().unwrap().to_string(),
                    to_address: json["toAddress"].as_str().unwrap().to_string(),
                    data: json["data"].clone(),
                });
            } else if method == "handleSend" {
                assert!(json["_id"].is_string());
                assert!(json["toAddress"].is_string());
                assert!(json["fromAddress"].is_string());
                return ProtocolWrapper::HandleSend(HandleSendData {
                    msg_id: json["_id"].as_str().unwrap().to_string(),
                    to_address: json["toAddress"].as_str().unwrap().to_string(),
                    from_address: json["fromAddress"].as_str().unwrap().to_string(),
                    data: json["data"].clone(),
                });
            }
        }

        ProtocolWrapper::RawProtocol(p.clone())
    }
}

impl From<Protocol> for ProtocolWrapper {
    fn from(p: Protocol) -> Self {
        ProtocolWrapper::from(&p)
    }
}

impl<'a> From<&'a ProtocolWrapper> for Protocol {
    fn from(w: &'a ProtocolWrapper) -> Self {
        match w {
            ProtocolWrapper::RawProtocol(p) => p.clone(),
            ProtocolWrapper::RequestState => Protocol::Json(
                json!({
                    "method": "requestState",
                }).into(),
            ),
            ProtocolWrapper::State(s) => Protocol::Json(
                json!({
                    "method": "state",
                    "state": s.state,
                    "id": s.id,
                    "bindings": s.bindings,
                }).into(),
            ),
            ProtocolWrapper::RequestDefaultConfig => Protocol::Json(
                json!({
                    "method": "requestDefaultConfig",
                }).into(),
            ),
            ProtocolWrapper::DefaultConfig(c) => Protocol::Json(
                json!({
                    "method": "defaultConfig",
                    "config": c.config,
                }).into(),
            ),
            ProtocolWrapper::SetConfig(c) => Protocol::Json(
                json!({
                    "method": "setConfig",
                    "config": c.config,
                }).into(),
            ),
            ProtocolWrapper::Connect(c) => Protocol::Json(
                json!({
                    "method": "connect",
                    "address": c.address,
                }).into(),
            ),
            ProtocolWrapper::PeerConnected(c) => Protocol::Json(
                json!({
                    "method": "peerConnected",
                    "id": c.id,
                }).into(),
            ),
            ProtocolWrapper::SendMessage(m) => Protocol::Json(
                json!({
                    "method": "send",
                    "_id": m.msg_id,
                    "toAddress": m.to_address,
                    "data": m.data,
                }).into(),
            ),
            ProtocolWrapper::HandleSend(m) => Protocol::Json(
                json!({
                    "method": "handleSend",
                    "_id": m.msg_id,
                    "toAddress": m.to_address,
                    "fromAddress": m.from_address,
                    "data": m.data,
                }).into(),
            ),
        }
    }
}

impl From<ProtocolWrapper> for Protocol {
    fn from(w: ProtocolWrapper) -> Self {
        Protocol::from(&w)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StateData {
    pub state: String,
    pub id: String,
    pub bindings: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ConfigData {
    pub config: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ConnectData {
    pub address: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PeerConnectData {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SendData {
    pub msg_id: String,
    pub to_address: String,
    pub data: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HandleSendData {
    pub msg_id: String,
    pub to_address: String,
    pub from_address: String,
    pub data: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_convert_request_state() {
        let p: Protocol = ProtocolWrapper::RequestState.into();
        assert_eq!("{\"method\":\"requestState\"}", &p.as_json_string());
        let w: ProtocolWrapper = p.into();
        assert_eq!(ProtocolWrapper::RequestState, w);
    }

    #[test]
    fn it_can_convert_state() {
        let orig = ProtocolWrapper::State(StateData {
            state: "test_state".to_string(),
            id: "test_id".to_string(),
            bindings: vec![
                "test_b1".to_string(),
                "test_b2".to_string(),
            ]
        });
        let p: Protocol = (&orig).into();
        let w: ProtocolWrapper = p.into();
        assert_eq!(orig, w);
    }

    #[test]
    fn it_can_convert_request_default_config() {
        let p: Protocol = ProtocolWrapper::RequestDefaultConfig.into();
        let w: ProtocolWrapper = p.into();
        assert_eq!(ProtocolWrapper::RequestDefaultConfig, w);
    }

    #[test]
    fn it_can_convert_default_config() {
        let orig = ProtocolWrapper::DefaultConfig(ConfigData {
            config: "test_config".to_string(),
        });
        let p: Protocol = (&orig).into();
        let w: ProtocolWrapper = p.into();
        assert_eq!(orig, w);
    }

    #[test]
    fn it_can_convert_set_config() {
        let orig = ProtocolWrapper::SetConfig(ConfigData {
            config: "test_config".to_string(),
        });
        let p: Protocol = (&orig).into();
        let w: ProtocolWrapper = p.into();
        assert_eq!(orig, w);
    }
}
