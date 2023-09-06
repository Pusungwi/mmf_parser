use std::io::{self, Cursor, Read, Seek};
use byteorder::{ReadBytesExt, BigEndian};

struct ContentInfoBlock {
    signature:String,
    size:usize,
    class:u8,
    file_type:u8,
    code_type:u8,
    status:u8,
    counts:u8,
    song_title:String,
    version:u8,
}

impl Default for ContentInfoBlock {
    fn default() -> Self {
        ContentInfoBlock {
            signature: String::new(),
            size: 0,
            class: 0,
            file_type: 0,
            code_type: 0,
            status: 0,
            counts: 0,
            song_title: String::new(),
            version: 0,
        }
    }
}

struct MidiTrackBlock {
    size:usize,
    track_no:u8,
    data:Vec<u8>,
}

impl Default for MidiTrackBlock {
    fn default() -> Self {
        MidiTrackBlock {
            size: 0,
            track_no: 0,
            data: Vec::new(),
        }
    }
}

struct WaveTrackBlock {
    size:usize,
    track_no:u8,
    data:Vec<u8>,
}

impl Default for WaveTrackBlock {
    fn default() -> Self {
        WaveTrackBlock {
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
    midi_blocks:Vec<MidiTrackBlock>,
    wave_blocks:Vec<WaveTrackBlock>,
}

impl MmfFileInfo {
    pub fn new() -> MmfFileInfo {
        MmfFileInfo {
            result: MmfParseResult::UnknownError,
            data_size: 0,
            cnti_block: ContentInfoBlock::default(),
            midi_blocks: Vec::new(),
            wave_blocks: Vec::new(),
        }
    }
}

fn find_block_with_tag<R: Read>(stream: &mut R, tag: &[u8]) -> io::Result<Vec<u8>> {
    let mut buffer = [0; 1024];
    let mut block = Vec::new();

    loop {
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        block.extend_from_slice(&buffer[..bytes_read]);

        if let Some(index) = block.windows(tag.len()).position(|window| window == tag) {
            block.truncate(index + tag.len());
            return Ok(block);
        }
    }

    Err(io::Error::new(io::ErrorKind::NotFound, "Block not found"))
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

    //TODO: Find and read MIDI track
    loop {
        let mut midi_result = find_block_with_tag(&mut stream, b"MTR");
        match midi_result {
            Ok(block_data) => {
                // Use the new function to create a new MidiTrackBlock instance
                let mut new_midi_block = MidiTrackBlock::default();
                new_midi_block.track_no = block_data[0];
                new_midi_block.size = block_data.len();
                new_midi_block.data = block_data;

                file_info.midi_blocks.push(new_midi_block);
            }
            Err(_err) => {
                break;
            }
        }
    }

    let mut rewind_result = stream.rewind();
    match rewind_result {
        Ok(()) => {
            loop {
                let mut wave_result = find_block_with_tag(&mut stream, b"ATR");
                match wave_result {
                    Ok(block_data) => {
                        // Use the new function to create a new MidiTrackBlock instance
                        let mut new_wave_block = WaveTrackBlock::default();
                        new_wave_block.track_no = 1;
                        new_wave_block.size = block_data.len();
                        new_wave_block.data = block_data;

                        file_info.wave_blocks.push(new_wave_block);
                    }
                    Err(_err) => {
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
    fn it_works() {
        let info = parse(get_file_as_byte_vec(&String::from("mmf_parser_test.mmf")));
        assert_eq!(info.result, MmfParseResult::OK);
        assert_eq!(info.data_size, 1625);
    }
}
