use crate::datastructures::MemoryWord;
use crate::error::{TeXError, TeXResult};
use crate::{
    Global, HalfWord, Integer
};

use std::fs::File;
use std::io::{Read, Write};

pub(crate) enum ByteFileInSelector {
    TfmFile,
    FmtFile,
}

pub(crate) enum ByteFileOutSelector {
    DviFile,
    FmtFile
}

// Section 25
pub struct ByteFileIn {
    file: Option<File>,
    bytes: Option<&'static [u8]>
}

pub(crate) struct ByteFileOut {
    file: Option<File>
}

impl ByteFileIn {
    pub(crate) fn new() -> Self {
        Self {
            file: None,
            bytes: None
        }
    }

    pub fn close(&mut self) {
        self.file = None;
        self.bytes = None;
    }

    pub fn set_preloaded(&mut self, bytes: &'static [u8]) {
        self.bytes = Some(bytes);
    }

    pub(crate) fn fbyte(&mut self) -> Option<u8> {
        let mut buf = [0; 1];
        let result = if self.file.is_some() {
            self.file.as_mut().unwrap().read(&mut buf)
        }
        else {
            self.bytes.as_mut().unwrap().read(&mut buf)
        };
        match result {
            Ok(1) => Some(buf[0]),
            _ => {
                // EOF, or end of iterator or error.
                // If the caller really expected some bytes,
                // then it returns the appropriate TeXError
                self.close();
                None
            }
        }
    }

    pub(crate) fn read_wd(&mut self) -> Option<MemoryWord> {
        let mut buf = [0; 8];
        let result = if self.file.is_some() {
            self.file.as_mut().unwrap().read(&mut buf)
        }
        else {
            self.bytes.as_mut().unwrap().read(&mut buf)
        };
        match result {
            Ok(8) => {
                let b = u64::from_le_bytes(buf);
                Some(MemoryWord { word: b })
            },
            _ => {
                // EOF or error.
                // If the caller really expected some bytes,
                // then it returns the appropriate TeXError
                self.close();
                None
            }
        }
    }

    pub(crate) fn read_int(&mut self) -> Option<Integer> {
        let mut buf = [0; 4];
        let result = if self.file.is_some() {
            self.file.as_mut().unwrap().read(&mut buf)
        }
        else {
            self.bytes.as_mut().unwrap().read(&mut buf)
        };
        match result {
            Ok(4) => {
                Some(Integer::from_le_bytes(buf))
            },
            _ => {
                // EOF or error.
                // If the caller really expected some bytes,
                // then it returns the appropriate TeXError
                self.close();
                None
            }
        }
    }

    // Section 564
    pub(crate) fn read_sixteen(&mut self) -> Option<HalfWord> {
        let mut buf = [0; 2];
        let result = if self.file.is_some() {
            self.file.as_mut().unwrap().read(&mut buf)
        }
        else {
            self.bytes.as_mut().unwrap().read(&mut buf)
        };
        match result {
            Ok(2) => {
                if buf[0] > 127 {
                    None
                }
                else {
                    Some((buf[0] as HalfWord)*256 + buf[1] as HalfWord)
                }
            },
            _ => {
                // EOF or error.
                // If the caller really expected some bytes,
                // then it returns the appropriate TeXError
                self.close();
                None
            }
        }
    }
    
    pub(crate) fn read_four_quarters(&mut self) -> Option<(u8, u8, u8, u8)> {
        let mut buf = [0; 4];
        let result = if self.file.is_some() {
            self.file.as_mut().unwrap().read(&mut buf)
        }
        else {
            self.bytes.as_mut().unwrap().read(&mut buf)
        };
        match result {
            Ok(4) => {
                Some((buf[0], buf[1], buf[2], buf[3]))
            },
            _ => {
                // EOF or error.
                // If the caller really expected some bytes,
                // then it returns the appropriate TeXError
                self.close();
                None
            }
        }
    }
}

impl ByteFileOut {
    pub(crate) fn new() -> Self {
        Self {
            file: None
        }
    }

    pub(crate) fn close(&mut self) {
        self.file = None;
    }

    pub(crate) fn write(&mut self, b: &[u8]) {
        if self.file.as_mut().unwrap().write_all(b).is_err() {
            panic!();
        }
    }

    pub(crate) fn write_wd(&mut self, w: MemoryWord) {
        if self.file.as_mut().unwrap().write_all(&w.word().to_le_bytes()).is_err() {
            panic!()
        }
    }

    pub(crate) fn write_int(&mut self, x: Integer) {
        if self.file.as_mut().unwrap().write_all(&x.to_le_bytes()).is_err() {
            panic!();
        }
    }
}

impl Global {
    pub(crate) fn b_open_in(&mut self, selection: ByteFileInSelector) -> bool {
        match File::open(&self.name_of_file) {
            Ok(file) => {
                let byte_file = match selection {
                    ByteFileInSelector::TfmFile => &mut self.tfm_file,
                    ByteFileInSelector::FmtFile => &mut self.fmt_file,
                };
                byte_file.file = Some(file);
                if byte_file.bytes.is_some() {
                    byte_file.bytes = None;
                }
                true
            }
            Err(_) => false,
        }
    }

    pub(crate) fn b_open_out(&mut self, selection: ByteFileOutSelector) -> TeXResult<()> {
        match File::create(&self.name_of_file) {
            Ok(file) => {
                let byte_file = match selection {
                    ByteFileOutSelector::DviFile => &mut self.dvi_file,
                    ByteFileOutSelector::FmtFile => &mut self.fmt_file_out,
                };
                byte_file.file = Some(file);
                Ok(())
            },
            Err(_) => Err(TeXError::CantWriteFile),
        }
    }
}
