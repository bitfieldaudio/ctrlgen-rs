#![feature(generic_associated_types, type_alias_impl_trait)]
use ctrlgen::CallMutAsync;

#[derive(Default)]
struct Service {
    counter: i32,
    flag: bool,
}

#[ctrlgen::ctrlgen(pub enum ServiceMsg,
    returnval = ctrlgen::returnval::LocalRetval,
    proxy = ServiceProxy
)]
impl Service {
    pub fn increment_by(&mut self, arg: i32) -> i32 {
        self.counter += arg;
        self.counter
    }

    pub async fn set_flag(&mut self, flag: bool) {
        self.flag = flag;
    }
}

// impl CallMut<Service> for ServiceMsg
// where
//     TokioRetval: Returnval,
// {
//     type Output = core::result::Result<(), <TokioRetval as Returnval>::SendError>;
//     fn call_mut(self, service: &mut Service) -> Self::Output {
//         match self {
//             ServiceMsg::IncrementBy { arg, ret } => <TokioRetval as Returnval>::send(ret, todo!("call foo")),
//             ServiceMsg::SetFlag { flag } => {
//                 todo!("Call bar");
//                 Ok(())
//             }
//         }
//     }
// }

// struct ServiceProxy<Sender: ctrlgen::MessageSender<ServiceMsg>> {
//     sender: Sender,
// }

// impl<Sender: ctrlgen::MessageSender<ServiceMsg>> ServiceProxy<Sender>
// where
//     TokioRetval: Returnval,
// {
//     pub fn new(sender: Sender) -> Self {
//         Self { sender }
//     }

//     pub fn increment_by(&self, arg: i32) -> <TokioRetval as Returnval>::RecvResult<i32> {
//         let ret = <TokioRetval as Returnval>::create();
//         let msg = ServiceMsg::IncrementBy { arg, ret: ret.0 };
//         self.sender.send(msg);
//         <TokioRetval as Returnval>::recv(ret.1)
//     }

//     pub fn set_flag(&self, flag: bool) {
//         let msg = ServiceMsg::SetFlag { flag };
//         self.sender.send(msg);
//     }
// }

#[tokio::test]
async fn call_mut_works() {
    let mut service = Service {
        counter: 0,
        flag: false,
    };
    let msg = ServiceMsg::SetFlag { flag: true };
    msg.call_mut_async(&mut service).await.unwrap();

    assert!(service.flag)
}
// impl CallMutAsync<Service> for ServiceMsg
// where
//     TokioRetval: AsyncReturnval,
// {
//     type Future = impl Future<Output = Result<(), <TokioRetval as Returnval>::SendError>>;
//     fn call_mut_async(self, service: &mut Service) -> Self::Future {
//         match self {
//             ServiceMsg::Foo { arg, ret } => {
//                 <TokioRetval as Returnval>::send(ret, todo!("call foo"))
//             }
//             ServiceMsg::Bar { flag } => {
//                 todo!("Call bar");
//                 Ok(())
//             }
//         }
//     }
// }
