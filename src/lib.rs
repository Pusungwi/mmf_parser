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
            header: FileHeader { data_size: 0, class: 0, file_type: 0, code_type: 0, status: 0, counts: 0, song_title:"".to_string(), version: 0 },
            midi_blocks: Vec::new(),
            wave_blocks: Vec::new(),
        }
    }
}

pub fn parse(file:Vec<u8>) -> MmfFileInfo {
    let mut file_info:MmfFileInfo = MmfFileInfo::new();

    let mut stream = ByteStream::new_from_buffer(file);
    //If not found data in file bytes vector, Just return not found smaf header
    if stream.length <= 0 || !stream.read_string_size(4).unwrap().eq("MMMD") {
        file_info.result = MmfParseResult::NOT_FOUND_SMAF_HEADER;
        return file_info;
    }

    let smaf_size = stream.read_uint32();
    match smaf_size {
        Ok(size) => {
            file_info.header.data_size = size as _;
        }
        Err(_err) => {
            file_info.result = MmfParseResult::UNKNOWN_ERROR;
            return file_info;
        }
    }

    //Finally, All infos are set.
    file_info
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;

    //https://www.reddit.com/r/rust/comments/dekpl5/how_to_read_binary_data_from_a_file_into_a_vecu8/
    fn get_file_as_byte_vec(filename: &String) -> Vec<u8> {
        match std::fs::read(filename) {
            Ok(bytes) => {
                return bytes;
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    eprintln!("please run again with appropriate permissions.");
                }
                panic!("{}", e);
            }
        }
    }

    #[test]
    fn it_works() {
        let info = parse(get_file_as_byte_vec(&String::from("test.mmf")));
        assert_eq!(info.result, MmfParseResult::OK);
    }
}
