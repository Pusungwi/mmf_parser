use std::{str::Bytes, result};

struct FileHeader {
    data_size:usize,
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
pub enum MMFParseResult {
    OK,
    NOT_FOUND_SMAF_HEADER,
    UNKNOWN_ERROR,
}

pub struct MMFFileInfo {
    result:MMFParseResult,
    header:FileHeader,
    midi_blocks:Vec<MidiBlockHeader>,
    wave_blocks:Vec<WaveBlockHeader>,
}

impl MMFFileInfo {
    pub fn new() -> MMFFileInfo {
        MMFFileInfo {
            result: MMFParseResult::UNKNOWN_ERROR,
            header: FileHeader { data_size: 0 },
            midi_blocks: Vec::new(),
            wave_blocks: Vec::new(),
        }
    }
}

pub fn parse(file:Vec<Bytes>) -> MMFFileInfo {
    let mut file_info:MMFFileInfo = MMFFileInfo::new();
    
    //If no data in file bytes vector, Just return not found smaf header
    if file.len() <= 0 {
        file_info.result = MMFParseResult::NOT_FOUND_SMAF_HEADER;
    }

    
    
    file_info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let info = parse(Vec::new());
        assert_eq!(info.result, MMFParseResult::OK);
    }
}
