use byteorder::{BigEndian, ReadBytesExt};
use std::{io::{Cursor, Read, Seek}};
use bitflags::bitflags;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SampleRate {
    #[default]
    Hz4000,
    Hz8000,
    Hz11025,
    Hz22050,
    Hz44100,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BitDepth {
    #[default]
    Adpcm4Bit,
    Pcm8Bit,
    Adpcm12Bit,
    Pcm16Bit,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Channels {
    #[default]
    Mono,
    Stereo,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum StreamType {
    #[default]
    Normal,
    Ringtone,
    Effect,
    Background,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Default)]
    pub struct PcmStreamParam: u32 {
        const SPEAKER  = 0x01;
        const EARPHONE = 0x02;
        const VIBRATOR = 0x04;
        const LED      = 0x08;
    }
}

#[derive(Debug, Default)]
pub struct PcmMetaData {
    pub format_type: u8,
    pub sequence_type: u8,
    pub wave_type_raw: u8,
    pub stream_type: StreamType,
    pub stream_param: PcmStreamParam,
    pub sample_rate: SampleRate,
    pub bit_depth: BitDepth,
    pub channels: Channels,
}

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

#[derive(Default)]
pub struct MidiTrackBlock {
    pub size: usize,
    pub track_no: u8,
    pub data: Vec<u8>,
}

#[derive(Default)]
pub struct PcmTrackBlock {
    pub size: usize,
    pub track_no: u8,
    pub metadata: PcmMetaData,
    pub wave_data: Vec<u8>,
    pub raw_data: Vec<u8>, // ATR Block raw data (for debugging)
}

#[derive(Default)]
pub struct RawTrackBlock {
    pub size: usize,
    pub track_no: u8,
    pub data: Vec<u8>,
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
    pub midi_blocks: Vec<MidiTrackBlock>,
    pub pcm_blocks: Vec<PcmTrackBlock>,
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
            Err(_) => break,
        }
    }
    None
}

fn read_track_block(cursor: &mut Cursor<Vec<u8>>, signature: &[u8]) -> Option<RawTrackBlock>
{
    let mut buffer = Vec::new();
    loop {
        let mut byte_buffer = [0; 1];
        match cursor.read(&mut byte_buffer) {
            Ok(0) => break,
            Ok(_) => {
                buffer.push(byte_buffer[0]);
                if buffer.ends_with(signature) {
                    let track_no = cursor.read_u8().unwrap();
                    let size = cursor.read_u32::<BigEndian>().unwrap() as usize;
                    let mut data = vec![0; size];
                    let _ = cursor.read_exact(&mut data);
                    return Some(RawTrackBlock { size, track_no, data });
                }
            }
            Err(_) => break,
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

fn parse_wave_type(raw: u8) -> (SampleRate, BitDepth, Channels) {
    //   bit:  7  6  5  4 | 3  2 | 1 | 0
    //         ~~~~~~~~~~~  ~~~~~  ~~   ~~
    //         sample_rate  depth  ch  reserved
    //
    // ex) raw = 0b_0001_0110 = 0x16
    //   sample_rate = 0b_0001 = 1 → Hz8000
    //   bit_depth   = 0b_01   = 1 → Pcm8Bit
    //   channels    = 0b_1    = 1 → Stereo

    let sample_rate = match (raw >> 4) & 0x0F {
        0 => SampleRate::Hz4000,
        1 => SampleRate::Hz8000,
        2 => SampleRate::Hz11025,
        3 => SampleRate::Hz22050,
        4 => SampleRate::Hz44100,
        // fallback
        _ => SampleRate::Hz8000,
    };

    let bit_depth = match (raw >> 2) & 0x03 {
        0 => BitDepth::Adpcm4Bit,
        1 => BitDepth::Pcm8Bit,
        2 => BitDepth::Adpcm12Bit,
        3 => BitDepth::Pcm16Bit,
         // Available only 0~3 because of 2 bit mask
        _ => unreachable!(),
    };

    let channels = if raw & 0x02 != 0 {
        Channels::Stereo
    } else {
        Channels::Mono
    };

    (sample_rate, bit_depth, channels)
}

fn parse_atr_data(block: RawTrackBlock) -> PcmTrackBlock {
    // offset 0: Format Type  (1 byte, Decoded to parse_wave_type)
    // offset 1: Sequence Type (1 byte, Decoded to parse_wave_type)
    // offset 2: Wave Type     (1 byte, Decoded to parse_wave_type)
    // offset 3~5: Additional field of ATR header (TODO: find more usage) 
    // offset 6~: Repeating many subchunks...
    let data = &block.data;

    // 1) Parsing offset 0,1,2
    let format_type = data[0];
    let sequence_type = data[1];
    let wave_type_raw = data[2];
    let (sample_rate, bit_depth, channels) = parse_wave_type(wave_type_raw);

    //   [sig][size][payload]
    //   ex) 41 77 61 01 | 00 00 10 00 | ...4096bytes...
    //       "A""w""a"\x01  size=4096    Some kind of data
    let mut stream_type: StreamType = StreamType::Normal;
    let mut stream_param: PcmStreamParam = PcmStreamParam::SPEAKER;
    let mut wave_data: Vec<u8> = Vec::new();
    let mut pos: usize = 6;
    // pos + 8 = minimum size of sub chunk
    while pos + 8 <= data.len() {
        // 3 byte is enough i think.
        let sig = &data[pos..pos + 3];
        let chunk_size = u32::from_be_bytes([
            data[pos + 4], data[pos + 5],
            data[pos + 6], data[pos + 7],
        ]) as usize;

        if sig == b"Asp" {
            // Audio Setup Parameter Info (sig:AspI / Offset 6~29)
            // 73 74 3a 00 00 00 00 2c 73 70 3a 00 00 00 0c 2c
            // s  t  :  -----------  ,  s  p  : -----------  ,
            let st = u32::from_be_bytes([
                data[pos + 11], data[pos + 12],
                data[pos + 13], data[pos + 14],
            ]);
            stream_type = match st {
                0 => StreamType::Normal,
                1 => StreamType::Ringtone,
                2 => StreamType::Effect,
                3 => StreamType::Background,
                // fallback
                _ => StreamType::Normal,
            };

            let sp = u32::from_be_bytes([
                data[pos + 19], data[pos + 20],
                data[pos + 21], data[pos + 22],
            ]);
            stream_param = PcmStreamParam::from_bits_truncate(sp);
        }
        else if sig == b"Ats" {
            // Audio Track Sequence (sig:Atsq / offset 30~53)
            //TODO: Parse Audio Track Sequence
        }
        else if sig == b"Awa" {
            // Wave Audio Data (sig:Awa\x01 / offset 54~)
            let payload_start = pos + 8;
            let payload_end = (payload_start + chunk_size).min(data.len());
            wave_data = data[payload_start..payload_end].to_vec();
        }

        pos += 8 + chunk_size;
    }

    // 3) 결과 조립
    PcmTrackBlock {
        size: block.size,
        track_no: block.track_no,
        metadata: PcmMetaData {
            format_type,
            sequence_type,
            wave_type_raw,
            stream_type,
            stream_param,
            sample_rate,
            bit_depth,
            channels,
        },
        wave_data,
        raw_data: block.data,
    }
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
            Some(block) => {
                // RawTrackBlock → MidiTrackBlock
                file_info.midi_blocks.push(MidiTrackBlock {
                    size: block.size,
                    track_no: block.track_no,
                    data: block.data,
                });
            }
            None => break,
        }
    }

    let midi_rewind_result = file_stream.rewind();
    if let Ok(()) = midi_rewind_result {
        loop {
            let wave_result = read_track_block(&mut file_stream, b"ATR");
            match wave_result {
                Some(block) => {
                    // RawTrackBlock → PcmTrackBlock
                    let pcm_block = parse_atr_data(block);
                    file_info.pcm_blocks.push(pcm_block);
                }
                None => break,
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

    const STEP_TABLE_SIMPLE: [i32; 49] = [
        16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60,
        66, 73, 80, 88, 97, 107, 118, 130, 143, 157, 173, 190, 209,
        230, 253, 279, 307, 337, 371, 408, 449, 494, 544, 598, 658,
        724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552,
    ];

    const INDEX_TABLE: [i32; 8] = [-1, -1, -1, -1, 2, 4, 6, 8];

    fn decode_adpcm_4bit(data: &[u8]) -> Vec<i16> {
        let mut predicted: i32 = 0;
        let mut step_index: i32 = 0;
        let mut output = Vec::with_capacity(data.len() * 2);

        for &byte in data {
            for nibble in [byte >> 4, byte & 0x0F] {
                let code = nibble as i32;
                let step = STEP_TABLE_SIMPLE[step_index as usize];

                let mut diff = step >> 3;
                if code & 4 != 0 { diff += step; }
                if code & 2 != 0 { diff += step >> 1; }
                if code & 1 != 0 { diff += step >> 2; }

                if code & 8 != 0 {
                    predicted -= diff;
                } else {
                    predicted += diff;
                }
                predicted = predicted.clamp(-32768, 32767);

                output.push(predicted as i16);

                step_index += INDEX_TABLE[(code & 7) as usize];
                step_index = step_index.clamp(0, 48);
            }
        }
        output
    }

    #[test]
    fn test_mmf_parsing_opda() {
        // sha256sum : 4a1d512c7cf3845b664946749ff3e7162f92a768d91406e8a91fd0f3f37fa720
        let info = parse(get_file_as_byte_vec(&String::from("MachineWoman.mmf")));
        match info {
            Ok(result) => {
                assert_eq!(result.data_size, 7408);
                assert_eq!(result.opda_block.song_title, "Machine Woman");
                assert_eq!(result.opda_block.author, "SMAF MA-3 Sample Data");
                assert_eq!(result.opda_block.copyright, "Copyright(c) 2002-2004 YAMAHA CORPORATION");
                assert_eq!(result.midi_blocks.len(), 1);
                assert_eq!(result.midi_blocks[0].size, 7242);
                assert_eq!(result.pcm_blocks.len(), 0);
            }
            Err(e) => {
                assert_eq!(e, MmfParseResult::OK);
            }
        }
    }

    #[test]
    fn test_mmf_parsing_pcm() {
        // sha256sum : 56fd041545b16f7df41cabcb1b0c7fca27d789e6c32d97a8e367519aea3380c5
        let info = parse(get_file_as_byte_vec(&String::from("TestPCM.mmf")));
        match info {
            Ok(result) => {
                assert_eq!(result.data_size, 4990);
                assert_eq!(result.midi_blocks.len(), 0);
                assert_eq!(result.pcm_blocks.len(), 1);
                assert_eq!(result.pcm_blocks[0].size, 4903);
                assert_eq!(result.pcm_blocks[0].metadata.stream_type, StreamType::Normal);
                assert!(result.pcm_blocks[0].metadata.stream_param.contains(PcmStreamParam::LED));
                assert!(result.pcm_blocks[0].metadata.stream_param.contains(PcmStreamParam::VIBRATOR));
                assert_eq!(result.pcm_blocks[0].metadata.sample_rate, SampleRate::Hz8000);
                assert_eq!(result.pcm_blocks[0].metadata.bit_depth, BitDepth::Adpcm4Bit);
                assert_eq!(result.pcm_blocks[0].metadata.channels, Channels::Mono);

                //dbg!(&result.pcm_blocks[0].metadata);
            }
            Err(e) => {
                assert_eq!(e, MmfParseResult::OK);
            }
        }
    }

    #[test]
    fn test_mmf_pcm_to_wav() {
        let info = parse(get_file_as_byte_vec(&String::from("TestPCM.mmf"))).unwrap();
        let pcm = &info.pcm_blocks[0];
        let meta = &pcm.metadata;

        let sample_rate = match meta.sample_rate {
            SampleRate::Hz4000 => 4000,
            SampleRate::Hz8000 => 8000,
            SampleRate::Hz11025 => 11025,
            SampleRate::Hz22050 => 22050,
            SampleRate::Hz44100 => 44100,
        };
        let channels = match meta.channels {
            Channels::Mono => 1,
            Channels::Stereo => 2,
        };

        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create("test_output.wav", spec).unwrap();

        match meta.bit_depth {
            BitDepth::Adpcm4Bit => {
                let samples = decode_adpcm_4bit(&pcm.wave_data);
                for s in samples {
                    writer.write_sample(s).unwrap();
                }
            }
            BitDepth::Pcm8Bit => {
                // 8bit unsigned → 16bit signed
                for &b in &pcm.wave_data {
                    let s = ((b as i16) - 128) * 256;
                    writer.write_sample(s).unwrap();
                }
            }
            BitDepth::Pcm16Bit => {
                // Big-Endian 16bit → 16bit signed
                for chunk in pcm.wave_data.chunks_exact(2) {
                    let s = i16::from_be_bytes([chunk[0], chunk[1]]);
                    writer.write_sample(s).unwrap();
                }
            }
            BitDepth::Adpcm12Bit => {
                todo!("12-bit ADPCM");
            }
        }
        writer.finalize().unwrap();
    }
}
