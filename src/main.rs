use num::complex::{self, ComplexFloat};
use num::{complex::Complex32, Complex};
use std::env::args;
use std::io::{BufRead, BufReader, Read, SeekFrom};
use std::{fs, io::Seek};

fn read_complex_value<T: Read>(source_stream: &mut T) -> Option<Complex<f32>> {
    let mut buf_re = [0; 4];
    let mut buf_im = [0; 4];
    source_stream.read_exact(&mut buf_re).ok()?;
    let re = f32::from_ne_bytes(buf_re);

    source_stream.read_exact(&mut buf_im).ok()?;
    let im = f32::from_ne_bytes(buf_im);
    return Some(Complex::new(re, im));
}

fn open_source_file() -> Option<fs::File> {
    let filename = args().nth(1)?;
    let mut file = fs::File::open(filename).ok()?;

    let file_len = file.seek(SeekFrom::End(0)).ok()?;
    if file_len & 3 != 0 {
        return None;
    }

    file.seek(SeekFrom::Start(0)).ok()?;
    Some(file)
}

// struct SampleGenerator {
//     file:
// }

trait FeedFwd {
    type ValueIn;
    type ValueOut;

    fn feed(&mut self, val: &Self::ValueIn) -> Self::ValueOut;
}

struct T1 {
    current_value: f32,
    response_const: f32,
}

impl FeedFwd for T1 {
    type ValueIn = f32;
    type ValueOut = f32;

    fn feed(&mut self, val: &Self::ValueIn) -> Self::ValueOut {
        self.current_value =
            (1.0 - self.response_const) * self.current_value + self.response_const * val;
        return self.current_value;
    }
}

struct NormSquared {}
impl FeedFwd for NormSquared {
    type ValueIn = Complex32;
    type ValueOut = f32;

    fn feed(&mut self, val: &Self::ValueIn) -> Self::ValueOut {
        return val.norm_sqr();
    }
}

struct Sqrt {}
impl FeedFwd for Sqrt {
    type ValueIn = f32;
    type ValueOut = f32;

    fn feed(&mut self, val: &Self::ValueIn) -> Self::ValueOut {
        return val.sqrt();
    }
}

struct Complex32ReadSource<T: Read> {
    file: T,
}

impl<T: Read> FeedFwd for Complex32ReadSource<T> {
    type ValueIn = ();
    type ValueOut = Complex32;

    fn feed(&mut self, val: &Self::ValueIn) -> Self::ValueOut {
        return read_complex_value(&mut self.file).unwrap();
    }
}

trait Block {
    type ValueOut;
    // fn to_handle(&mut self);
    fn update(&mut self);
    fn output(&self) -> &Self::ValueOut;
}

struct SourceBlock<T: FeedFwd<ValueIn = ()>> {
    inner: T,
    output: T::ValueOut,
}

impl<T: FeedFwd<ValueIn = ()>> Block for SourceBlock<T> {
    type ValueOut = T::ValueOut;
    fn update(&mut self) {
        self.output = self.inner.feed(&());
    }
    fn output(&self) -> &T::ValueOut {
        &self.output
    }
}

struct PullingFwdBlock<T: FeedFwd, S: Block<ValueOut = T::ValueIn>> {
    previous: S,
    inner: T,
    output: T::ValueOut,
}

impl<T, S> Block for PullingFwdBlock<T, S>
where
    T: FeedFwd,
    S: Block<ValueOut = T::ValueIn>,
{
    type ValueOut = T::ValueOut;
    fn update(&mut self) {
        self.previous.update();
        self.output = self.inner.feed(self.previous.output());
    }
    fn output(&self) -> &Self::ValueOut {
        &self.output
    }
}

struct DualFwdBlock<T: FeedFwd<ValueIn=(S1::ValueOut, S2::ValueOut)>, S1:Block, S2:Block> {
    previous1: S1,
    previous2: S2,
    inner: T,
    output: T::ValueOut
}

impl<T,S1,S2> Block for DualFwdBlock<T, S1, S2> 
where
    T: FeedFwd<ValueIn=(S1::ValueOut, S2::ValueOut)>, 
    S1:Block, 
    S2:Block
{
    type ValueOut = T::ValueOut;
    fn update(&mut self) {
        self.previous1.update();
        self.previous2.update();
        self.output = self.inner.feed( ( self.previous1.output(), self.previous2.output()) );
    }

    fn output(&self) -> &Self::ValueOut {
        return &self.output;    
    }
}

// struct BlockTracker {
//     blocks: Vec<Box<dyn Block>>
// }

fn main() -> Result<(), ()> {
    let mut source = BufReader::new(open_source_file().expect("Failed to open source-file!"));

    let mut samples: Vec<Complex32> = vec![];
    let mut rms: Vec<Complex32> = vec![];

    let source = SourceBlock {
        inner: Complex32ReadSource { file: source },
        output: Complex::new(0.0, 0.0),
    };

    let norm_squared = PullingFwdBlock {
        previous: source,
        inner: NormSquared {},
        output: 0.0,
    };

    let t1_block = PullingFwdBlock {
        previous: norm_squared,
        inner: T1 {
            current_value: 0.0,
            response_const: 0.05,
        },
        output: 0.0,
    };

    let mut rms_filtered = PullingFwdBlock {
        previous: t1_block,
        inner: Sqrt {},
        output: 0.0,
    };



    loop {
        rms_filtered.update();
        let value = rms_filtered.output();

        println!("{}", value);
    }

    Ok(())
}
