#![no_std]
#![no_main]

use cortex_m::asm::nop;
use defmt::*;
use embassy_executor::Spawner;
use embassy_nxp::pac::*;
use {defmt_rtt as _, panic_halt as _};

fn init() {
    info!("Initialization");

    // Pointers to the registers used

    let syscon = unsafe { &*SYSCON::ptr() };
    let flexcomm = unsafe { &*FLEXCOMM2::ptr() };
    let iocon = unsafe { &*IOCON::ptr() };
    let usart = unsafe { &*USART2::ptr() };

    // Enable clocks (Syscon is enabled by default)
    info!("Enable clocks");
    syscon.ahbclkctrl0.modify(|_, w| w.iocon().enable());
    syscon.ahbclkctrl1.modify(|_, w| w.fc2().enable());

    // Reset Flexcomm 2
    info!("Reset Flexcomm");
    syscon.presetctrl1.modify(|_, w| w.fc2_rst().set_bit());
    syscon.presetctrl1.modify(|_, w| w.fc2_rst().clear_bit());

    // Select the clock source for Flexcomm 2

    info!("Select clock");
    syscon.fcclksel2().write(|w| w.sel().enum_0x3());

    flexcomm.pselid.modify(|_, w| w.persel().usart());

    let flexcomm_cfg = flexcomm.pselid.read().bits();

    info!("PSELID: {:b}", flexcomm_cfg);

    // IOCON Setup
    info!("IOCON Setup");
    //iocon.pio1_4.modify(|_, w| w.func().alt1()); // clk
    iocon.pio1_24.modify(|_, w| {
        w.func()
            .alt1()
            .digimode()
            .digital()
            .slew()
            .standard()
            .mode()
            .inactive()
            .invert()
            .disabled()
            .od()
            .normal()
    }); // rx
    iocon.pio0_27.modify(|_, w| {
        w.func()
            .alt1()
            .digimode()
            .digital()
            .slew()
            .standard()
            .mode()
            .inactive()
            .invert()
            .disabled()
            .od()
            .normal()
    }); // tx
        //iocon.pio0_31.modify(|_, w| w.func().alt1()); // cts
        //iocon.pio1_0.modify(|_, w| w.func().alt1()); // rts

    // Flexcomm interface clock and USART baud rate
    info!("Flexcomm clock");
    syscon
        .flexfrg2ctrl()
        .modify(|_, w| unsafe { w.div().bits(2).mult().bits(0) }); // 12 MHz

    info!("Baud rate config");
    // The clock is divided by 16 afterwards
    usart.brg.modify(|_, w| unsafe { w.brgval().bits(52) }); //115200 approximately

    info!("USART Config");
    // USART configuration part
    usart.cfg.modify(|_, w| {
        w.datalen()
            .bit_8()
            .paritysel()
            .even_parity()
            .stoplen()
            .bits_2()
            .syncen()
            .asynchronous_mode()
            .clkpol()
            .rising_edge()
            .loop_()
            .normal()
            .rxpol()
            .standard()
            .mode32k()
            .disabled()
            .txpol()
            .standard()
    });

    // By default, the oversampling rate is 16x

    // FIFO Configuration

    info!("Disabling DMA");
    usart.fifocfg.modify(|_, w| w.dmatx().disabled().dmarx().disabled());

    // USART Interrupts

    // FIFO Interrupts

    // Enable interrupts

    /* Cortex Interrupts enabling
    let mut cp = cortex_m::peripheral::Peripherals::take().unwrap();
    unsafe { cp.NVIC.set_priority(interrupt::FLEXCOMM0, 3) };
    unsafe { cortex_m::peripheral::NVIC::unmask(interrupt::FLEXCOMM0) };
    */

    // Enable FIFO and USART

    info!("After all settings, enable fifo");
    usart.fifocfg.modify(|_, w| w.enabletx().enabled().enablerx().enabled());

    for _ in 0..200_000 {
        nop();
    }

    info!("Enable USART");
    usart.cfg.modify(|_, w| w.enable().enabled());

    for _ in 0..200_000 {
        nop();
    }

    let usart_cfg = usart.cfg.read().bits();

    info!("USART Bits: {:b}", usart_cfg);

    let fifo_cfg = usart.fifocfg.read().bits();

    info!("FIFO Bits: {:b}", fifo_cfg);

    // Write, Read USART trial

    if usart.fifostat.read().txempty().bit_is_set() {
        info!("TX FIFO is empty");
    }

    info!("Write 25 to the register");
    usart.fifowr.write(|w| unsafe { w.txdata().bits(25) });

    if usart.fifostat.read().txempty().bit_is_clear() {
        info!("The data was written successfully");
    }

    let stat = usart.fifostat.read();
    info!("FIFOSTAT: {:b}", stat.bits());

    /*while usart.fifostat.read().rxnotempty().bit_is_clear() {
        nop();
    }

    let result = usart.fiford.read().rxdata().bits();

    info!("Result: {}", result);*/
    for _ in 0..5 {
        info!("Write 25 to the register");
        usart.fifowr.write(|w| unsafe { w.txdata().bits(25) });

        if usart.fifostat.read().txempty().bit_is_clear() {
            info!("The data was written successfully");
        }

        let stat = usart.fifostat.read();
        info!("FIFOSTAT: {:b}", stat.bits());

        /*while usart.fifostat.read().rxnotempty().bit_is_clear() {
            nop();
        }

        let result = usart.fiford.read().rxdata().bits();

        info!("Result: {}", result);*/
        for _ in 0..200_000 {
            nop()
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    init();
    loop {}
}
