use std::cmp::Ordering;
use std::io::{Error, ErrorKind, Result};
use std::num::Wrapping;

pub struct Rom {
    content: Vec<u8>,
}

#[derive(Debug, PartialEq)]
enum MemoryBankType {
    ROM,
    MBC1,
    MBC2,
    MMM01,
    MBC3,
    MBC5,
    MBC6,
    MBC7,
}

impl Rom {
    pub fn new(path: &str) -> Result<Rom> {
        let rom = Rom {
            content: std::fs::read(path)?,
        };

        rom.verify_nintendo_logo()?;
        rom.verify_memory_bank_matches_ram()?;

        Ok(rom)
    }

    fn verify_nintendo_logo(&self) -> Result<()> {
        if self.content[NINTENDO_LOGO_RANGE]
            .iter()
            .cmp(NINTENDO_LOGO.iter())
            != Ordering::Equal
        {
            return Err(Error::new(
                ErrorKind::Other,
                "Nintendo logo not found in header.",
            ));
        }

        Ok(())
    }

    fn verify_memory_bank_matches_ram(&self) -> Result<()> {
        let ram_size = self.get_ram_size()?;
        if self.get_memory_bank_type()? == MemoryBankType::MBC2 && ram_size != 0 {
            Err(Error::new(
                ErrorKind::Other,
                format!(
                    "When the memory bank type is MBC2, the ram must be 0 but was {}",
                    ram_size
                ),
            ))
        } else {
            Ok(())
        }
    }

    fn get_memory_bank_type(&self) -> Result<MemoryBankType> {
        let memory_bank_type = match self.content[CARTRIDGE_TYPE_INDEX] {
            0x00 | 0x08 | 0x09 => MemoryBankType::ROM,
            0x01..=0x03 => MemoryBankType::MBC1,
            0x05 | 0x06 => MemoryBankType::MBC2,
            0x0B..=0x0D => MemoryBankType::MMM01,
            0x0F..=0x13 => MemoryBankType::MBC3,
            0x19..=0x1E => MemoryBankType::MBC5,
            0x20 => MemoryBankType::MBC6,
            0x22 => MemoryBankType::MBC7,
            _ => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!(
                        "{} is an invalid value for the cartridge type.",
                        self.content[CARTRIDGE_TYPE_INDEX]
                    ),
                ))
            }
        };

        Ok(memory_bank_type)
    }

    fn get_rom_size(&self) -> Result<u32> {
        let byte = self.content[ROM_SIZE_INDEX];
        if byte > 0x08 {
            return Err(Error::new(
                ErrorKind::Other,
                format!("{} is an invalid value for the ROM size.", byte),
            ));
        }

        // This ranges from 32 KB to 8 MB.
        return Ok(32768 << byte);
    }

    fn get_ram_size(&self) -> Result<u32> {
        let ram_size = match self.content[RAM_SIZE_INDEX] {
            0x00 | 0x01 => 0, // 0x01 is not officially documented. Only used in homebrew ROMs and the expect no RAM so having it set to 0.
            0x02 => 8 * KB,
            0x03 => 32 * KB,
            0x04 => 128 * KB,
            0x05 => 64 * KB,
            _ => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!(
                        "{} is an invalid value for the RAM size.",
                        self.content[RAM_SIZE_INDEX]
                    ),
                ));
            }
        };

        Ok(ram_size)
    }

    fn verify_header_checksum(&self) -> Result<()> {
        let checksum = self.content[HEADER_CHECKSUM_RANGE]
            .iter()
            .cloned()
            .fold(Wrapping(0), |acc, v| acc - Wrapping(v) - Wrapping(1));

        if checksum.0 != self.content[HEADER_CHECKSUM_INDEX] {
            return Err(Error::new(
                ErrorKind::Other,
                format!(
                    "The expected checksum, {}, did not match the actual checksum, {}.",
                    self.content[HEADER_CHECKSUM_INDEX], checksum
                ),
            ));
        }

        Ok(())
    }
}

const NINTENDO_LOGO: [u8; 48] = [
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
    0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
    0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];

const NINTENDO_LOGO_RANGE: std::ops::Range<usize> = 0x104..0x134;

const CARTRIDGE_TYPE_INDEX: usize = 0x147;

const ROM_SIZE_INDEX: usize = 0x148;

const RAM_SIZE_INDEX: usize = 0x149;

const KB: u32 = 1024;

const HEADER_CHECKSUM_INDEX: usize = 0x14D;

const HEADER_CHECKSUM_RANGE: std::ops::RangeInclusive<usize> = 0x134..=0x14C;

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn test_verify_nintendo_logo() {
        // Arrange
        let mut content: Vec<u8> = vec![0; NINTENDO_LOGO_RANGE.end];
        for byte in NINTENDO_LOGO_RANGE {
            content[byte] = NINTENDO_LOGO[byte - NINTENDO_LOGO_RANGE.start];
        }

        let rom = Rom { content };

        // Act + Assert
        assert!(rom.verify_nintendo_logo().is_ok());
    }

    #[test]
    fn test_verify_nintendo_logo_negative() {
        // Arrange
        let content: Vec<u8> = vec![0; NINTENDO_LOGO_RANGE.end];

        let rom = Rom { content };

        // Act + Assert
        assert!(rom.verify_nintendo_logo().is_err());
    }

    #[rstest]
    #[case(0x00, MemoryBankType::ROM)]
    #[case(0x01, MemoryBankType::MBC1)]
    #[case(0x02, MemoryBankType::MBC1)]
    #[case(0x03, MemoryBankType::MBC1)]
    #[case(0x05, MemoryBankType::MBC2)]
    #[case(0x06, MemoryBankType::MBC2)]
    #[case(0x08, MemoryBankType::ROM)]
    #[case(0x09, MemoryBankType::ROM)]
    #[case(0x0B, MemoryBankType::MMM01)]
    #[case(0x0C, MemoryBankType::MMM01)]
    #[case(0x0D, MemoryBankType::MMM01)]
    #[case(0x0F, MemoryBankType::MBC3)]
    #[case(0x10, MemoryBankType::MBC3)]
    #[case(0x11, MemoryBankType::MBC3)]
    #[case(0x12, MemoryBankType::MBC3)]
    #[case(0x13, MemoryBankType::MBC3)]
    #[case(0x19, MemoryBankType::MBC5)]
    #[case(0x1A, MemoryBankType::MBC5)]
    #[case(0x1B, MemoryBankType::MBC5)]
    #[case(0x1C, MemoryBankType::MBC5)]
    #[case(0x1D, MemoryBankType::MBC5)]
    #[case(0x1E, MemoryBankType::MBC5)]
    #[case(0x20, MemoryBankType::MBC6)]
    #[case(0x22, MemoryBankType::MBC7)]
    fn test_get_memory_bank_type(#[case] byte: u8, #[case] memory_bank_type: MemoryBankType) {
        // Arrange
        let mut content: Vec<u8> = vec![0; CARTRIDGE_TYPE_INDEX + 1];
        content[CARTRIDGE_TYPE_INDEX] = byte;

        let rom = Rom { content };

        // Act
        let actual_memory_bank_type = rom.get_memory_bank_type().unwrap();

        // Assert
        assert_eq!(actual_memory_bank_type, memory_bank_type);
    }

    #[test]
    fn test_get_memory_bank_type_negative() {
        // Arrange
        let mut content: Vec<u8> = vec![0; CARTRIDGE_TYPE_INDEX + 1];
        content[CARTRIDGE_TYPE_INDEX] = 0x23;

        let rom = Rom { content };

        // Act + Assert
        assert!(rom.get_memory_bank_type().is_err());
    }

    #[rstest]
    #[case(0x00, 32768)]
    #[case(0x01, 65536)]
    #[case(0x02, 131072)]
    #[case(0x03, 262144)]
    #[case(0x04, 524288)]
    #[case(0x05, 1048576)]
    #[case(0x06, 2097152)]
    #[case(0x07, 4194304)]
    #[case(0x08, 8388608)]
    fn test_get_rom_size(#[case] byte: u8, #[case] expected_size: u32) {
        // Arrange
        let mut content: Vec<u8> = vec![0; ROM_SIZE_INDEX + 1];
        content[ROM_SIZE_INDEX] = byte;
        let rom = Rom { content };

        // Act + Assert
        assert_eq!(expected_size, rom.get_rom_size().unwrap());
    }

    #[test]
    fn test_get_rom_size_negative() {
        let mut content: Vec<u8> = vec![0; ROM_SIZE_INDEX + 1];
        content[ROM_SIZE_INDEX] = 0x09;
        let rom = Rom { content };
        assert!(rom.get_rom_size().is_err());
    }

    #[rstest]
    #[case(0x00, 0)]
    #[case(0x01, 0)]
    #[case(0x02, 8192)]
    #[case(0x03, 32768)]
    #[case(0x04, 131072)]
    #[case(0x05, 65536)]
    fn test_get_ram_size(#[case] byte: u8, #[case] expected_size: u32) {
        // Arrange
        let mut content: Vec<u8> = vec![0; RAM_SIZE_INDEX + 1];
        content[RAM_SIZE_INDEX] = byte;
        let rom = Rom { content };

        // Act + Assert
        assert_eq!(expected_size, rom.get_ram_size().unwrap());
    }

    #[test]
    fn test_get_ram_size_negative() {
        let mut content: Vec<u8> = vec![0; RAM_SIZE_INDEX + 1];
        content[RAM_SIZE_INDEX] = 0x06;
        let rom = Rom { content };
        assert!(rom.get_ram_size().is_err());
    }

    #[rstest]
    #[case(0x05)]
    #[case(0x06)]
    fn test_verify_memory_bank_matches_ram(#[case] byte: u8) {
        let mut content: Vec<u8> = vec![0; RAM_SIZE_INDEX + 1];
        content[CARTRIDGE_TYPE_INDEX] = byte;
        let rom = Rom { content };
        assert!(rom.verify_memory_bank_matches_ram().is_ok());
    }

    #[rstest]
    #[case(0x05)]
    #[case(0x06)]
    fn test_verify_memory_bank_matches_ram_negative(#[case] byte: u8) {
        let mut content: Vec<u8> = vec![0; RAM_SIZE_INDEX + 1];
        content[CARTRIDGE_TYPE_INDEX] = byte;
        content[RAM_SIZE_INDEX] = 0x02;
        let rom = Rom { content };
        assert!(rom.verify_memory_bank_matches_ram().is_err());
    }

    #[test]
    fn test_verify_header_checksum() {
        let header_checksum_source = vec![
            0x50, 0x4F, 0x4B, 0x45, 0x4D, 0x4F, 0x4E, 0x20, 0x42, 0x4C, 0x55, 0x45, 0x00, 0x00,
            0x00, 0x00, 0x30, 0x31, 0x03, 0x13, 0x05, 0x03, 0x01, 0x33, 0x00,
        ];

        let mut content: Vec<u8> = vec![0; HEADER_CHECKSUM_INDEX + 1];
        for byte in HEADER_CHECKSUM_RANGE {
            content[byte] = header_checksum_source[byte - HEADER_CHECKSUM_RANGE.start()];
        }

        content[0x14D] = 0xD3;

        let rom = Rom { content };
        assert!(rom.verify_header_checksum().is_ok());
    }

    #[test]
    fn test_verify_header_checksum_negative() {
        let mut content: Vec<u8> = vec![0; HEADER_CHECKSUM_INDEX + 1];
        let rom = Rom { content };
        assert!(rom.verify_header_checksum().is_err());
    }
}
