extern crate futures;
#[macro_use]
extern crate lazy_static;
extern crate riker;
extern crate riker_default;
extern crate riker_patterns;

pub mod net_protocol;
pub use crate::net_protocol::NetProtocol;

pub mod net_actor_system;
pub use crate::net_actor_system::NET_ACTOR_SYS;

#[cfg(test)]
mod tests {
    use super::*;

    use futures::executor::block_on;
    use riker::actor::{
        Actor,
        ActorRef,
        ActorRefFactory,
        BoxActor,
        BoxActorProd,
        Context,
        Props,
        TryTell,
    };
    use riker_patterns::ask::ask;

    struct A;

    impl Actor for A {
        type Msg = NetProtocol;

        fn receive(
            &mut self,
            ctx: &Context<Self::Msg>,
            msg: Self::Msg,
            sender: Option<ActorRef<Self::Msg>>
        ) {
            match msg {
                NetProtocol::NamedJson(name, json) => {
                    assert_eq!("a".to_string(), name);
                    assert_eq!("b".to_string(), json);
                    sender.try_tell(NetProtocol::NamedJson(
                            "c".into(), "d".into()), Some(ctx.myself())).unwrap();
                }
            }
        }
    }

    impl A {
        fn actor() -> BoxActor<NetProtocol> {
            Box::new(A)
        }

        fn props() -> BoxActorProd<NetProtocol> {
            Props::new(Box::new(A::actor))
        }
    }

    #[test]
    fn it_receives() {
        let a = NET_ACTOR_SYS.actor_of(A::props(), "a").unwrap();

        let res = ask(&(*NET_ACTOR_SYS), &a, NetProtocol::NamedJson(
                "a".into(), "b".into()));

        let res = block_on(res).unwrap();
        match res {
            NetProtocol::NamedJson(name, json) => {
                assert_eq!("c".to_string(), name);
                assert_eq!("d".to_string(), json);
            }
        }
    }
}
