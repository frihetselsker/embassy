use core::ops::RangeInclusive;

#[cfg(stm32h7rs)]
use stm32_metapac::rcc::vals::Plldivst;

use crate::pac;
pub use crate::pac::rcc::vals::{
    Hsidiv as HSIPrescaler, Plldiv as PllDiv, Pllm as PllPreDiv, Plln as PllMul, Pllsrc as PllSource, Sw as Sysclk,
};
use crate::pac::rcc::vals::{Pllrge, Pllvcosel, Timpre};
use crate::pac::{FLASH, PWR, RCC};
use crate::time::Hertz;

/// HSI speed
pub const HSI_FREQ: Hertz = Hertz(64_000_000);

/// CSI speed
pub const CSI_FREQ: Hertz = Hertz(4_000_000);

const VCO_RANGE: RangeInclusive<Hertz> = Hertz(150_000_000)..=Hertz(420_000_000);
#[cfg(any(stm32h5, pwr_h7rm0455))]
const VCO_WIDE_RANGE: RangeInclusive<Hertz> = Hertz(128_000_000)..=Hertz(560_000_000);
#[cfg(pwr_h7rm0468)]
const VCO_WIDE_RANGE: RangeInclusive<Hertz> = Hertz(192_000_000)..=Hertz(836_000_000);
#[cfg(any(pwr_h7rm0399, pwr_h7rm0433))]
const VCO_WIDE_RANGE: RangeInclusive<Hertz> = Hertz(192_000_000)..=Hertz(960_000_000);
#[cfg(any(stm32h7rs))]
const VCO_WIDE_RANGE: RangeInclusive<Hertz> = Hertz(384_000_000)..=Hertz(1672_000_000);

pub use crate::pac::rcc::vals::{Hpre as AHBPrescaler, Ppre as APBPrescaler};

#[cfg(any(stm32h5, stm32h7))]
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum VoltageScale {
    Scale0,
    Scale1,
    Scale2,
    Scale3,
}
#[cfg(stm32h7rs)]
pub use crate::pac::pwr::vals::Vos as VoltageScale;
#[cfg(all(stm32h7rs, peri_usb_otg_hs))]
pub use crate::pac::rcc::vals::{Usbphycsel, Usbrefcksel};

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum HseMode {
    /// crystal/ceramic oscillator (HSEBYP=0)
    Oscillator,
    /// external analog clock (low swing) (HSEBYP=1, HSEEXT=0)
    Bypass,
    /// external digital clock (full swing) (HSEBYP=1, HSEEXT=1)
    #[cfg(any(rcc_h5, rcc_h50, rcc_h7rs))]
    BypassDigital,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Hse {
    /// HSE frequency.
    pub freq: Hertz,
    /// HSE mode.
    pub mode: HseMode,
}

#[derive(Clone, Copy)]
pub struct Pll {
    /// Source clock selection.
    pub source: PllSource,

    /// PLL pre-divider (DIVM).
    pub prediv: PllPreDiv,

    /// PLL multiplication factor.
    pub mul: PllMul,

    /// PLL P division factor. If None, PLL P output is disabled.
    /// On PLL1, it must be even for most series (in particular,
    /// it cannot be 1 in series other than stm32h7, stm32h7rs23/733,
    /// stm32h7, stm32h7rs25/735 and stm32h7, stm32h7rs30.)
    pub divp: Option<PllDiv>,
    /// PLL Q division factor. If None, PLL Q output is disabled.
    pub divq: Option<PllDiv>,
    /// PLL R division factor. If None, PLL R output is disabled.
    pub divr: Option<PllDiv>,
    #[cfg(stm32h7rs)]
    /// PLL S division factor. If None, PLL S output is disabled.
    pub divs: Option<Plldivst>,
    #[cfg(stm32h7rs)]
    /// PLL T division factor. If None, PLL T output is disabled.
    pub divt: Option<Plldivst>,
}

fn apb_div_tim(apb: &APBPrescaler, clk: Hertz, tim: TimerPrescaler) -> Hertz {
    match (tim, apb) {
        (TimerPrescaler::DefaultX2, APBPrescaler::DIV1) => clk,
        (TimerPrescaler::DefaultX2, APBPrescaler::DIV2) => clk,
        (TimerPrescaler::DefaultX2, APBPrescaler::DIV4) => clk / 2u32,
        (TimerPrescaler::DefaultX2, APBPrescaler::DIV8) => clk / 4u32,
        (TimerPrescaler::DefaultX2, APBPrescaler::DIV16) => clk / 8u32,

        (TimerPrescaler::DefaultX4, APBPrescaler::DIV1) => clk,
        (TimerPrescaler::DefaultX4, APBPrescaler::DIV2) => clk,
        (TimerPrescaler::DefaultX4, APBPrescaler::DIV4) => clk,
        (TimerPrescaler::DefaultX4, APBPrescaler::DIV8) => clk / 2u32,
        (TimerPrescaler::DefaultX4, APBPrescaler::DIV16) => clk / 4u32,

        _ => unreachable!(),
    }
}

/// Timer prescaler
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum TimerPrescaler {
    /// The timers kernel clock is equal to hclk if PPREx corresponds to a
    /// division by 1 or 2, else it is equal to 2*pclk
    DefaultX2,

    /// The timers kernel clock is equal to hclk if PPREx corresponds to a
    /// division by 1, 2 or 4, else it is equal to 4*pclk
    DefaultX4,
}

impl From<TimerPrescaler> for Timpre {
    fn from(value: TimerPrescaler) -> Self {
        match value {
            TimerPrescaler::DefaultX2 => Timpre::DEFAULT_X2,
            TimerPrescaler::DefaultX4 => Timpre::DEFAULT_X4,
        }
    }
}

/// Power supply configuration
/// See RM0433 Rev 4 7.4
#[cfg(any(pwr_h7rm0399, pwr_h7rm0455, pwr_h7rm0468, pwr_h7rs))]
#[derive(Clone, Copy, PartialEq)]
pub enum SupplyConfig {
    /// Default power supply configuration.
    /// V CORE Power Domains are supplied from the LDO according to VOS.
    /// SMPS step-down converter enabled at 1.2V, may be used to supply the LDO.
    Default,

    /// Power supply configuration using the LDO.
    /// V CORE Power Domains are supplied from the LDO according to VOS.
    /// LDO power mode (Main, LP, Off) will follow system low-power modes.
    /// SMPS step-down converter disabled.
    LDO,

    /// Power supply configuration directly from the SMPS step-down converter.
    /// V CORE Power Domains are supplied from SMPS step-down converter according to VOS.
    /// LDO bypassed.
    /// SMPS step-down converter power mode (MR, LP, Off) will follow system low-power modes.
    DirectSMPS,

    /// Power supply configuration from the SMPS step-down converter, that supplies the LDO.
    /// V CORE Power Domains are supplied from the LDO according to VOS
    /// LDO power mode (Main, LP, Off) will follow system low-power modes.
    /// SMPS step-down converter enabled according to SDLEVEL, and supplies the LDO.
    /// SMPS step-down converter power mode (MR, LP, Off) will follow system low-power modes.
    SMPSLDO(SMPSSupplyVoltage),

    /// Power supply configuration from SMPS supplying external circuits and potentially the LDO.
    /// V CORE Power Domains are supplied from voltage regulator according to VOS
    /// LDO power mode (Main, LP, Off) will follow system low-power modes.
    /// SMPS step-down converter enabled according to SDLEVEL used to supply external circuits and may supply the LDO.
    /// SMPS step-down converter forced ON in MR mode.
    SMPSExternalLDO(SMPSSupplyVoltage),

    /// Power supply configuration from SMPS supplying external circuits and bypassing the LDO.
    /// V CORE supplied from external source
    /// SMPS step-down converter enabled according to SDLEVEL used to supply external circuits and may supply the external source for V CORE .
    /// SMPS step-down converter forced ON in MR mode.
    SMPSExternalLDOBypass(SMPSSupplyVoltage),

    /// Power supply configuration from an external source, SMPS disabled and the LDO bypassed.
    /// V CORE supplied from external source
    /// SMPS step-down converter disabled and LDO bypassed, voltage monitoring still active.
    SMPSDisabledLDOBypass,
}

/// SMPS step-down converter voltage output level.
/// This is only used in certain power supply configurations:
/// SMPSLDO, SMPSExternalLDO, SMPSExternalLDOBypass.
#[cfg(any(pwr_h7rm0399, pwr_h7rm0455, pwr_h7rm0468, pwr_h7rs))]
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum SMPSSupplyVoltage {
    /// 1.8v
    V1_8,
    /// 2.5v
    #[cfg(not(pwr_h7rs))]
    V2_5,
}

/// Configuration of the core clocks
#[non_exhaustive]
#[derive(Clone, Copy)]
pub struct Config {
    pub hsi: Option<HSIPrescaler>,
    pub hse: Option<Hse>,
    pub csi: bool,
    pub hsi48: Option<super::Hsi48Config>,
    pub sys: Sysclk,

    pub pll1: Option<Pll>,
    pub pll2: Option<Pll>,
    #[cfg(any(rcc_h5, stm32h7, stm32h7rs))]
    pub pll3: Option<Pll>,

    #[cfg(any(stm32h7, stm32h7rs))]
    pub d1c_pre: AHBPrescaler,
    pub ahb_pre: AHBPrescaler,
    pub apb1_pre: APBPrescaler,
    pub apb2_pre: APBPrescaler,
    #[cfg(not(stm32h7rs))]
    pub apb3_pre: APBPrescaler,
    #[cfg(any(stm32h7, stm32h7rs))]
    pub apb4_pre: APBPrescaler,
    #[cfg(stm32h7rs)]
    pub apb5_pre: APBPrescaler,

    pub timer_prescaler: TimerPrescaler,
    pub voltage_scale: VoltageScale,
    pub ls: super::LsConfig,

    #[cfg(any(pwr_h7rm0399, pwr_h7rm0455, pwr_h7rm0468, pwr_h7rs))]
    pub supply_config: SupplyConfig,

    /// Per-peripheral kernel clock selection muxes
    pub mux: super::mux::ClockMux,
}

impl Config {
    pub const fn new() -> Self {
        Self {
            hsi: Some(HSIPrescaler::DIV1),
            hse: None,
            csi: false,
            hsi48: Some(crate::rcc::Hsi48Config::new()),
            sys: Sysclk::HSI,
            pll1: None,
            pll2: None,
            #[cfg(any(rcc_h5, stm32h7, stm32h7rs))]
            pll3: None,

            #[cfg(any(stm32h7, stm32h7rs))]
            d1c_pre: AHBPrescaler::DIV1,
            ahb_pre: AHBPrescaler::DIV1,
            apb1_pre: APBPrescaler::DIV1,
            apb2_pre: APBPrescaler::DIV1,
            #[cfg(not(stm32h7rs))]
            apb3_pre: APBPrescaler::DIV1,
            #[cfg(any(stm32h7, stm32h7rs))]
            apb4_pre: APBPrescaler::DIV1,
            #[cfg(stm32h7rs)]
            apb5_pre: APBPrescaler::DIV1,

            timer_prescaler: TimerPrescaler::DefaultX2,
            #[cfg(not(rcc_h7rs))]
            voltage_scale: VoltageScale::Scale0,
            #[cfg(rcc_h7rs)]
            voltage_scale: VoltageScale::HIGH,
            ls: crate::rcc::LsConfig::new(),

            #[cfg(any(pwr_h7rm0399, pwr_h7rm0455, pwr_h7rm0468, pwr_h7rs))]
            supply_config: SupplyConfig::LDO,

            mux: super::mux::ClockMux::default(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) unsafe fn init(config: Config) {
    #[cfg(any(stm32h7))]
    let pwr_reg = PWR.cr3();
    #[cfg(any(stm32h7rs))]
    let pwr_reg = PWR.csr2();

    // NB. The lower bytes of CR3 can only be written once after
    // POR, and must be written with a valid combination. Refer to
    // RM0433 Rev 7 6.8.4. This is partially enforced by dropping
    // `self` at the end of this method, but of course we cannot
    // know what happened between the previous POR and here.
    #[cfg(pwr_h7rm0433)]
    pwr_reg.modify(|w| {
        w.set_scuen(true);
        w.set_ldoen(true);
        w.set_bypass(false);
    });

    #[cfg(any(pwr_h7rm0399, pwr_h7rm0455, pwr_h7rm0468, pwr_h7rs))]
    {
        use pac::pwr::vals::Sdlevel;
        match config.supply_config {
            SupplyConfig::Default => {
                pwr_reg.modify(|w| {
                    w.set_sdlevel(Sdlevel::RESET);
                    w.set_sdexthp(false);
                    w.set_sden(true);
                    w.set_ldoen(true);
                    w.set_bypass(false);
                });
            }
            SupplyConfig::LDO => {
                pwr_reg.modify(|w| {
                    w.set_sden(false);
                    w.set_ldoen(true);
                    w.set_bypass(false);
                });
            }
            SupplyConfig::DirectSMPS => {
                pwr_reg.modify(|w| {
                    w.set_sdexthp(false);
                    w.set_sden(true);
                    w.set_ldoen(false);
                    w.set_bypass(false);
                });
            }
            SupplyConfig::SMPSLDO(sdlevel)
            | SupplyConfig::SMPSExternalLDO(sdlevel)
            | SupplyConfig::SMPSExternalLDOBypass(sdlevel) => {
                let sdlevel = match sdlevel {
                    SMPSSupplyVoltage::V1_8 => Sdlevel::V1_8,
                    #[cfg(not(pwr_h7rs))]
                    SMPSSupplyVoltage::V2_5 => Sdlevel::V2_5,
                };
                pwr_reg.modify(|w| {
                    w.set_sdlevel(sdlevel);
                    w.set_sdexthp(matches!(
                        config.supply_config,
                        SupplyConfig::SMPSExternalLDO(_) | SupplyConfig::SMPSExternalLDOBypass(_)
                    ));
                    w.set_sden(true);
                    w.set_ldoen(matches!(
                        config.supply_config,
                        SupplyConfig::SMPSLDO(_) | SupplyConfig::SMPSExternalLDO(_)
                    ));
                    w.set_bypass(matches!(config.supply_config, SupplyConfig::SMPSExternalLDOBypass(_)));
                });
            }
            SupplyConfig::SMPSDisabledLDOBypass => {
                pwr_reg.modify(|w| {
                    w.set_sden(false);
                    w.set_ldoen(false);
                    w.set_bypass(true);
                });
            }
        }
    }

    // Validate the supply configuration. If you are stuck here, it is
    // because the voltages on your board do not match those specified
    // in the D3CR.VOS and CR3.SDLEVEL fields. By default after reset
    // VOS = Scale 3, so check that the voltage on the VCAP pins =
    // 1.0V.
    #[cfg(any(stm32h7))]
    while !PWR.csr1().read().actvosrdy() {}
    #[cfg(any(stm32h7rs))]
    while !PWR.sr1().read().actvosrdy() {}

    // Configure voltage scale.
    #[cfg(any(pwr_h5, pwr_h50))]
    {
        PWR.voscr().modify(|w| {
            w.set_vos(match config.voltage_scale {
                VoltageScale::Scale0 => crate::pac::pwr::vals::Vos::SCALE0,
                VoltageScale::Scale1 => crate::pac::pwr::vals::Vos::SCALE1,
                VoltageScale::Scale2 => crate::pac::pwr::vals::Vos::SCALE2,
                VoltageScale::Scale3 => crate::pac::pwr::vals::Vos::SCALE3,
            })
        });
        while !PWR.vossr().read().vosrdy() {}
    }
    #[cfg(syscfg_h7)]
    {
        // in chips without the overdrive bit, we can go from any scale to any scale directly.
        PWR.d3cr().modify(|w| {
            w.set_vos(match config.voltage_scale {
                VoltageScale::Scale0 => crate::pac::pwr::vals::Vos::SCALE0,
                VoltageScale::Scale1 => crate::pac::pwr::vals::Vos::SCALE1,
                VoltageScale::Scale2 => crate::pac::pwr::vals::Vos::SCALE2,
                VoltageScale::Scale3 => crate::pac::pwr::vals::Vos::SCALE3,
            })
        });
        while !PWR.d3cr().read().vosrdy() {}
    }
    #[cfg(pwr_h7rs)]
    {
        PWR.csr4().modify(|w| w.set_vos(config.voltage_scale));
        while !PWR.csr4().read().vosrdy() {}
    }

    #[cfg(syscfg_h7od)]
    {
        match config.voltage_scale {
            VoltageScale::Scale0 => {
                // to go to scale0, we must go to Scale1 first...
                PWR.d3cr().modify(|w| w.set_vos(crate::pac::pwr::vals::Vos::SCALE1));
                while !PWR.d3cr().read().vosrdy() {}

                // Then enable overdrive.
                critical_section::with(|_| pac::SYSCFG.pwrcr().modify(|w| w.set_oden(1)));
                while !PWR.d3cr().read().vosrdy() {}
            }
            _ => {
                // for all other scales, we can go directly.
                PWR.d3cr().modify(|w| {
                    w.set_vos(match config.voltage_scale {
                        VoltageScale::Scale0 => unreachable!(),
                        VoltageScale::Scale1 => crate::pac::pwr::vals::Vos::SCALE1,
                        VoltageScale::Scale2 => crate::pac::pwr::vals::Vos::SCALE2,
                        VoltageScale::Scale3 => crate::pac::pwr::vals::Vos::SCALE3,
                    })
                });
                while !PWR.d3cr().read().vosrdy() {}
            }
        }
    }

    // Turn on the HSI
    match config.hsi {
        None => RCC.cr().modify(|w| w.set_hsion(true)),
        Some(hsidiv) => RCC.cr().modify(|w| {
            w.set_hsidiv(hsidiv);
            w.set_hsion(true);
        }),
    }
    while !RCC.cr().read().hsirdy() {}

    // Use the HSI clock as system clock during the actual clock setup
    RCC.cfgr().modify(|w| w.set_sw(Sysclk::HSI));
    while RCC.cfgr().read().sws() != Sysclk::HSI {}

    // Configure HSI
    let hsi = match config.hsi {
        None => None,
        Some(hsidiv) => Some(HSI_FREQ / hsidiv),
    };

    // Configure HSE
    let hse = match config.hse {
        None => {
            RCC.cr().modify(|w| w.set_hseon(false));
            None
        }
        Some(hse) => {
            RCC.cr().modify(|w| {
                w.set_hsebyp(hse.mode != HseMode::Oscillator);
                #[cfg(any(rcc_h5, rcc_h50, rcc_h7rs))]
                w.set_hseext(match hse.mode {
                    HseMode::Oscillator | HseMode::Bypass => pac::rcc::vals::Hseext::ANALOG,
                    HseMode::BypassDigital => pac::rcc::vals::Hseext::DIGITAL,
                });
            });
            RCC.cr().modify(|w| w.set_hseon(true));
            while !RCC.cr().read().hserdy() {}
            Some(hse.freq)
        }
    };

    // Configure HSI48.
    let hsi48 = config.hsi48.map(super::init_hsi48);

    // Configure CSI.
    RCC.cr().modify(|w| w.set_csion(config.csi));
    let csi = match config.csi {
        false => None,
        true => {
            while !RCC.cr().read().csirdy() {}
            Some(CSI_FREQ)
        }
    };

    // H7 has shared PLLSRC, check it's equal in all PLLs.
    #[cfg(any(stm32h7, stm32h7rs))]
    {
        let plls = [&config.pll1, &config.pll2, &config.pll3];
        if !super::util::all_equal(plls.into_iter().flatten().map(|p| p.source)) {
            panic!("Source must be equal across all enabled PLLs.")
        };
    }

    // Configure PLLs.
    let pll_input = PllInput { csi, hse, hsi };
    let pll1 = init_pll(0, config.pll1, &pll_input);
    let pll2 = init_pll(1, config.pll2, &pll_input);
    #[cfg(any(rcc_h5, stm32h7, stm32h7rs))]
    let pll3 = init_pll(2, config.pll3, &pll_input);

    // Configure sysclk
    let sys = match config.sys {
        Sysclk::HSI => unwrap!(hsi),
        Sysclk::HSE => unwrap!(hse),
        Sysclk::CSI => unwrap!(csi),
        Sysclk::PLL1_P => unwrap!(pll1.p),
        _ => unreachable!(),
    };

    // Check limits.
    #[cfg(stm32h5)]
    let (hclk_max, pclk_max) = match config.voltage_scale {
        VoltageScale::Scale0 => (Hertz(250_000_000), Hertz(250_000_000)),
        VoltageScale::Scale1 => (Hertz(200_000_000), Hertz(200_000_000)),
        VoltageScale::Scale2 => (Hertz(150_000_000), Hertz(150_000_000)),
        VoltageScale::Scale3 => (Hertz(100_000_000), Hertz(100_000_000)),
    };
    #[cfg(pwr_h7rm0455)]
    let (d1cpre_clk_max, hclk_max, pclk_max) = match config.voltage_scale {
        VoltageScale::Scale0 => (Hertz(280_000_000), Hertz(280_000_000), Hertz(140_000_000)),
        VoltageScale::Scale1 => (Hertz(225_000_000), Hertz(225_000_000), Hertz(112_500_000)),
        VoltageScale::Scale2 => (Hertz(160_000_000), Hertz(160_000_000), Hertz(80_000_000)),
        VoltageScale::Scale3 => (Hertz(88_000_000), Hertz(88_000_000), Hertz(44_000_000)),
    };
    #[cfg(pwr_h7rm0468)]
    let (d1cpre_clk_max, hclk_max, pclk_max) = match config.voltage_scale {
        VoltageScale::Scale0 => {
            let d1cpre_clk_max = if pac::SYSCFG.ur18().read().cpu_freq_boost() {
                550_000_000
            } else {
                520_000_000
            };
            (Hertz(d1cpre_clk_max), Hertz(275_000_000), Hertz(137_500_000))
        }
        VoltageScale::Scale1 => (Hertz(400_000_000), Hertz(200_000_000), Hertz(100_000_000)),
        VoltageScale::Scale2 => (Hertz(300_000_000), Hertz(150_000_000), Hertz(75_000_000)),
        VoltageScale::Scale3 => (Hertz(170_000_000), Hertz(85_000_000), Hertz(42_500_000)),
    };
    #[cfg(all(stm32h7, not(any(pwr_h7rm0455, pwr_h7rm0468))))]
    let (d1cpre_clk_max, hclk_max, pclk_max) = match config.voltage_scale {
        VoltageScale::Scale0 => (Hertz(480_000_000), Hertz(240_000_000), Hertz(120_000_000)),
        VoltageScale::Scale1 => (Hertz(400_000_000), Hertz(200_000_000), Hertz(100_000_000)),
        VoltageScale::Scale2 => (Hertz(300_000_000), Hertz(150_000_000), Hertz(75_000_000)),
        VoltageScale::Scale3 => (Hertz(200_000_000), Hertz(100_000_000), Hertz(50_000_000)),
    };
    #[cfg(stm32h7rs)]
    let (d1cpre_clk_max, hclk_max, pclk_max) = match config.voltage_scale {
        VoltageScale::HIGH => (Hertz(600_000_000), Hertz(300_000_000), Hertz(150_000_000)),
        VoltageScale::LOW => (Hertz(400_000_000), Hertz(200_000_000), Hertz(100_000_000)),
    };

    #[cfg(any(stm32h7, stm32h7rs))]
    let hclk = {
        let d1cpre_clk = sys / config.d1c_pre;
        assert!(d1cpre_clk <= d1cpre_clk_max);
        sys / config.ahb_pre
    };
    #[cfg(stm32h5)]
    let hclk = sys / config.ahb_pre;
    assert!(hclk <= hclk_max);

    let apb1 = hclk / config.apb1_pre;
    let apb1_tim = apb_div_tim(&config.apb1_pre, hclk, config.timer_prescaler);
    assert!(apb1 <= pclk_max);
    let apb2 = hclk / config.apb2_pre;
    let apb2_tim = apb_div_tim(&config.apb2_pre, hclk, config.timer_prescaler);
    assert!(apb2 <= pclk_max);
    #[cfg(not(stm32h7rs))]
    let apb3 = hclk / config.apb3_pre;
    #[cfg(not(stm32h7rs))]
    assert!(apb3 <= pclk_max);
    #[cfg(any(stm32h7, stm32h7rs))]
    let apb4 = hclk / config.apb4_pre;
    #[cfg(any(stm32h7, stm32h7rs))]
    assert!(apb4 <= pclk_max);
    #[cfg(stm32h7rs)]
    let apb5 = hclk / config.apb5_pre;
    #[cfg(stm32h7rs)]
    assert!(apb5 <= pclk_max);

    flash_setup(hclk, config.voltage_scale);

    let rtc = config.ls.init();

    #[cfg(all(stm32h7rs, peri_usb_otg_hs))]
    let usb_refck = match config.mux.usbphycsel {
        Usbphycsel::HSE => hse,
        Usbphycsel::HSE_DIV_2 => hse.map(|hse_val| hse_val / 2u8),
        Usbphycsel::PLL3_Q => pll3.q,
        _ => None,
    };
    #[cfg(all(stm32h7rs, peri_usb_otg_hs))]
    let usb_refck_sel = match usb_refck {
        Some(clk_val) => match clk_val {
            Hertz(16_000_000) => Usbrefcksel::MHZ16,
            Hertz(19_200_000) => Usbrefcksel::MHZ19_2,
            Hertz(20_000_000) => Usbrefcksel::MHZ20,
            Hertz(24_000_000) => Usbrefcksel::MHZ24,
            Hertz(26_000_000) => Usbrefcksel::MHZ26,
            Hertz(32_000_000) => Usbrefcksel::MHZ32,
            _ => panic!("cannot select USBPHYC reference clock with source frequency of {}, must be one of 16, 19.2, 20, 24, 26, 32 MHz", clk_val),
        },
        None => Usbrefcksel::MHZ24,
    };

    #[cfg(stm32h7)]
    {
        RCC.d1cfgr().modify(|w| {
            w.set_d1cpre(config.d1c_pre);
            w.set_d1ppre(config.apb3_pre);
            w.set_hpre(config.ahb_pre);
        });
        // Ensure core prescaler value is valid before future lower core voltage
        while RCC.d1cfgr().read().d1cpre() != config.d1c_pre {}

        RCC.d2cfgr().modify(|w| {
            w.set_d2ppre1(config.apb1_pre);
            w.set_d2ppre2(config.apb2_pre);
        });
        RCC.d3cfgr().modify(|w| {
            w.set_d3ppre(config.apb4_pre);
        });
    }
    #[cfg(stm32h7rs)]
    {
        RCC.cdcfgr().write(|w| {
            w.set_cpre(config.d1c_pre);
        });
        while RCC.cdcfgr().read().cpre() != config.d1c_pre {}

        RCC.bmcfgr().write(|w| {
            w.set_bmpre(config.ahb_pre);
        });
        while RCC.bmcfgr().read().bmpre() != config.ahb_pre {}

        RCC.apbcfgr().modify(|w| {
            w.set_ppre1(config.apb1_pre);
            w.set_ppre2(config.apb2_pre);
            w.set_ppre4(config.apb4_pre);
            w.set_ppre5(config.apb5_pre);
        });

        #[cfg(peri_usb_otg_hs)]
        RCC.ahbperckselr().modify(|w| {
            w.set_usbrefcksel(usb_refck_sel);
        });
    }
    #[cfg(stm32h5)]
    {
        // Set hpre
        RCC.cfgr2().modify(|w| w.set_hpre(config.ahb_pre));
        while RCC.cfgr2().read().hpre() != config.ahb_pre {}

        // set ppre
        RCC.cfgr2().modify(|w| {
            w.set_ppre1(config.apb1_pre);
            w.set_ppre2(config.apb2_pre);
            w.set_ppre3(config.apb3_pre);
        });
    }

    RCC.cfgr().modify(|w| w.set_timpre(config.timer_prescaler.into()));

    RCC.cfgr().modify(|w| w.set_sw(config.sys));
    while RCC.cfgr().read().sws() != config.sys {}

    // Disable HSI if not used
    if config.hsi.is_none() {
        RCC.cr().modify(|w| w.set_hsion(false));
    }

    // IO compensation cell - Requires CSI clock and SYSCFG
    #[cfg(any(stm32h7))] // TODO h5, h7rs
    if csi.is_some() {
        // Enable the compensation cell, using back-bias voltage code
        // provide by the cell.
        critical_section::with(|_| {
            pac::SYSCFG.cccsr().modify(|w| {
                w.set_en(true);
                w.set_cs(false);
                w.set_hslv(false);
            })
        });
        while !pac::SYSCFG.cccsr().read().rdy() {}
    }

    config.mux.init();

    set_clocks!(
        sys: Some(sys),
        hclk1: Some(hclk),
        hclk2: Some(hclk),
        hclk3: Some(hclk),
        hclk4: Some(hclk),
        #[cfg(stm32h7rs)]
        hclk5: Some(hclk),
        pclk1: Some(apb1),
        pclk2: Some(apb2),
        #[cfg(not(stm32h7rs))]
        pclk3: Some(apb3),
        #[cfg(any(stm32h7, stm32h7rs))]
        pclk4: Some(apb4),
        #[cfg(stm32h7rs)]
        pclk5: Some(apb5),

        pclk1_tim: Some(apb1_tim),
        pclk2_tim: Some(apb2_tim),
        rtc: rtc,

        hsi: hsi,
        hsi48: hsi48,
        csi: csi,
        hse: hse,

        lse: None,
        lsi: None,

        pll1_q: pll1.q,
        pll2_p: pll2.p,
        pll2_q: pll2.q,
        pll2_r: pll2.r,
        #[cfg(stm32h7rs)]
        pll2_s: None, // TODO
        #[cfg(stm32h7rs)]
        pll2_t: None, // TODO
        #[cfg(any(rcc_h5, stm32h7, stm32h7rs))]
        pll3_p: pll3.p,
        #[cfg(any(rcc_h5, stm32h7, stm32h7rs))]
        pll3_q: pll3.q,
        #[cfg(any(rcc_h5, stm32h7, stm32h7rs))]
        pll3_r: pll3.r,

        #[cfg(rcc_h50)]
        pll3_p: None,
        #[cfg(rcc_h50)]
        pll3_q: None,
        #[cfg(rcc_h50)]
        pll3_r: None,

        #[cfg(dsihost)]
        dsi_phy: None, // DSI PLL clock not supported, don't call `RccPeripheral::frequency()` in the drivers

        #[cfg(stm32h5)]
        audioclk: None,
        i2s_ckin: None,
        #[cfg(stm32h7rs)]
        spdifrx_symb: None, // TODO
        #[cfg(stm32h7rs)]
        clk48mohci: None, // TODO
        #[cfg(stm32h7rs)]
        usb: Some(Hertz(48_000_000)),
        #[cfg(stm32h5)]
        hse_div_rtcpre: None, // TODO
    );
}

struct PllInput {
    hsi: Option<Hertz>,
    hse: Option<Hertz>,
    csi: Option<Hertz>,
}

struct PllOutput {
    p: Option<Hertz>,
    #[allow(dead_code)]
    q: Option<Hertz>,
    #[allow(dead_code)]
    r: Option<Hertz>,
    #[cfg(stm32h7rs)]
    #[allow(dead_code)]
    s: Option<Hertz>,
    #[cfg(stm32h7rs)]
    #[allow(dead_code)]
    t: Option<Hertz>,
}

fn init_pll(num: usize, config: Option<Pll>, input: &PllInput) -> PllOutput {
    let Some(config) = config else {
        // Stop PLL
        RCC.cr().modify(|w| w.set_pllon(num, false));
        while RCC.cr().read().pllrdy(num) {}

        // "To save power when PLL1 is not used, the value of PLL1M must be set to 0.""
        #[cfg(any(stm32h7, stm32h7rs))]
        RCC.pllckselr().write(|w| w.set_divm(num, PllPreDiv::from_bits(0)));
        #[cfg(stm32h5)]
        RCC.pllcfgr(num).write(|w| w.set_divm(PllPreDiv::from_bits(0)));

        return PllOutput {
            p: None,
            q: None,
            r: None,
            #[cfg(stm32h7rs)]
            s: None,
            #[cfg(stm32h7rs)]
            t: None,
        };
    };

    let in_clk = match config.source {
        PllSource::DISABLE => panic!("must not set PllSource::Disable"),
        PllSource::HSI => unwrap!(input.hsi),
        PllSource::HSE => unwrap!(input.hse),
        PllSource::CSI => unwrap!(input.csi),
    };

    let ref_clk = in_clk / config.prediv as u32;

    let ref_range = match ref_clk.0 {
        ..=1_999_999 => Pllrge::RANGE1,
        ..=3_999_999 => Pllrge::RANGE2,
        ..=7_999_999 => Pllrge::RANGE4,
        ..=16_000_000 => Pllrge::RANGE8,
        x => panic!("pll ref_clk out of range: {} hz", x),
    };

    // The smaller range (150 to 420 MHz) must
    // be chosen when the reference clock frequency is lower than 2 MHz.
    let wide_allowed = ref_range != Pllrge::RANGE1;

    let vco_clk = ref_clk * config.mul;
    let vco_range = if VCO_RANGE.contains(&vco_clk) {
        Pllvcosel::MEDIUM_VCO
    } else if wide_allowed && VCO_WIDE_RANGE.contains(&vco_clk) {
        Pllvcosel::WIDE_VCO
    } else {
        panic!("pll vco_clk out of range: {}", vco_clk)
    };

    let p = config.divp.map(|div| {
        if num == 0 {
            // on PLL1, DIVP must be even for most series.
            // The enum value is 1 less than the divider, so check it's odd.
            #[cfg(not(any(pwr_h7rm0468, stm32h7rs)))]
            assert!(div.to_bits() % 2 == 1);
            #[cfg(pwr_h7rm0468)]
            assert!(div.to_bits() % 2 == 1 || div.to_bits() == 0);
        }

        vco_clk / div
    });
    let q = config.divq.map(|div| vco_clk / div);
    let r = config.divr.map(|div| vco_clk / div);
    #[cfg(stm32h7rs)]
    let s = config.divs.map(|div| vco_clk / div);
    #[cfg(stm32h7rs)]
    let t = config.divt.map(|div| vco_clk / div);

    #[cfg(stm32h5)]
    RCC.pllcfgr(num).write(|w| {
        w.set_pllsrc(config.source);
        w.set_divm(config.prediv);
        w.set_pllvcosel(vco_range);
        w.set_pllrge(ref_range);
        w.set_pllfracen(false);
        w.set_pllpen(p.is_some());
        w.set_pllqen(q.is_some());
        w.set_pllren(r.is_some());
    });

    #[cfg(any(stm32h7, stm32h7rs))]
    {
        RCC.pllckselr().modify(|w| {
            w.set_divm(num, config.prediv);
            w.set_pllsrc(config.source);
        });
        RCC.pllcfgr().modify(|w| {
            w.set_pllvcosel(num, vco_range);
            w.set_pllrge(num, ref_range);
            w.set_pllfracen(num, false);
            w.set_divpen(num, p.is_some());
            w.set_divqen(num, q.is_some());
            w.set_divren(num, r.is_some());
            #[cfg(stm32h7rs)]
            w.set_divsen(num, s.is_some());
            #[cfg(stm32h7rs)]
            w.set_divten(num, t.is_some());
        });
    }

    RCC.plldivr(num).write(|w| {
        w.set_plln(config.mul);
        w.set_pllp(config.divp.unwrap_or(PllDiv::DIV2));
        w.set_pllq(config.divq.unwrap_or(PllDiv::DIV2));
        w.set_pllr(config.divr.unwrap_or(PllDiv::DIV2));
    });

    #[cfg(stm32h7rs)]
    RCC.plldivr2(num).write(|w| {
        w.set_plls(config.divs.unwrap_or(Plldivst::DIV2));
        w.set_pllt(config.divt.unwrap_or(Plldivst::DIV2));
    });

    RCC.cr().modify(|w| w.set_pllon(num, true));
    while !RCC.cr().read().pllrdy(num) {}

    PllOutput {
        p,
        q,
        r,
        #[cfg(stm32h7rs)]
        s,
        #[cfg(stm32h7rs)]
        t,
    }
}

fn flash_setup(clk: Hertz, vos: VoltageScale) {
    // RM0481 Rev 1, table 37
    // LATENCY  WRHIGHFREQ  VOS3           VOS2            VOS1            VOS0
    //      0           0   0 to 20 MHz    0 to 30 MHz     0 to 34 MHz     0 to 42 MHz
    //      1           0   20 to 40 MHz   30 to 60 MHz    34 to 68 MHz    42 to 84 MHz
    //      2           1   40 to 60 MHz   60 to 90 MHz    68 to 102 MHz   84 to 126 MHz
    //      3           1   60 to 80 MHz   90 to 120 MHz   102 to 136 MHz  126 to 168 MHz
    //      4           2   80 to 100 MHz  120 to 150 MHz  136 to 170 MHz  168 to 210 MHz
    //      5           2                                  170 to 200 MHz  210 to 250 MHz
    #[cfg(stm32h5)]
    let (latency, wrhighfreq) = match (vos, clk.0) {
        (VoltageScale::Scale0, ..=42_000_000) => (0, 0),
        (VoltageScale::Scale0, ..=84_000_000) => (1, 0),
        (VoltageScale::Scale0, ..=126_000_000) => (2, 1),
        (VoltageScale::Scale0, ..=168_000_000) => (3, 1),
        (VoltageScale::Scale0, ..=210_000_000) => (4, 2),
        (VoltageScale::Scale0, ..=250_000_000) => (5, 2),

        (VoltageScale::Scale1, ..=34_000_000) => (0, 0),
        (VoltageScale::Scale1, ..=68_000_000) => (1, 0),
        (VoltageScale::Scale1, ..=102_000_000) => (2, 1),
        (VoltageScale::Scale1, ..=136_000_000) => (3, 1),
        (VoltageScale::Scale1, ..=170_000_000) => (4, 2),
        (VoltageScale::Scale1, ..=200_000_000) => (5, 2),

        (VoltageScale::Scale2, ..=30_000_000) => (0, 0),
        (VoltageScale::Scale2, ..=60_000_000) => (1, 0),
        (VoltageScale::Scale2, ..=90_000_000) => (2, 1),
        (VoltageScale::Scale2, ..=120_000_000) => (3, 1),
        (VoltageScale::Scale2, ..=150_000_000) => (4, 2),

        (VoltageScale::Scale3, ..=20_000_000) => (0, 0),
        (VoltageScale::Scale3, ..=40_000_000) => (1, 0),
        (VoltageScale::Scale3, ..=60_000_000) => (2, 1),
        (VoltageScale::Scale3, ..=80_000_000) => (3, 1),
        (VoltageScale::Scale3, ..=100_000_000) => (4, 2),

        _ => unreachable!(),
    };

    #[cfg(all(flash_h7, not(pwr_h7rm0468)))]
    let (latency, wrhighfreq) = match (vos, clk.0) {
        // VOS 0 range VCORE 1.26V - 1.40V
        (VoltageScale::Scale0, ..=70_000_000) => (0, 0),
        (VoltageScale::Scale0, ..=140_000_000) => (1, 1),
        (VoltageScale::Scale0, ..=185_000_000) => (2, 1),
        (VoltageScale::Scale0, ..=210_000_000) => (2, 2),
        (VoltageScale::Scale0, ..=225_000_000) => (3, 2),
        (VoltageScale::Scale0, ..=240_000_000) => (4, 2),
        // VOS 1 range VCORE 1.15V - 1.26V
        (VoltageScale::Scale1, ..=70_000_000) => (0, 0),
        (VoltageScale::Scale1, ..=140_000_000) => (1, 1),
        (VoltageScale::Scale1, ..=185_000_000) => (2, 1),
        (VoltageScale::Scale1, ..=210_000_000) => (2, 2),
        (VoltageScale::Scale1, ..=225_000_000) => (3, 2),
        // VOS 2 range VCORE 1.05V - 1.15V
        (VoltageScale::Scale2, ..=55_000_000) => (0, 0),
        (VoltageScale::Scale2, ..=110_000_000) => (1, 1),
        (VoltageScale::Scale2, ..=165_000_000) => (2, 1),
        (VoltageScale::Scale2, ..=224_000_000) => (3, 2),
        // VOS 3 range VCORE 0.95V - 1.05V
        (VoltageScale::Scale3, ..=45_000_000) => (0, 0),
        (VoltageScale::Scale3, ..=90_000_000) => (1, 1),
        (VoltageScale::Scale3, ..=135_000_000) => (2, 1),
        (VoltageScale::Scale3, ..=180_000_000) => (3, 2),
        (VoltageScale::Scale3, ..=224_000_000) => (4, 2),
        _ => unreachable!(),
    };

    // See RM0468 Rev 3 Table 16. FLASH recommended number of wait
    // states and programming delay
    #[cfg(all(flash_h7, pwr_h7rm0468))]
    let (latency, wrhighfreq) = match (vos, clk.0) {
        // VOS 0 range VCORE 1.26V - 1.40V
        (VoltageScale::Scale0, ..=70_000_000) => (0, 0),
        (VoltageScale::Scale0, ..=140_000_000) => (1, 1),
        (VoltageScale::Scale0, ..=210_000_000) => (2, 2),
        (VoltageScale::Scale0, ..=275_000_000) => (3, 3),
        // VOS 1 range VCORE 1.15V - 1.26V
        (VoltageScale::Scale1, ..=67_000_000) => (0, 0),
        (VoltageScale::Scale1, ..=133_000_000) => (1, 1),
        (VoltageScale::Scale1, ..=200_000_000) => (2, 2),
        // VOS 2 range VCORE 1.05V - 1.15V
        (VoltageScale::Scale2, ..=50_000_000) => (0, 0),
        (VoltageScale::Scale2, ..=100_000_000) => (1, 1),
        (VoltageScale::Scale2, ..=150_000_000) => (2, 2),
        // VOS 3 range VCORE 0.95V - 1.05V
        (VoltageScale::Scale3, ..=35_000_000) => (0, 0),
        (VoltageScale::Scale3, ..=70_000_000) => (1, 1),
        (VoltageScale::Scale3, ..=85_000_000) => (2, 2),
        _ => unreachable!(),
    };

    // See RM0455 Rev 10 Table 16. FLASH recommended number of wait
    // states and programming delay
    #[cfg(flash_h7ab)]
    let (latency, wrhighfreq) = match (vos, clk.0) {
        // VOS 0 range VCORE 1.25V - 1.35V
        (VoltageScale::Scale0, ..=42_000_000) => (0, 0),
        (VoltageScale::Scale0, ..=84_000_000) => (1, 0),
        (VoltageScale::Scale0, ..=126_000_000) => (2, 1),
        (VoltageScale::Scale0, ..=168_000_000) => (3, 1),
        (VoltageScale::Scale0, ..=210_000_000) => (4, 2),
        (VoltageScale::Scale0, ..=252_000_000) => (5, 2),
        (VoltageScale::Scale0, ..=280_000_000) => (6, 3),
        // VOS 1 range VCORE 1.15V - 1.25V
        (VoltageScale::Scale1, ..=38_000_000) => (0, 0),
        (VoltageScale::Scale1, ..=76_000_000) => (1, 0),
        (VoltageScale::Scale1, ..=114_000_000) => (2, 1),
        (VoltageScale::Scale1, ..=152_000_000) => (3, 1),
        (VoltageScale::Scale1, ..=190_000_000) => (4, 2),
        (VoltageScale::Scale1, ..=225_000_000) => (5, 2),
        // VOS 2 range VCORE 1.05V - 1.15V
        (VoltageScale::Scale2, ..=34) => (0, 0),
        (VoltageScale::Scale2, ..=68) => (1, 0),
        (VoltageScale::Scale2, ..=102) => (2, 1),
        (VoltageScale::Scale2, ..=136) => (3, 1),
        (VoltageScale::Scale2, ..=160) => (4, 2),
        // VOS 3 range VCORE 0.95V - 1.05V
        (VoltageScale::Scale3, ..=22) => (0, 0),
        (VoltageScale::Scale3, ..=44) => (1, 0),
        (VoltageScale::Scale3, ..=66) => (2, 1),
        (VoltageScale::Scale3, ..=88) => (3, 1),
        _ => unreachable!(),
    };
    #[cfg(flash_h7rs)]
    let (latency, wrhighfreq) = match (vos, clk.0) {
        // VOS high range VCORE 1.30V - 1.40V
        (VoltageScale::HIGH, ..=40_000_000) => (0, 0),
        (VoltageScale::HIGH, ..=80_000_000) => (1, 0),
        (VoltageScale::HIGH, ..=120_000_000) => (2, 1),
        (VoltageScale::HIGH, ..=160_000_000) => (3, 1),
        (VoltageScale::HIGH, ..=200_000_000) => (4, 2),
        (VoltageScale::HIGH, ..=240_000_000) => (5, 2),
        (VoltageScale::HIGH, ..=280_000_000) => (6, 3),
        (VoltageScale::HIGH, ..=320_000_000) => (7, 3),
        // VOS low range VCORE 1.15V - 1.26V
        (VoltageScale::LOW, ..=36_000_000) => (0, 0),
        (VoltageScale::LOW, ..=72_000_000) => (1, 0),
        (VoltageScale::LOW, ..=108_000_000) => (2, 1),
        (VoltageScale::LOW, ..=144_000_000) => (3, 1),
        (VoltageScale::LOW, ..=180_000_000) => (4, 2),
        (VoltageScale::LOW, ..=216_000_000) => (5, 2),
        _ => unreachable!(),
    };

    debug!("flash: latency={} wrhighfreq={}", latency, wrhighfreq);

    FLASH.acr().write(|w| {
        w.set_wrhighfreq(wrhighfreq);
        w.set_latency(latency);
    });
    while FLASH.acr().read().latency() != latency {}
}
