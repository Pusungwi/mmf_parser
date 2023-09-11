use std::io::{Cursor, Read, Seek};
use byteorder::{ReadBytesExt, BigEndian};

struct ContentInfoBlock {
    signature:String,
    size:usize,
    class:u8,
    file_type:u8,
    code_type:u8,
    status:u8,
    counts:u8,
}

impl ContentInfoBlock {
    pub fn new() -> ContentInfoBlock {
        ContentInfoBlock {
            signature: String::new(),
            size: 0,
            class: 0,
            file_type: 0,
            code_type: 0,
            status: 0,
            counts: 0,
        }
    }
}

struct OptionalDataBlock {
    song_title:String,
}

impl OptionalDataBlock {
    pub fn new() -> OptionalDataBlock {
        OptionalDataBlock {
            song_title: String::new(),
        }
    }
}

struct TrackBlock {
    size:usize,
    track_no:u8,
    data:Vec<u8>,
}

impl TrackBlock {
    pub fn new() -> TrackBlock {
        TrackBlock {
            size: 0,
            track_no: 0,
            data: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum MmfParseResult {
    OK,
    NotFoundSmafHeader,
    UnknownError,
}

pub struct MmfFileInfo {
    result:MmfParseResult,
    data_size:usize,
    cnti_block:ContentInfoBlock,
    opda_block:OptionalDataBlock,
    midi_blocks:Vec<TrackBlock>,
    wave_blocks:Vec<TrackBlock>,
}

impl MmfFileInfo {
    pub fn new() -> MmfFileInfo {
        MmfFileInfo {
            result: MmfParseResult::UnknownError,
            data_size: 0,
            cnti_block: ContentInfoBlock::new(),
            opda_block: OptionalDataBlock::new(),
            midi_blocks: Vec::new(),
            wave_blocks: Vec::new(),
        }
    }
}

fn read_opda_block_info(cursor: &mut Cursor<Vec<u8>>, signature: &[u8]) -> Option<String> {
    let mut buffer = Vec::new();
    loop {
        let mut byte_buffer = [0; 1];
        match cursor.read(&mut byte_buffer) {
            Ok(0) => break, // end of stream
            Ok(_) => {
                buffer.push(byte_buffer[0]);
                if buffer.ends_with(signature) {
                    let info_length = cursor.read_u16::<BigEndian>().unwrap() as usize;                    
                    let mut exact_data = vec![0; info_length];
                    let _ = cursor.read_exact(&mut exact_data);
                    
                    let read_result = String::from_utf8(exact_data);
                    match read_result {
                        Ok(result) => {
                            return Some(result);
                        }
                        Err(_err) => break,
                    }
                }
            }
            Err(_) => break, // error
        }
    }
    None
}

fn read_track_block(cursor: &mut Cursor<Vec<u8>>, signature: &[u8]) -> Option<TrackBlock> {
    let mut buffer = Vec::new();
    loop {
        let mut byte_buffer = [0; 1];
        match cursor.read(&mut byte_buffer) {
            Ok(0) => break, // end of stream
            Ok(_) => {
                buffer.push(byte_buffer[0]);
                if buffer.ends_with(signature) {
                    let mut new_block = TrackBlock::new();
                    new_block.track_no = cursor.read_u8().unwrap();
                    new_block.size = cursor.read_u32::<BigEndian>().unwrap() as _;
                    
                    let mut exact_data = vec![0; new_block.size];
                    let _ = cursor.read_exact(&mut exact_data);
                    new_block.data.extend_from_slice(&exact_data);
                    
                    return Some(new_block);
                }
            }
            Err(_) => break, // error
        }
    }
    None
}

fn find_signature_from_cursor(stream:&mut Cursor<Vec<u8>>, signature: &str) -> bool
{
    for sig_byte in signature.as_bytes() {
        if !stream.read_u8().unwrap() == *sig_byte {
            return false;
        }
    }

    return true;
}

pub fn parse(file:Vec<u8>) -> MmfFileInfo {
    let mut file_info:MmfFileInfo = MmfFileInfo::new();

    let mut stream = Cursor::new(file);

    //If not found data in file bytes vector, Just return not found smaf header
    if !find_signature_from_cursor(&mut stream, "MMMD") {
        file_info.result = MmfParseResult::NotFoundSmafHeader;
        return file_info;
    }

    let smaf_size = stream.read_u32::<BigEndian>();
    match smaf_size {
        Ok(size) => {
            file_info.data_size = size as _;
        }
        Err(_err) => {
            file_info.result = MmfParseResult::UnknownError;
            return file_info;
        }
    }
    
    //Read content info block info
    if find_signature_from_cursor(&mut stream, "CNTI") {
        file_info.cnti_block.signature = String::from("CNTI");
        
        let cnti_block_size = stream.read_u32::<BigEndian>();
        match cnti_block_size {
            Ok(size) => {
                file_info.cnti_block.size = size as _;
            }
            Err(_err) => {
            }
        }

        let cnti_block_class = stream.read_u8();
        match cnti_block_class {
            Ok(class) => {
                file_info.cnti_block.class = class as _;
            }
            Err(_err) => {
            }
        }

        let cnti_block_file_type = stream.read_u8();
        match cnti_block_file_type {
            Ok(class) => {
                file_info.cnti_block.file_type = class as _;
            }
            Err(_err) => {
            }
        }

        let cnti_block_code_type = stream.read_u8();
        match cnti_block_code_type {
            Ok(class) => {
                file_info.cnti_block.code_type = class as _;
            }
            Err(_err) => {
            }
        }

        let cnti_block_status = stream.read_u8();
        match cnti_block_status {
            Ok(class) => {
                file_info.cnti_block.status = class as _;
            }
            Err(_err) => {
            }
        }

        let cnti_block_counts = stream.read_u8();
        match cnti_block_counts {
            Ok(class) => {
                file_info.cnti_block.counts = class as _;
            }
            Err(_err) => {
            }
        }
    }

    //Read optional data block info (Not required block)
    if find_signature_from_cursor(&mut stream, "OPDA") {
        //TODO: signature "ST" and read int is song title length
        let song_title_result = read_opda_block_info(&mut stream, b"ST");
        match song_title_result {
            Some(song_title) => {
                file_info.opda_block.song_title = song_title;
            }
            None => {

            }
        }
        
        //TODO: signature "CA" and read int is copyright author? length

        //TODO: signature "CR" and read int is copyright length

        //TODO: signature "A0"

        //and rewind
        //TODO: un-optimized find data style
        let _opda_rewind_result = stream.rewind();
    }

    //Find and read MIDI track
    loop {
        let midi_result = read_track_block(&mut stream, b"MTR");
        match midi_result {
            Some(block_data) => {
                // Use the new function to create a new MidiTrackBlock instance
                file_info.midi_blocks.push(block_data);
            }
            None => {
                break;
            }
        }
    }

    let midi_rewind_result = stream.rewind();
    match midi_rewind_result {
        Ok(()) => {
            loop {
                let wave_result = read_track_block(&mut stream, b"ATR");
                match wave_result {
                    Some(block_data) => {
                        file_info.wave_blocks.push(block_data);
                    }
                    None => {
                        break;
                    }
                }
            }
        }
        Err(..) => {
            
        }
    }

    //Finally, All infos are set.
    file_info.result = MmfParseResult::OK;
    file_info
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_mmf_parsing() {
        //cd456ee3c9dfb4c7e5f9b2b1c875fd88b0506050e10c359621cbc382df7ab71a  mmf_parser_test.mmf
        let info = parse(get_file_as_byte_vec(&String::from("mmf_parser_test.mmf")));
        assert_eq!(info.result, MmfParseResult::OK);
        assert_eq!(info.data_size, 1625);
        assert_eq!(info.midi_blocks.len(), 4);
        assert_eq!(info.midi_blocks[0].size, 636);
        assert_eq!(info.midi_blocks[1].size, 443);
        assert_eq!(info.midi_blocks[2].size, 328);
        assert_eq!(info.midi_blocks[3].size, 146);
    }
}
