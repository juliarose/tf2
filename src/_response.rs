use protobuf::{Message, ProtobufResult};
use std::fmt::Debug;
use std::io::Read;
use tf2_protobuf::{
    econ_gcmessages::EGCItemMsg,
    base_gcmessages::CMsgGCRemoveCustomizationAttributeSimple,
    gcsdk_gcmessages,
    gcsystemmsgs::ESOMsg,
};

pub fn send_message(appid: u32, msg_type: EGCItemMsg) {
    let mut message = CMsgGCRemoveCustomizationAttributeSimple::new();
    
    message.set_item_id(1);
    
    send(appid, msg_type, message);
}

fn send<T>(appid: u32, msg_type: EGCItemMsg, msg: T)
where T: Message {
    
}


pub trait GCItemRequest: Debug + Message {
    const APPID: u32;
    const MSG_TYPE: EGCItemMsg;
    const RESPONSE: Option<EGCItemMsg>;
    // type Response: GCItemResponse;
}

// pub trait GCItemResponse: Debug + Sized {
//     fn parse_from_reader(reader: &mut dyn Read) -> ProtobufResult<Self>;
// }

// impl GCItemResponse for () {
//     fn parse_from_reader(_reader: &mut dyn Read) -> ProtobufResult<Self> {
//         Ok(())
//     }
// }

macro_rules! gc_message {
    ($appid:expr, $msg_type:expr => $req:path, $res:expr) => {
        impl GCItemRequest for $req {
            const APPID: u32 = $appid;
            const MSG_TYPE: EGCItemMsg = $msg_type;
            const RESPONSE: Option<EGCItemMsg> = Some($res);
            // type Response = $res;
        }

        // impl GCItemResponse for $res {
        //     fn parse_from_reader(reader: &mut dyn Read) -> ProtobufResult<Self> {
        //         <Self as Message>::parse_from_reader(reader)
        //     }
        // }
    };
    ($appid:expr, $msg_type:expr => $req:path) => {
        impl GCItemRequest for $req {
            const APPID: u32 = $appid;
            const MSG_TYPE: Option<EGCItemMsg> = $msg_type;
            const RESPONSE = None;
            // type Response = ();
        }
    };
}

gc_message!(440, EGCItemMsg::k_EMsgGCRemoveGiftedBy => CMsgGCRemoveCustomizationAttributeSimple, EGCItemMsg::k_EMsgGCRemoveGiftedByResponse);