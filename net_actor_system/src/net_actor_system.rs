use riker::actors::ActorSystem;
use riker_default::DefaultModel;

use crate::net_protocol::NetProtocol;

lazy_static! {
    pub static ref NET_ACTOR_SYS: ActorSystem<NetProtocol> = {
        let model: DefaultModel<NetProtocol> = DefaultModel::new();
        ActorSystem::new(&model).unwrap()
    };
}
