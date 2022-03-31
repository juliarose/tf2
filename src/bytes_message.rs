use bytes::BytesMut;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor, Read, Write};
use std::fmt::Debug;

pub type GCBytesMessageError = std::io::Error;

pub trait GCBytesMessage: Debug + Sized {
    
    fn from_payload(payload: BytesMut) -> Result<Self, GCBytesMessageError>;
}

#[derive(Debug)]
pub struct CraftResponse {
    pub blueprint: i16,
    pub assetids: Vec<u64>,
}

impl GCBytesMessage for CraftResponse {
    
    fn from_payload(payload: BytesMut) -> Result<Self, GCBytesMessageError> {
        let mut reader = Cursor::new(payload);
        let blueprint = reader.read_i16::<LittleEndian>()?;
        let _ = reader.read_u32::<LittleEndian>()?; // unknown
        let id_count = reader.read_u16::<LittleEndian>()? as usize;
        let mut assetids = Vec::with_capacity(id_count);
        
        for _i in 0..id_count {
            assetids.push(reader.read_u64::<LittleEndian>()?);
        }
        
        Ok(Self {
            blueprint,
            assetids,
        })
    }
}