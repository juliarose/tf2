use protobuf::Message;
use steam_vent::{
    net::NetworkError,
    connection::Connection,
    proto::{
        steammessages_base::CMsgProtoBufHeader,
    },
    net::PROTO_MASK,
    game_coordinator::ClientToGCMessage,
};
use tf2_protobuf::{
    econ_gcmessages::EGCItemMsg,
    base_gcmessages::{
        CMsgUseItem,
        CMsgGCRemoveCustomizationAttributeSimple,
    },
};
use byteorder::{LittleEndian, WriteBytesExt};
use bytes::{BufMut, BytesMut};
use std::io::Write;
use crate::app::App;

pub const JOBID_NONE: u64 = u64::MAX;

pub struct TF2 {
    source_job_id: u64,
}

impl App for TF2 {
    const APPID: u32 = 440;
}

impl TF2 {
    
    pub fn new() -> Self {
        Self {
            source_job_id: 0,
        }
    }
    
    fn next_jobid(&mut self) -> u64 {
        self.source_job_id += 1;
        self.source_job_id
    }
    
    async fn send(
        &self,
        connection: &mut Connection,
        msg: ClientToGCMessage,
    ) -> Result<u64, NetworkError> {
        connection.send_gc(msg).await
    }
    
    pub async fn remove_gifted_by(
        &mut self,
        connection: &mut Connection,
        item_id: u64,
    ) -> Result<u64, NetworkError> {
        let msgtype = EGCItemMsg::k_EMsgGCRemoveGiftedBy as i32;
        let mut msg = ClientToGCMessage::new(Self::APPID, msgtype);
        let mut message = CMsgGCRemoveCustomizationAttributeSimple::new();
        
        message.set_item_id(item_id);
        msg.0.set_payload(self.proto_payload(
            message,
            msgtype,
        )?);
        self.send(connection, msg).await
    }
    
    pub async fn craft(
        &mut self,
        connection: &mut Connection,
        items: &[u64],
    ) -> Result<u64, NetworkError> {
        self.craft_recipe(connection,-2, items).await
    }
    
    pub async fn craft_recipe(
        &mut self,
        connection: &mut Connection,
        recipe: i16,
        items: &[u64],
    ) -> Result<u64, NetworkError> {
        let msgtype = EGCItemMsg::k_EMsgGCCraft as i32;
        let mut msg = ClientToGCMessage::new(Self::APPID, msgtype);
        let mut buff = BytesMut::with_capacity(
            2 + 2 + (8 * items.len())
        );
        let mut writer = (&mut buff).writer();
        
        writer.write_i16::<LittleEndian>(recipe)?;
        writer.write_i16::<LittleEndian>(items.len() as i16)?;
        
        for item in items {
            writer.write_u64::<LittleEndian>(*item)?;
        }
        
        let payload = self.payload(
            buff,
        )?;
        
        msg.0.set_payload(payload);
        
        self.send(connection, msg).await
    }
    
    pub async fn use_item(
        &mut self,
        connection: &mut Connection,
        item: u64,
    ) -> Result<u64, NetworkError> {
        let msgtype = EGCItemMsg::k_EMsgGCUseItemRequest as i32;
        let mut msg = ClientToGCMessage::new(Self::APPID, msgtype);
        let mut message = CMsgUseItem::new();
        
        message.set_item_id(item);
        msg.0.set_payload(self.proto_payload(
            message,
            msgtype,
        )?);
        self.send(connection, msg).await
    }
    
    fn proto_payload<Msg: Message>(
        &mut self,
        message: Msg,
        msg_type: i32,
    ) -> Result<Vec<u8>, std::io::Error> {
        let source_job_id = self.next_jobid();
        let mut buff = BytesMut::with_capacity(
            Self::proto_encode_size(source_job_id) + message.compute_size() as usize
        );
        let mut writer = (&mut buff).writer();
        
        Self::write_proto_header(&mut writer, msg_type, source_job_id)?;
        
        message.write_to_writer(&mut writer)?;
    
        Ok(buff.to_vec())
    }
    
    fn payload(
        &mut self,
        message: BytesMut,
    ) -> Result<Vec<u8>, std::io::Error> {
        let source_job_id = self.next_jobid();
        let mut buff = BytesMut::with_capacity(
            Self::encode_size() + message.len() as usize
        );
        let mut writer = (&mut buff).writer();
        
        Self::write_header(&mut writer, source_job_id)?;
        
        writer.write(&message[..])?;
    
        Ok(buff.to_vec())
    }
    
    fn write_proto_header<W: WriteBytesExt>(
        writer: &mut W,
        msg_type: i32,
        source_job_id: u64,
    ) -> std::io::Result<()> {
        let mut proto_header = CMsgProtoBufHeader::new();
        // proto_header.set_jobid_target(self.target_job_id);
        proto_header.set_jobid_source(source_job_id);
        
        writer.write_u32::<LittleEndian>(msg_type as u32 | PROTO_MASK)?;
        writer.write_u32::<LittleEndian>(proto_header.compute_size())?;
        proto_header.write_to_writer(writer)?;
        
        Ok(())
    }
    
    fn proto_encode_size(source_job_id: u64) -> usize {
        let mut proto_header = CMsgProtoBufHeader::new();
        proto_header.set_jobid_source(source_job_id);
        
        4 + 4 + proto_header.compute_size() as usize
    }
    
    fn write_header<W: WriteBytesExt>(
        writer: &mut W,
        source_job_id: u64,
    ) -> std::io::Result<()> {
        writer.write_u16::<LittleEndian>(1)?;
        writer.write_u64::<LittleEndian>(JOBID_NONE)?;
        writer.write_u64::<LittleEndian>(source_job_id)?;
        Ok(())
    }
    
    fn encode_size() -> usize {
        2 + 8 + 8 + 4
    }
}
