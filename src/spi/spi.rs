use volatile::Volatile;
use bit_field::BitField;
use embedded_hal::spi::{FullDuplex, Mode, Phase, Polarity};
use nb;
use crate::sim::Sim;
use crate::port::{Port, PortName};
use crate::sim::Clock::PortC;
use crate::sleep;
use crate::sleep::sleep_ms;
use core::mem;

#[repr(C, packed)]
pub struct Spi0 {
    mcr: Volatile<u32>,   // 000..004
    _pad0: [u8; 4],       // 004..008
    tcr: Volatile<u32>,   // 008..00c
    ctar0: Volatile<u32>, // 00c..010// alternatively slave
    ctar1: Volatile<u32>, // 010..014
    _pad1: [u8; 24],      // 014..02c
    sr: Volatile<u32>,    // 02c..030
    rser: Volatile<u32>,  // 030..034
    pushr: Volatile<u32>, // 034..038
    popr: Volatile<u32>,  // 038..03c
    txfr0: Volatile<u32>, // 03c..040
    txfr1: Volatile<u32>, // 040..044
    txfr2: Volatile<u32>, // 044..048
    txfr3: Volatile<u32>, // 048..04c
    _pad2: [u8; 48],      // 04c..07c
    rxfr0: Volatile<u32>, // 03c..040
    rxfr1: Volatile<u32>, // 040..044
    rxfr2: Volatile<u32>, // 044..048
    rxfr3: Volatile<u32>, // 048..04c
}


/// SPI error
#[derive(Debug)]
pub enum Error {
    /// Overrun occurred
    Overrun,
    /// Mode fault occurred
    ModeFault,
    /// CRC error
    Crc,
    #[doc(hidden)]
    _Extensible,
}


impl Spi0 {
    pub unsafe fn new() -> &'static mut Spi0 {
        &mut *(0x4002_C000 as *mut Spi0)
    }
}

impl Spi0 {
    // TODO: add cont: bool
    pub fn write(&mut self, b: u32) {
        // TODO: set PCS
        self.pushr.update(|pushr| {
            pushr.set_bits(0..9, b & 0xFF); // 8 bits of data (should this be framesize?)
            pushr.set_bits(16..22, 1 & 0b111111); // set asserted PCS
            pushr.set_bit(31, true); // keep asserted PCS
        });
        while self.sr.read().get_bits(12..16) > 3 {}

        // else
        //  sr[eoqf] = true // end of queue flag
        //  pushr = (last 8 bits of b) | (if cont, 0, else 0x80000000)
        //  if (cont)
        //   wait for txctr to clear
        //  else
        //   wait while(sr[eoqf]==0)
        //   *reg = 1????
    }
}

/*
    inline void write(uint32_t b, uint32_t cont=0) __attribute__((always_inline)) {
        // pcs is a u8
        // 00000001 << 16 = 00000000_00000001_00000000_00000000
		uint32_t pcsbits = pcs << 16;
		if (pcsbits) { // if it's not zero
			KINETISK_SPI0.PUSHR = (b & 0xFF) | pcsbits | (cont ? SPI_PUSHR_CONT : 0);
			while (((KINETISK_SPI0.SR) & (15 << 12)) > (3 << 12)) ; // wait if FIFO full
		} else {
			*reg = 0;
			KINETISK_SPI0.SR = SPI_SR_EOQF;
			KINETISK_SPI0.PUSHR = (b & 0xFF) | (cont ? 0 : SPI_PUSHR_EOQ);
			if (cont) {
				while (((KINETISK_SPI0.SR) & (15 << 12)) > (3 << 12)) ;
			} else {
				while (!(KINETISK_SPI0.SR & SPI_SR_EOQF)) ;
				*reg = 1;
			}
		}
	}

	inline void write16(uint32_t b, uint32_t cont=0) __attribute__((always_inline)) {
		uint32_t pcsbits = pcs << 16;
		if (pcsbits) {
			KINETISK_SPI0.PUSHR = (b & 0xFFFF) | (pcs << 16) |
				(cont ? SPI_PUSHR_CONT : 0) | SPI_PUSHR_CTAS(1);
			while (((KINETISK_SPI0.SR) & (15 << 12)) > (3 << 12)) ;
		} else {
			*reg = 0;
			KINETISK_SPI0.SR = SPI_SR_EOQF;
			KINETISK_SPI0.PUSHR = (b & 0xFFFF) | (cont ? 0 : SPI_PUSHR_EOQ) | SPI_PUSHR_CTAS(1);
			if (cont) {
				while (((KINETISK_SPI0.SR) & (15 << 12)) > (3 << 12)) ;
			} else {
				while (!(KINETISK_SPI0.SR & SPI_SR_EOQF)) ;
				*reg = 1;
			}
		}
	}
*/

impl Spi0 {
    pub fn start(&mut self) {
        unsafe {
            Sim::new().scgc6.update(|scgc6| {
                scgc6.set_bit(12, true); // enable spi0 clock
            });
        }

        self.master(); // enter master mode
        self.mcr.update(|mcr| {
            mcr.set_bit(14, false); // enable module (MDIS)
            mcr.set_bit(0, false); // start transfers? (HALT)
        });

        self.ctar0.update(|ctar0| {
            ctar0.set_bits(27..31, 7); // set frame size to 8
        });

        // setup pins
        unsafe {
            Port::new(PortName::D).pin(0).with_pin_mode(2); // PCS0
            Port::new(PortName::D).pin(1).with_pin_mode(2); // SCK
            Port::new(PortName::D).pin(2).with_pin_mode(2); // SOUT
            Port::new(PortName::D).pin(3).with_pin_mode(2); // SIN
        }

        // clear the queue (or is it buffers?)
        self.clear();
    }

    pub fn halt(&mut self) {
        // mcr[halt] = 1
    }

    pub fn clear(&mut self) {
        self.mcr.update(|mcr| {
            mcr.set_bit(31, true); // master mode
            mcr.set_bits(16..21, 0b11111); // set all PCS inactive states to high
            mcr.set_bit(11, true); // clear tx
            mcr.set_bit(10, true); // clear rx
        });
    }

    pub fn master(&mut self) {
        self.mcr.update(|mcr| {
            mcr.set_bit(31, true); // (MSTR)
        });
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.ctar0.update(|ctar0| {
            match mode.polarity {
                Polarity::IdleLow => { ctar0.set_bit(26, false); },
                Polarity::IdleHigh => { ctar0.set_bit(26, true); },
            }

            match mode.phase {
                Phase::CaptureOnFirstTransition => { ctar0.set_bit(25, false); },
                Phase::CaptureOnSecondTransition => { ctar0.set_bit(25, true); },
            }
        });
    }

    pub fn set_divider(&mut self, prescaler: u8, scaler: u16) {
        //TODO: fix this shit lmfao
        self.ctar0.update(|ctar0| {
            ctar0.set_bits(16..18, 0b00); // prescaler = 2
            ctar0.set_bit(31, false); // don't halve the baud rate
            ctar0.set_bits(0..4, 0b0011); // scaler = 6
        })
    }


}

impl FullDuplex<u8> for &mut Spi0 {
    type Error = Error;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        //while (self.sr.read() & (15 << 4)) == 0 {} // while there's nothing to read, wait...
        let a = self.popr.read();
        let b = unsafe {mem::transmute::<u32, [u8;4]>(a)};
        return Ok(b[3]);
    }

    fn send(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        let _ = self.read();
        self.write(word as u32);
        return Ok(());
    }
}

// TODO: SPI1