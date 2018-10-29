use riker::actors::ActorMsg;

type MsgType = String;
type JsonString = String;

#[derive(Clone, Debug)]
pub enum NetProtocol {
    NamedJson(MsgType, JsonString),
}

impl Into<ActorMsg<NetProtocol>> for NetProtocol {
    fn into(self) -> ActorMsg<NetProtocol> {
        ActorMsg::User(self)
    }
}
