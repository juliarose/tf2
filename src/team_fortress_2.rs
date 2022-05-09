use protobuf::{RepeatedField, Message};
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
        CMsgFulfillDynamicRecipeComponent,
        CMsgRecipeComponent,
        CMsgGCRemoveCustomizationAttributeSimple,
    },
};
use byteorder::{LittleEndian, WriteBytesExt};
use bytes::{BufMut, BytesMut};
use std::io::Write;
use crate::app::App;

pub const JOBID_NONE: u64 = u64::MAX;

#[derive(Debug)]
pub struct TeamFortress2 {
    source_job_id: u64,
}

impl App for TeamFortress2 {
    const APPID: u32 = 440;
}

#[derive(Debug, Clone, PartialEq)]
pub enum ItemCustomization {
    GiftedBy,
    CraftedBy,
    Decal,
    Killstreak,
    Paint,
    Festivizer,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecipeComponent {
    pub subject_item_id: u64,
    pub attribute_index: u64,
}

impl TeamFortress2 {
    
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
    
    pub async fn remove_item_name(
        &mut self,
        connection: &mut Connection,
        item_id: u64,
    ) -> Result<u64, NetworkError> {
        let msgtype = EGCItemMsg::k_EMsgGCRemoveItemName as i32;
        let mut msg = ClientToGCMessage::new(Self::APPID, msgtype, false);
        let mut buff = BytesMut::with_capacity(12);
        let mut writer = (&mut buff).writer();
        
        writer.write_u64::<LittleEndian>(item_id)?;
        writer.write_u32::<LittleEndian>(0)?;
        msg.0.set_payload(self.payload(
            buff,
        )?);
        self.send(connection, msg).await
    }
    
    pub async fn remove_item_description(
        &mut self,
        connection: &mut Connection,
        item_id: u64,
    ) -> Result<u64, NetworkError> {
        let msgtype = EGCItemMsg::k_EMsgGCRemoveItemName as i32;
        let mut msg = ClientToGCMessage::new(Self::APPID, msgtype, false);
        let mut buff = BytesMut::with_capacity(12);
        let mut writer = (&mut buff).writer();
        
        writer.write_u64::<LittleEndian>(item_id)?;
        writer.write_u32::<LittleEndian>(1)?;
        msg.0.set_payload(self.payload(
            buff,
        )?);
        self.send(connection, msg).await
    }
    
    pub async fn remove_customization(
        &mut self,
        connection: &mut Connection,
        item_id: u64,
        item_customization: &ItemCustomization,
    ) -> Result<u64, NetworkError> {
        let msgtype = match item_customization {
            ItemCustomization::GiftedBy => EGCItemMsg::k_EMsgGCRemoveGiftedBy,
            ItemCustomization::CraftedBy => EGCItemMsg::k_EMsgGCRemoveMakersMark,
            ItemCustomization::Decal => EGCItemMsg::k_EMsgGCRemoveCustomTexture,
            ItemCustomization::Killstreak => EGCItemMsg::k_EMsgGCRemoveKillStreak,
            ItemCustomization::Paint => EGCItemMsg::k_EMsgGCRemoveItemPaint,
            ItemCustomization::Festivizer=> EGCItemMsg::k_EMsgGCRemoveFestivizer,
        } as i32;
        let mut msg = ClientToGCMessage::new(Self::APPID, msgtype, true);
        let mut message = CMsgGCRemoveCustomizationAttributeSimple::new();
        
        message.set_item_id(item_id);
        msg.0.set_payload(self.proto_payload(
            message,
            msgtype,
        )?);
        self.send(connection, msg).await
    }
    
    pub async fn use_item(
        &mut self,
        connection: &mut Connection,
        item_id: u64,
    ) -> Result<u64, NetworkError> {
        let msgtype = EGCItemMsg::k_EMsgGCUseItemRequest as i32;
        let mut msg = ClientToGCMessage::new(Self::APPID, msgtype, true);
        let mut message = CMsgUseItem::new();
        
        message.set_item_id(item_id);
        msg.0.set_payload(self.proto_payload(
            message,
            msgtype,
        )?);
        self.send(connection, msg).await
    }
    
    pub async fn fulfill_recipe(
        &mut self,
        connection: &mut Connection,
        item_id: u64,
        components: Vec<RecipeComponent>,
    ) -> Result<u64, NetworkError> {
        let msgtype = EGCItemMsg::k_EMsgGCFulfillDynamicRecipeComponent as i32;
        let mut msg = ClientToGCMessage::new(Self::APPID, msgtype, true);
        let mut message = CMsgFulfillDynamicRecipeComponent::new();
        let components = components
            .into_iter()
            .map(|component| {
                let mut message = CMsgRecipeComponent::new();
                
                message.set_attribute_index(component.attribute_index);
                message.set_subject_item_id(component.subject_item_id);
                
                message
            })
            .collect::<Vec<_>>();
        
        message.set_tool_item_id(item_id);
        message.set_consumption_components(RepeatedField::from_vec(components));
        msg.0.set_payload(self.proto_payload(
            message,
            msgtype,
        )?);
        self.send(connection, msg).await
    }
    
    pub async fn delete_item(
        &mut self,
        connection: &mut Connection,
        item_id: u64,
    ) -> Result<u64, NetworkError> {
        let msgtype = EGCItemMsg::k_EMsgGCDelete as i32;
        let mut msg = ClientToGCMessage::new(Self::APPID, msgtype, false);
        let mut buff = BytesMut::with_capacity(8);
        let mut writer = (&mut buff).writer();
        
        writer.write_u64::<LittleEndian>(item_id)?;
        msg.0.set_payload(self.payload(
            buff,
        )?);
        self.send(connection, msg).await
    }
    
    pub async fn craft(
        &mut self,
        connection: &mut Connection,
        item_ids: &[u64],
    ) -> Result<u64, NetworkError> {
        self.craft_recipe(connection,-2, item_ids).await
    }
    
    pub async fn craft_recipe(
        &mut self,
        connection: &mut Connection,
        recipe: i16,
        item_ids: &[u64],
    ) -> Result<u64, NetworkError> {
        let msgtype = EGCItemMsg::k_EMsgGCCraft as i32;
        let mut msg = ClientToGCMessage::new(Self::APPID, msgtype, false);
        let mut buff = BytesMut::with_capacity(2 + 2 + (8 * item_ids.len()));
        let mut writer = (&mut buff).writer();
        
        writer.write_i16::<LittleEndian>(recipe)?;
        writer.write_i16::<LittleEndian>(item_ids.len() as i16)?;
        
        for item_id in item_ids {
            writer.write_u64::<LittleEndian>(*item_id)?;
        }
        
        msg.0.set_payload(self.payload(
            buff,
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
