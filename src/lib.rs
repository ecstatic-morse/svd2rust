//! Peripheral API generator from [CMSIS-SVD] files
//!
//! [CMSIS-SVD]: http://www.keil.com/pack/doc/CMSIS/SVD/html/index.html
//!
//! A SVD file is an XML file that describes the hardware features of a
//! microcontroller. In particular, it list all the peripherals available to the
//! device, where the registers associated to each device are located in memory
//! and what's the function of each register.
//!
//! `svd2rust` is a command line tool that transforms SVD files into crates that
//! expose a type safe API to access the peripherals of the device.
//!
//! # Installation
//!
//! ```
//! $ cargo install svd2rust
//! ```
//!
//! # Usage
//!
//! `svd2rust` supports Cortex-M and MSP430 microcontrollers. The generated
//! crate can be tailored for either architecture using the `--target` flag. The
//! flag accepts "cortex-m", "msp430" and "none" as values. "none" can be used
//! to generate a crate that's architecture agnostic and that should work for
//! architectures that `svd2rust` doesn't currently know about like the Cortex-A
//! architecture.
//!
//! If the `--target` flag is omitted `svd2rust` assumes the target is the
//! Cortex-M architecture.
//!
//! ```
//! $ svd2rust -i STM32F30x.svd | rustfmt | tee src/lib.rs
//! //! Peripheral access API for STM32F30X microcontrollers
//! //! (generated using svd2rust v0.12.0)
//!
//! #![deny(missing_docs)]
//! #![deny(warnings)]
//! #![no_std]
//!
//! extern crate bare_metal;
//! extern crate cortex_m;
//! #[cfg(feature = "rt")]
//! extern crate cortex_m_rt;
//! extern crate vcell;
//!
//! use cortex_m::peripheral::Peripheral;
//!
//! /// Interrupts
//! pub mod interrupt {
//!     // ..
//! }
//!
//! /// General-purpose I/Os
//! pub mod gpioa {
//!     pub struct RegisterBlock {
//!         /// GPIO port mode register
//!         pub moder: MODER,
//!         ..
//!     }
//!     ..
//! }
//!
//! /// General-purpose I/Os
//! pub struct GPIOA { _marker: PhantomData<*const ()> }
//!
//! unsafe impl Send for GPIOA {}
//!
//! impl GPIOA {
//!     /// Returns a pointer to the register block
//!     pub fn ptr() -> *const gpioa::RegisterBlock {
//!         0x4800_0000 as *const _
//!     }
//! }
//!
//! impl core::ops::Deref for GPIOA {
//!     type Target = gpioa::RegisterBlock;
//!
//!     fn deref(&self) -> &gpioa::RegisterBlock {
//!         unsafe { &*GPIOA::ptr() }
//!     }
//! }
//!
//! // ..
//! ```
//!
//! # Dependencies
//!
//! The generated crate depends on:
//!
//! - [`bare-metal`](https://crates.io/crates/bare-metal) v0.1.x
//! - [`vcell`](https://crates.io/crates/vcell) v0.1.x
//! - [`cortex-m-rt`](https://crates.io/crates/cortex-m-rt) v0.3.x if targeting
//!   the Cortex-M architecture.
//! - [`cortex-m`](https://crates.io/crates/cortex-m) v0.4.x if targeting the
//!   Cortex-M architecture.
//! - [`msp430`](https://crates.io/crates/msp430) v0.1.x if targeting the MSP430
//!   architecture.
//! - [`msp430-rt`](https://crates.io/crates/msp430-rt) v0.1.x if targeting the
//!   MSP430 architecture.
//!
//! # Peripheral API
//!
//! To use a peripheral first you must get an *instance* of the peripheral. All the device
//! peripherals are modeled as singletons (there can only ever be, at most, one instance of any
//! one of them) and the only way to get an instance of them is through the `Peripherals::take`
//! method.
//!
//! ```
//! fn main() {
//!     let mut peripherals = stm32f30x::Peripherals::take().unwrap();
//!     peripherals.GPIOA.odr.write(|w| w.bits(1));
//! }
//! ```
//!
//! This method can only be successfully called *once* -- that's why the method returns an `Option`.
//! Subsequent calls to the method will result in a `None` value being returned.
//!
//! ```
//! fn main() {
//!     let ok = stm32f30x::Peripherals::take().unwrap();
//!     let panics = stm32f30x::Peripherals::take().unwrap();
//! }
//! ```
//!
//! The singleton property can be *unsafely* bypassed using the `ptr` static method which is
//! available on all the peripheral types. This method is a useful for implementing safe higher
//! level abstractions.
//!
//! ```
//! struct PA0 { _0: () }
//! impl PA0 {
//!     fn is_high(&self) -> bool {
//!         // NOTE(unsafe) actually safe because this is an atomic read with no side effects
//!         unsafe { (*GPIOA::ptr()).idr.read().bits() & 1 != 0 }
//!     }
//!
//!     fn is_low(&self) -> bool {
//!         !self.is_high()
//!     }
//! }
//! struct PA1 { _0: () }
//! // ..
//!
//! fn configure(gpioa: GPIOA) -> (PA0, PA1, ..) {
//!     // configure all the PAx pins as inputs
//!     gpioa.moder.reset();
//!     // the GPIOA proxy is destroyed here now the GPIOA register block can't be modified
//!     // thus the configuration of the PAx pins is now frozen
//!     drop(gpioa);
//!     (PA0 { _0: () }, PA1 { _0: () }, ..)
//! }
//! ```
//!
//! Each peripheral proxy `deref`s to a `RegisterBlock` struct that represents a piece of device
//! memory. Each field in this `struct` represents one register in the register block associated to
//! the peripheral.
//!
//! ```
//! /// Inter-integrated circuit
//! pub mod i2c1 {
//!     /// Register block
//!     pub struct RegisterBlock {
//!         /// 0x00 - Control register 1
//!         pub cr1: CR1,
//!         /// 0x04 - Control register 2
//!         pub cr2: CR2,
//!         /// 0x08 - Own address register 1
//!         pub oar1: OAR1,
//!         /// 0x0c - Own address register 2
//!         pub oar2: OAR2,
//!         /// 0x10 - Timing register
//!         pub timingr: TIMINGR,
//!         /// Status register 1
//!         pub timeoutr: TIMEOUTR,
//!         /// Interrupt and Status register
//!         pub isr: ISR,
//!         /// 0x1c - Interrupt clear register
//!         pub icr: ICR,
//!         /// 0x20 - PEC register
//!         pub pecr: PECR,
//!         /// 0x24 - Receive data register
//!         pub rxdr: RXDR,
//!         /// 0x28 - Transmit data register
//!         pub txdr: TXDR,
//!     }
//! }
//! ```
//!
//! # `read` / `modify` / `write` API
//!
//! Each register in the register block, e.g. the `cr1` field in the `I2C` struct, exposes a
//! combination of the `read`, `modify` and `write` methods. Which methods exposes each register
//! depends on whether the register is read-only, read-write or write-only:
//!
//! - read-only registers only expose the `read` method.
//! - write-only registers only expose the `write` method.
//! - read-write registers expose all the methods: `read`, `modify` and
//!   `write`.
//!
//! This is signature of each of these methods:
//!
//! (using `I2C`'s `CR2` register as an example)
//!
//! ``` rust
//! impl CR2 {
//!     /// Modifies the contents of the register
//!     pub fn modify<F>(&self, f: F)
//!     where
//!         for<'w> F: FnOnce(&R, &'w mut W) -> &'w mut W
//!     {
//!         ..
//!     }
//!
//!     /// Reads the contents of the register
//!     pub fn read(&self) -> R { .. }
//!
//!     /// Writes to the register
//!     pub fn write<F>(&mut self, f: F)
//!     where
//!         F: FnOnce(&mut W) -> &mut W,
//!     {
//!         ..
//!     }
//! }
//! ```
//!
//! The `read` method "reads" the register using a **single**, volatile `LDR` instruction and
//! returns a proxy `R` struct that allows access to only the readable bits (i.e. not to the
//! reserved or write-only bits) of the `CR2` register:
//!
//! ``` rust
//! /// Value read from the register
//! impl R {
//!     /// Bit 0 - Slave address bit 0 (master mode)
//!     pub fn sadd0(&self) -> SADD0R { .. }
//!
//!     /// Bits 1:7 - Slave address bit 7:1 (master mode)
//!     pub fn sadd1(&self) -> SADD1R { .. }
//!
//!     (..)
//! }
//! ```
//!
//! Usage looks like this:
//!
//! ``` rust
//! // is the SADD0 bit of the CR2 register set?
//! if i2c1.c2r.read().sadd0().bit() {
//!     // yes
//! } else {
//!     // no
//! }
//! ```
//!
//! On the other hand, the `write` method writes some value to the register using a **single**,
//! volatile `STR` instruction. This method involves a `W` struct that only allows constructing
//! valid states of the `CR2` register.
//!
//! The only constructor that `W` provides is `reset_value` which returns the value of the `CR2`
//! register after a reset. The rest of `W` methods are "builder-like" and can be used to modify the
//! writable bitfields of the `CR2` register.
//!
//! ``` rust
//! impl CR2W {
//!     /// Reset value
//!     pub fn reset_value() -> Self {
//!         CR2W { bits: 0 }
//!     }
//!
//!     /// Bits 1:7 - Slave address bit 7:1 (master mode)
//!     pub fn sadd1(&mut self) -> _SADD1W { .. }
//!
//!     /// Bit 0 - Slave address bit 0 (master mode)
//!     pub fn sadd0(&mut self) -> SADD0R { .. }
//! }
//! ```
//!
//! The `write` method takes a closure with signature `(&mut W) -> &mut W`. If the "identity
//! closure", `|w| w`, is passed then the `write` method will set the `CR2` register to its reset
//! value. Otherwise, the closure specifies how the reset value will be modified *before* it's
//! written to `CR2`.
//!
//! Usage looks like this:
//!
//! ``` rust
//! // Starting from the reset value, `0x0000_0000`, change the bitfields SADD0
//! // and SADD1 to `1` and `0b0011110` respectively and write that to the
//! // register CR2.
//! i2c1.cr2.write(|w| unsafe { w.sadd0().bit(true).sadd1().bits(0b0011110) });
//! // NOTE ^ unsafe because you could be writing a reserved bit pattern into
//! // the register. In this case, the SVD doesn't provide enough information to
//! // check whether that's the case.
//!
//! // NOTE The argument to `bits` will be *masked* before writing it to the
//! // bitfield. This makes it impossible to write, for example, `6` to a 2-bit
//! // field; instead, `6 & 3` (i.e. `2`) will be written to the bitfield.
//! ```
//!
//! Finally, the `modify` method performs a **single** read-modify-write
//! operation that involves **one** read (`LDR`) to the register, modifying the
//! value and then a **single** write (`STR`) of the modified value to the
//! register. This method accepts a closure that specifies how the CR2 register
//! will be modified (the `w` argument) and also provides access to the state of
//! the register before it's modified (the `r` argument).
//!
//! Usage looks like this:
//!
//! ``` rust
//! // Set the START bit to 1 while KEEPING the state of the other bits intact
//! i2c1.cr2.modify(|_, w| unsafe { w.start().bit(true) });
//!
//! // TOGGLE the STOP bit, all the other bits will remain untouched
//! i2c1.cr2.modify(|r, w| w.stop().bit(!r.stop().bit()));
//! ```
//!
//! # enumeratedValues
//!
//! If your SVD uses the `<enumeratedValues>` feature, then the API will be *extended* to provide
//! even more type safety. This extension is backward compatible with the original version so you
//! could "upgrade" your SVD by adding, yourself, `<enumeratedValues>` to it and then use `svd2rust`
//! to re-generate a better API that doesn't break the existing code that uses that API.
//!
//! The new `read` API returns an enum that you can match:
//!
//! ```
//! match gpioa.dir.read().pin0() {
//!     gpioa::dir::PIN0R::Input => { .. },
//!     gpioa::dir::PIN0R::Output => { .. },
//! }
//! ```
//!
//! or test for equality
//!
//! ```
//! if gpioa.dir.read().pin0() == gpio::dir::PIN0R::Input {
//!     ..
//! }
//! ```
//!
//! It also provides convenience methods to check for a specific variant without
//! having to import the enum:
//!
//! ```
//! if gpioa.dir.read().pin0().is_input() {
//!     ..
//! }
//!
//! if gpioa.dir.read().pin0().is_output() {
//!     ..
//! }
//! ```
//!
//! The original `bits` method is available as well:
//!
//! ```
//! if gpioa.dir.read().pin0().bits() == 0 {
//!     ..
//! }
//! ```
//!
//! And the new `write` API provides similar additions as well: `variant` lets you pick the value to
//! write from an `enum`eration of the possible ones:
//!
//! ```
//! // enum DIRW { Input, Output }
//! gpioa.dir.write(|w| w.pin0().variant(gpio::dir::PIN0W::Output));
//! ```
//!
//! There are convenience methods to pick one of the variants without having to
//! import the enum:
//!
//! ```
//! gpioa.dir.write(|w| w.pin0().output());
//! ```
//!
//! The `bits` (or `bit`) method is still available but will become safe if it's
//! impossible to write a reserved bit pattern into the register:
//!
//! ```
//! // safe because there are only two options: `0` or `1`
//! gpioa.dir.write(|w| w.pin0().bit(true));
//! ```
//!
//! # Interrupt API
//!
//! SVD files also describe the device interrupts. svd2rust generated crates expose an enumeration
//! of the device interrupts as an `Interrupt` `enum` in the root of the crate. This `enum` can be
//! used with the `cortex-m` crate `NVIC` API.
//!
//! ```
//! extern crate cortex_m;
//! extern crate stm32f30x;
//!
//! use cortex_m::interrupt;
//! use cortex_m::peripheral::Peripherals;
//! use stm32f30x::Interrupt;
//!
//! let p = Peripherals::take().unwrap();
//! let mut nvic = p.NVIC;
//!
//! nvic.enable(Interrupt::TIM2);
//! nvic.enable(Interrupt::TIM3);
//! ```
//!
//! ## the "rt" feature
//!
//! If the "rt" Cargo feature of the svd2rust generated crate is enabled the crate will populate the
//! part of the vector table that contains the interrupt vectors and provide an
//! [`interrupt!`](macro.interrupt.html) macro that can be used to register interrupt handlers.

// NOTE This file is for documentation only

/// Assigns a handler to an interrupt
///
/// This macro takes two arguments: the name of an interrupt and the path to the
/// function that will be used as the handler of that interrupt. That function
/// must have signature `fn()`.
///
/// Optionally, a third argument may be used to declare interrupt local data.
/// The handler will have exclusive access to these *local* variables on each
/// invocation. If the third argument is used then the signature of the handler
/// function must be `fn(&mut $NAME::Locals)` where `$NAME` is the first argument
/// passed to the macro.
///
/// # Example
///
/// ``` ignore
/// interrupt!(TIM2, periodic);
///
/// fn periodic() {
///     print!(".");
/// }
///
/// interrupt!(TIM3, tick, locals: {
///     tick: bool = false;
/// });
///
/// fn tick(locals: &mut TIM3::Locals) {
///     locals.tick = !locals.tick;
///
///     if locals.tick {
///         println!("Tick");
///     } else {
///         println!("Tock");
///     }
/// }
/// ```
#[macro_export]
macro_rules! interrupt {
    ($NAME:ident, $path:path) => {};
    ($NAME:ident, $path:path, locals: {
        $($lvar:ident: $lty:ty = $lval:expr;)+
    }) => {};
}
