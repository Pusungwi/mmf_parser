use std::{str::Bytes, result};
use bytestream_rs::bytestream::{ByteStream, ByteStreamError};

struct FileHeader {
    data_size:usize,
    class:u8,
    file_type:u8,
    code_type:u8,
    status:u8,
    counts:u8,
    song_title:String,
    version:u8,
}

trait BlockHeaderBase {
    fn signature(&self) -> String;
    fn size(&self) -> Option<usize>;
}

struct MidiBlockHeader {
    signature:String,
    size:usize, 
}

impl BlockHeaderBase for MidiBlockHeader {
    fn signature(&self) -> String {
        self.signature.clone()
    }

    fn size(&self) -> Option<usize> {
        Some(self.size.clone())
    }
}

struct WaveBlockHeader {
    signature:String,
    size:usize, 
}

impl BlockHeaderBase for WaveBlockHeader {
    fn signature(&self) -> String {
        self.signature.clone()
    }

    fn size(&self) -> Option<usize> {
        Some(self.size.clone())
    }
}

#[derive(Debug, PartialEq)]
pub enum MmfParseResult {
    OK,
    NOT_FOUND_SMAF_HEADER,
    UNKNOWN_ERROR,
}

pub struct MmfFileInfo {
    result:MmfParseResult,
    header:FileHeader,
    midi_blocks:Vec<MidiBlockHeader>,
    wave_blocks:Vec<WaveBlockHeader>,
}

impl MmfFileInfo {
    pub fn new() -> MmfFileInfo {
        MmfFileInfo {
            result: MmfParseResult::UNKNOWN_ERROR,
            header: FileHeader { data_size: 0, class: todo!(), file_type: todo!(), code_type: todo!(), status: todo!(), counts: todo!(), song_title: todo!(), version: todo!() },
            midi_blocks: Vec::new(),
            wave_blocks: Vec::new(),
        }
    }
}

pub fn parse(file:Vec<u8>) -> MmfFileInfo {
    let mut file_info:MmfFileInfo = MmfFileInfo::new();

    let mut stream = ByteStream::new_from_buffer(file);
    //If not found data in file bytes vector, Just return not found smaf header
    if stream.length <= 0 {
        file_info.result = MmfParseResult::NOT_FOUND_SMAF_HEADER;
    }

    file_info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let info = parse(Vec::new());
        assert_eq!(info.result, MmfParseResult::OK);
    }
}
