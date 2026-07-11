# rusToniESP – Architecture

> **Reference project:** ESPuino
>
> rusToniESP is **not** a line-by-line Rust port of ESPuino. The goal is to build a clean, idiomatic Rust implementation that provides similar functionality while following modern Rust design principles.

---

# Design Principles

The architecture should evolve as the project grows, but every contribution should follow these principles.

## Domain before hardware

Application logic should be independent from ESP32 peripherals whenever possible.

Business concepts such as playlists, playback, RFID actions or settings should not depend directly on GPIOs, SPI, I2S or ESP-IDF APIs.

Hardware exists to support the application, not define it.

---

## Composition over conditional logic

New hardware variants should be added by implementing traits or creating new drivers, not by adding more `if`, `match`, or `cfg` branches throughout the codebase.

Good:

```
trait RfidReader
trait AudioOutput
trait LedDriver
```

Avoid:

```
if pn5180 { ... }
else if mfrc522 { ... }
```

---

## Small, focused modules

Each module should have a single responsibility.

Examples:

- audio
- rfid
- storage
- ui
- connectivity
- power

Avoid large "utility" or "misc" modules that accumulate unrelated code.

---

## Prefer abstractions around capabilities

Traits should describe **what** a component does, not the hardware implementing it.

For example:

- `RfidReader`
- `AudioOutput`
- `LedDriver`

instead of

- `Pn5180`
- `Max98357Driver`

The application should depend on capabilities, while board-specific code chooses the implementation.

---

## Separate policy from mechanism

Drivers interact with hardware.

Services implement application behaviour.

For example:

```
RFID reader
    ↓
Tag detected
    ↓
Tag service
    ↓
Player service
    ↓
Audio output
```

Drivers should not contain application logic.

---

## Message-driven communication

Modules should communicate through commands, events or messages instead of directly calling each other whenever practical.

Avoid creating a single giant global event enum shared by every module.

Instead, prefer small, focused message types owned by the relevant subsystem.

---

## Keep dependencies one-way

High-level code should never depend on hardware details.

Preferred dependency direction:

```
Application
    ↓
Services
    ↓
Drivers
    ↓
ESP-IDF / Hardware
```

Never the other way around.

---

# Suggested Project Layout

The exact layout may evolve, but the project should remain organized around responsibilities rather than ESPuino source files.

```
src/

main.rs

app/
    application wiring
    startup
    dependency composition

domain/
    pure business types
    playlists
    tracks
    settings
    actions

services/
    playback
    RFID handling
    media library
    configuration

drivers/
    audio
    rfid
    storage
    display

connectivity/
    wifi
    web
    mqtt
    ftp

board/
    board configuration
    pin mappings
    peripheral initialization

util/
    shared helpers
```

Not every directory needs to exist immediately.

Only introduce new modules when they have a clear responsibility.

---

# Board Layer

The board module owns hardware composition.

Its responsibilities include:

- pin assignments
- peripheral initialization
- selecting driver implementations

The rest of the application should not care whether a board uses:

- PN5180 or MFRC522
- SDMMC or SPI SD
- different audio amplifiers
- different LED drivers

---

# Concurrency

Concurrency is an implementation detail.

The architecture should not assume one thread or task per module.

Different features may share execution contexts if that simplifies the design.

Communication between concurrent components should happen through typed messages rather than shared mutable state whenever practical.

---

# Cargo Features

Cargo features should represent optional capabilities.

Examples:

- bluetooth
- mqtt
- ftp
- pn5180
- ir_remote

Features should enable or disable modules, not scatter conditional compilation throughout the codebase.

---

# Testing

Whenever possible:

- keep business logic testable on the host
- isolate ESP-IDF-specific code inside drivers
- avoid mixing hardware access with application logic

The more code that can be tested without an ESP32, the better.

---

# Relationship with ESPuino

ESPuino is the functional reference, not the architectural reference.

When implementing a feature:

- preserve behaviour where it makes sense
- redesign the implementation if Rust offers a cleaner approach
- do not mirror the original class hierarchy or file structure without a good reason

---

# Guidelines for Contributors

Before introducing new code, ask:

- Does this module have a single responsibility?
- Can this logic be independent from ESP-IDF?
- Should this be a service instead of a driver?
- Can this hardware be abstracted behind an existing trait?
- Am I adding complexity because of the original ESPuino design, or because the Rust implementation genuinely needs it?

When in doubt, prefer simpler, more modular designs over strict compatibility with the original codebase.

This document is intentionally high-level. The architecture is expected to evolve as the project grows, but these principles should remain stable.