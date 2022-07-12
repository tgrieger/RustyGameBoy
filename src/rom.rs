use std::cmp::Ordering;
use std::io::{Error, ErrorKind, Result};

pub struct Rom {
    content: Vec<u8>,
}

impl Rom {
    pub fn new(path: &str) -> Result<Rom> {
        let rom = Rom {
            content: std::fs::read(path)?,
        };

        rom.verify_header()?;

        Ok(rom)
    }

    fn verify_header(&self) -> Result<()> {
        self.verify_nintendo_logo()?;

        Ok(())
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
}

const NINTENDO_LOGO: [u8; 48] = [
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
    0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
    0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];

const NINTENDO_LOGO_RANGE: std::ops::Range<usize> = 0x104..0x134;

#[cfg(test)]
mod tests {
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
}
