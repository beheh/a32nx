mod logic;

pub trait FwcReaderWriter {
    fn read(&mut self, name: &str) -> f64;
    fn write(&mut self, name: &str, value: f64);
}

pub struct FwcWriter<'a> {
    fwc_read_writer: &'a mut dyn FwcReaderWriter,
}

impl<'a> FwcWriter<'a> {
    pub fn new(fwc_read_writer: &'a mut dyn FwcReaderWriter) -> Self {
        Self {
            fwc_read_writer,
        }
    }

    pub fn write_f64(&mut self, name: &str, value: f64) {
        self.fwc_read_writer.write(name, value);
    }
}
