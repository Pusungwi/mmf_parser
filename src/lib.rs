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

pub enum MMFParseResult {
    OK,
    NOT_FOUND_SAMF_HEADER,
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

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub fn parse(file:Vec<Bytes>) -> MMFFileInfo {
    let mut file_info:MMFFileInfo = MMFFileInfo::new();
    file_info.result = MMFParseResult::UNKNOWN_ERROR;
    file_info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 0);
    }
}
