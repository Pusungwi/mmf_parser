use byteorder::{BigEndian, ReadBytesExt};
use std::io::{Cursor, Read, Seek};

#[derive(Default)]
pub struct ContentInfoBlock {
    pub signature: String,
    pub size: usize,
    pub class: u8,
    pub file_type: u8,
    pub code_type: u8,
    pub status: u8,
    pub counts: u8,
}

impl ContentInfoBlock {
    pub fn new() -> ContentInfoBlock {
        ContentInfoBlock::default()
    }
}

#[derive(Default)]
pub struct OptionalDataBlock {
    pub song_title: String,
    pub author: String,
    pub copyright: String,
}

impl OptionalDataBlock {
    pub fn new() -> OptionalDataBlock {
        OptionalDataBlock::default()
    }
}

#[derive(Default, Debug, PartialEq)]
pub enum TrackType {
    MIDI,
    PCM,
    #[default]
    Unknown,
}


#[derive(Default)]
pub struct TrackBlock {
    pub size: usize,
    pub track_no: u8,
    pub track_type: TrackType,
    pub data: Vec<u8>,
}

impl TrackBlock {
    pub fn new() -> TrackBlock {
        TrackBlock::default()
    }
}

#[derive(Debug, PartialEq)]
pub enum MmfParseResult {
    OK,
    NotFoundSmafHeader,
    UnknownError,
}

#[derive(Default)]
pub struct MmfFileInfo {
    pub data_size: usize,
    pub cnti_block: ContentInfoBlock,
    pub opda_block: OptionalDataBlock,
    pub midi_blocks: Vec<TrackBlock>,
    pub pcm_blocks: Vec<TrackBlock>,
}

impl MmfFileInfo {
    pub fn new() -> MmfFileInfo {
        MmfFileInfo::default()
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

fn find_signature_from_cursor(stream: &mut Cursor<Vec<u8>>, signature: &str) -> bool {
    for sig_byte in signature.as_bytes() {
        if stream.read_u8().unwrap() != *sig_byte {
            return false;
        }
    }
    true
}

pub fn parse(file: Vec<u8>) -> Result<MmfFileInfo, MmfParseResult> {
    let mut file_info: MmfFileInfo = MmfFileInfo::default();
    let mut file_stream = Cursor::new(file);

    //If not found data in file bytes vector, Just return not found smaf header
    if !find_signature_from_cursor(&mut file_stream, "MMMD") {
        return Err(MmfParseResult::NotFoundSmafHeader);
    }

    let smaf_size = file_stream.read_u32::<BigEndian>();
    match smaf_size {
        Ok(size) => {
            file_info.data_size = size as _;
        }
        Err(_err) => {
            return Err(MmfParseResult::UnknownError);
        }
    }
    
    //Read content info block info
    if find_signature_from_cursor(&mut file_stream, "CNTI") {
        file_info.cnti_block.signature = String::from("CNTI");
        
        let cnti_block_size = file_stream.read_u32::<BigEndian>();
        match cnti_block_size {
            Ok(size) => {
                file_info.cnti_block.size = size as _;
            }
            Err(_err) => {
            }
        }

        let cnti_block_class = file_stream.read_u8();
        match cnti_block_class {
            Ok(class) => {
                file_info.cnti_block.class = class as _;
            }
            Err(_err) => {
            }
        }

        let cnti_block_file_type = file_stream.read_u8();
        match cnti_block_file_type {
            Ok(class) => {
                file_info.cnti_block.file_type = class as _;
            }
            Err(_err) => {
            }
        }

        let cnti_block_code_type = file_stream.read_u8();
        match cnti_block_code_type {
            Ok(class) => {
                file_info.cnti_block.code_type = class as _;
            }
            Err(_err) => {
            }
        }

        let cnti_block_status = file_stream.read_u8();
        match cnti_block_status {
            Ok(class) => {
                file_info.cnti_block.status = class as _;
            }
            Err(_err) => {
            }
        }

        let cnti_block_counts = file_stream.read_u8();
        match cnti_block_counts {
            Ok(class) => {
                file_info.cnti_block.counts = class as _;
            }
            Err(_err) => {
            }
        }
    }

    //Read optional data block info (Not required block)
    if find_signature_from_cursor(&mut file_stream, "OPDA") {
        let opda_block_size_parse = file_stream.read_u32::<BigEndian>();
        match opda_block_size_parse {
            Ok(block_size) => {
                let mut block_data = vec![0; block_size as _];
                file_stream.read_exact(&mut block_data).unwrap();
                let mut opda_block_stream = Cursor::new(block_data);

                //signature "ST" is song title
                let song_title_result = read_opda_block_info(&mut opda_block_stream, b"ST");
                if let Some(song_title) = song_title_result {
                    file_info.opda_block.song_title = song_title;
                }
                let _opda_stream_rewind = file_stream.rewind();

                //signature "CA" is copyright author?
                let author_result = read_opda_block_info(&mut opda_block_stream, b"CA");
                if let Some(author_title) = author_result {
                    file_info.opda_block.author = author_title;
                }
                let _opda_stream_rewind = file_stream.rewind();
                
                //signature "CR" is copyright
                let copyright_result = read_opda_block_info(&mut opda_block_stream, b"CR");
                if let Some(copyright_title) = copyright_result {
                    file_info.opda_block.copyright = copyright_title;
                }
                let _opda_stream_rewind = file_stream.rewind();

                //TODO: signature "A0"
            }
            Err(_err) => {

            }
        }
    }

    //Find and read MIDI track
    loop {
        let midi_result = read_track_block(&mut file_stream, b"MTR");
        match midi_result {
            Some(mut block_data) => {
                // Use the new function to create a new MidiTrackBlock instance
                block_data.track_type = TrackType::MIDI;
                file_info.midi_blocks.push(block_data);
            }
            None => {
                break;
            }
        }
    }

    let midi_rewind_result = file_stream.rewind();
    if let Ok(()) = midi_rewind_result {
        loop {
            let wave_result = read_track_block(&mut file_stream, b"ATR");
            match wave_result {
                Some(mut block_data) => {
                    block_data.track_type = TrackType::PCM;
                    file_info.pcm_blocks.push(block_data);
                }
            None => {
                break;
            }
            }
        }
    }

    //Finally, All infos are set.
    Ok(file_info)
}

#[cfg(test)]
mod tests {
    use super::*;

    //https://www.reddit.com/r/rust/comments/dekpl5/how_to_read_binary_data_from_a_file_into_a_vecu8/
    fn get_file_as_byte_vec(filename: &String) -> Vec<u8> {
        match std::fs::read(filename) {
            Ok(bytes) => {
                bytes
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
        //4a1d512c7cf3845b664946749ff3e7162f92a768d91406e8a91fd0f3f37fa720  MachineWoman.mmf
        let info = parse(get_file_as_byte_vec(&String::from("MachineWoman.mmf")));
        match info {
            Ok(result) => {
                assert_eq!(result.data_size, 7408);
                assert_eq!(result.opda_block.song_title, "Machine Woman");
                assert_eq!(result.opda_block.author, "SMAF MA-3 Sample Data");
                assert_eq!(result.opda_block.copyright, "Copyright(c) 2002-2004 YAMAHA CORPORATION");
                assert_eq!(result.midi_blocks.len(), 1);
                assert_eq!(result.midi_blocks[0].track_type, TrackType::MIDI);
                assert_eq!(result.midi_blocks[0].size, 7242);
            }
            Err(e) => {
                assert_eq!(e, MmfParseResult::OK);
            }
        }
    }
}
