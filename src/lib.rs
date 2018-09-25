#![no_std]

extern crate byteorder;
use byteorder::ByteOrder;

#[derive(Debug)]
pub enum Error {
    BadHeader,
}

pub struct YmFile<'a> {
    data: &'a [u8],
    registers: &'a [u8],
}

pub struct RegisterSet<'a> {
    data: &'a [u8],
    external_frequency: u32,
}

const REGISTER_LEN: usize = 14 * 2;

impl<'a> YmFile<'a> {
    pub fn new(data: &'a [u8]) -> Result<YmFile<'a>, Error> {
        let (header, remainder) = data.split_at(4);
        if header != [b'Y', b'M', b'5', b'!'] {
            return Err(Error::BadHeader);
        }
        let (check, remainder) = remainder.split_at(8);
        if check != b"LeOnArD!" {
            return Err(Error::BadHeader);
        }

        // Skip over headers
        let (_, remainder) = remainder.split_at(22);

        let mut num_newlines = 0;
        let mut metadata_len = 0;
        for (idx, ch) in remainder.iter().enumerate() {
            if *ch == 0 {
                num_newlines = num_newlines + 1;
                if num_newlines == 3 {
                    // Start just after this last-newline
                    metadata_len = idx + 1;
                    break;
                }
            }
        }

        let registers = &remainder[metadata_len..];
        let len = registers.len() - 4;

        Ok(YmFile {
            data,
            registers: &registers[0..len],
        })
    }

    pub fn num_vbl(&self) -> u32 {
        byteorder::BigEndian::read_u32(&self.data[12..16])
    }

    pub fn song_attributes(&self) -> u32 {
        byteorder::BigEndian::read_u32(&self.data[16..20])
    }

    pub fn num_digi_drums(&self) -> u16 {
        byteorder::BigEndian::read_u16(&self.data[20..22])
    }

    pub fn external_frequency(&self) -> u32 {
        byteorder::BigEndian::read_u32(&self.data[22..26])
    }

    pub fn player_frequency(&self) -> u16 {
        byteorder::BigEndian::read_u16(&self.data[26..28])
    }

    pub fn vbl_loop_number(&self) -> u32 {
        byteorder::BigEndian::read_u32(&self.data[28..32])
    }

    pub fn num_registers(&self) -> usize {
        self.registers.len() / REGISTER_LEN
    }

    pub fn register(&self, idx: usize) -> RegisterSet<'a> {
        let offset = idx * REGISTER_LEN;
        RegisterSet {
            data: &self.registers[offset..(offset + REGISTER_LEN)],
            external_frequency: self.external_frequency(),
        }
    }
}

impl<'a> RegisterSet<'a> {
    pub fn raw(&self) -> &'a [u8] {
        self.data
    }

    pub fn tone_a(&self) -> u16 {
        (((self.data[1] & 0x0F) as u16) << 8) + (self.data[0] as u16)
    }

    pub fn tone_a_hz(&self) -> f32 {
        (self.external_frequency as f32) / (self.tone_a() as f32)
    }

    pub fn tone_b(&self) -> u16 {
        (((self.data[3] & 0x0F) as u16) << 8) + (self.data[2] as u16)
    }

    pub fn tone_b_hz(&self) -> f32 {
        (self.external_frequency as f32) / (self.tone_b() as f32)
    }

    pub fn tone_c(&self) -> u16 {
        (((self.data[5] & 0x0F) as u16) << 8) + (self.data[4] as u16)
    }

    pub fn tone_c_hz(&self) -> f32 {
        (self.external_frequency as f32) / (self.tone_c() as f32)
    }

    pub fn noise_period(&self) -> u8 {
        self.data[6]
    }

    pub fn noise_a_enabled(&self) -> bool {
        (self.data[7] & (1 << 3)) != 0
    }

    pub fn noise_b_enabled(&self) -> bool {
        (self.data[7] & (1 << 4)) != 0
    }

    pub fn noise_c_enabled(&self) -> bool {
        (self.data[7] & (1 << 4)) != 0
    }

    pub fn tone_a_enabled(&self) -> bool {
        (self.data[7] & (1 << 0)) != 0
    }

    pub fn tone_b_enabled(&self) -> bool {
        (self.data[7] & (1 << 1)) != 0
    }

    pub fn tone_c_enabled(&self) -> bool {
        (self.data[7] & (1 << 2)) != 0
    }

    pub fn channel_a_envelope(&self) -> bool {
        (self.data[8] & (1 << 5)) != 0
    }

    pub fn channel_a_volume(&self) -> u8 {
        self.data[8] & 0x1F
    }

    pub fn channel_b_envelope(&self) -> bool {
        (self.data[9] & (1 << 5)) != 0
    }

    pub fn channel_b_volume(&self) -> u8 {
        self.data[9] & 0x1F
    }

    pub fn channel_c_envelope(&self) -> bool {
        (self.data[10] & (1 << 5)) != 0
    }

    pub fn channel_c_volume(&self) -> u8 {
        self.data[10] & 0x1F
    }

    pub fn envelope_period(&self) -> u16 {
        ((self.data[11] as u16) << 8) + (self.data[12] as u16)
    }

    pub fn envelope_cont(&self) -> bool {
        (self.data[13] & (1 << 3)) != 0
    }

    pub fn envelope_att(&self) -> bool {
        (self.data[13] & (1 << 2)) != 0
    }

    pub fn envelope_alt(&self) -> bool {
        (self.data[13] & (1 << 1)) != 0
    }

    pub fn envelope_hold(&self) -> bool {
        (self.data[13] & (1 << 0)) != 0
    }
}

impl<'a> core::fmt::Debug for RegisterSet<'a> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(fmt, "RegisterSet(")?;
        if self.tone_a_enabled() {
            write!(fmt, "tone_a={} ({} Hz),", self.tone_a(), self.tone_a_hz())?;
        }
        if self.tone_b_enabled() {
            write!(fmt, "tone_b={} ({} Hz),", self.tone_b(), self.tone_b_hz())?;
        }
        if self.tone_c_enabled() {
            write!(fmt, "tone_c={} ({} Hz),", self.tone_c(), self.tone_c_hz())?;
        }
        write!(fmt, "noise_period={},", self.noise_period())?;
        write!(
            fmt,
            "noise_enabled={}{}{},",
            self.noise_a_enabled() as u8,
            self.noise_b_enabled() as u8,
            self.noise_c_enabled() as u8
        )?;
        write!(
            fmt,
            "chan_a={}/{},",
            self.channel_a_envelope(),
            self.channel_a_volume()
        )?;
        write!(
            fmt,
            "chan_b={}/{},",
            self.channel_b_envelope(),
            self.channel_b_volume()
        )?;
        write!(
            fmt,
            "chan_c={}/{},",
            self.channel_c_envelope(),
            self.channel_c_volume()
        )?;
        write!(fmt, "envelope_period={},", self.envelope_period())?;
        write!(
            fmt,
            "envelope_shape={}/{}/{}/{})",
            self.envelope_cont() as u8,
            self.envelope_att() as u8,
            self.envelope_alt() as u8,
            self.envelope_hold() as u8
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
