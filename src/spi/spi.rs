use volatile::Volatile;

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

// in the PDF's memory map, these are "together", but I've split them...
#[repr(C, packed)]
pub struct Spi1 {

}