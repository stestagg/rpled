# RPLed

An addressable LED firmware for RP2XXX microcontrollers, written in Rust.

## Features

- Support for a range of standard addressable LED protocols
- A bytecode interpreter for efficient LED control and animations
- Modular design for easy extension and customization
- A common command protocol for system control linked to a configurable set of:
  - HTTP API
  - Raw Socket API
  - WebSocket API
  - HTTP UI Frontend
- Uses PIO to drive up to 8 LED strips in parallel (depending on # free PIO state machines)
- Supports multiple LED strips with different protocols simultaneously

## Architecture

RPLed splits execution across both cores:
 - 1 core handles the LED driving using PIO and DMA, including the interpreter and LED refresh scheduling, receiving commands over a channel
 - The other core handles all IO, networking, command protocols, configuration etc.

All execution is coordinated and run using the embassy framework.

## Code Organization

- `rpled-vm`: The virtual machine implementation that runs the bytecode interpreter.
- `rpled-core`: Core library that implements the main loop of the LED control task, running the VM, LED protocol implementations, common command protocol, DMA & PIO programs, and other shared functionality.
- `rpled-cyw43`: WiFi and networking stack implementation for the CYW43 chip.
   - Features for HTTP and raw socket servers.
- `rpled-compile`: A compiler that translates high-level pixelscript control scripts into bytecode for the RPLed interpreter.
- `rpled-sim`: A simulator that can run pixelscript and bytecode on a host machine for testing and development, visualizing LED output.
- `rpled-fw`: A binary crate that boots the RP, configures the core and non-core modules on the correct core, and starts the main event loop.

## LEDScript

A subset of lua with built-in functions that provide high level interfaces over the bytecode commands.

## Interpreter

The interpreter runs a simple 16-bit virtual machine executing a trivial command set

The VM is stack based, with a flat N KB memory space partitioned into:
(N = 4KB/8KB/16KB depending on configuration)
 - Program/Data space (sized to fit the loaded bytecode)
 - Heap space (configured by program header)
 - Stack space (the rest).

A Program Counter (PC) tracks the current instruction, and a Stack Pointer (SP) tracks the top of the stack.

Stack overflows or underflows cause immediate program termination.

Unless otherwise specified, all arithmetic is performed using 16-bit signed integers with wraparound on overflow.

Boolean values are represented as 0 (false) and 1 (true).

The VM is extensible via modules.  Each module globally reserves 4 opcodes in the opcode space for performing calls with varying numbers of arguments.  All arguments are 16-bit values whose interpretation is module-specific.

For example, a module named LED causes 4 opcodes to be reserved:
 - LED0 c
 - LED1 c
 - LED2 c
 - LEDN n c

Where `c` is a built-in function code, and `n` is the number of arguments to pop from the stack.

In this way, the pixelscript call:
   led.clear()
Compiles to the bytecode instruction:
   LED0 CLEAR_CODE
And the call:
   led.set_pixel(x, r, g, b)
Compiles to:
    PUSH b
    PUSH g
    PUSH r
    PUSH x
    LEDN 4 SET_PIXEL_CODE

## Command Set

Commands use the following notation:
<OPCODE> [ARGUMENTS]
Where:
 - `u8` = 8-bit unsigned integer constant
 - `i16` = 16-bit signed integer constant
   `a` = 16-bit constant unsigned address in entire memory space
 - `addr` = 16-bit signed integer, relative address offset in program space (relative to the next instruction: PC + 1)
 - `c` = 8-bit Module function code
Comment conventions:
 - `push(<...>)` = push value onto stack
 - `pop()`/`pop(n)` = pop value(s) from stack
 - `s[n]` = stack value at index n (0 = top)
 - `s[n1,n2,n3]` = multiple stack values at indices n1, n2, n3
 - `mem[a]` = memory at address a (entire memory space)

|  # | OP Spec     | Mini pseudocode                | Description                    |
| -: | ----------- | ------------------------------ | ------------------------------ |
|  1 | PUSH i16    | `push(i16)`                    | Push constant value            |
|  2 | LOAD a      | `push(mem[a])`                 | Push value from memory         |
|  3 | STORE a     | `mem[a] = pop()`               | Store top of stack into memory |
|  4 | POP         | `pop()`/`sp -= 1`              | Discard top value              |
|  5 | POPN u8     | `pop(u8)`/`sp -= u8`           | Discard u8 values             |
|  6 | DUP         | `push(s[0])`                   | Duplicate top of stack         |
|  7 | SWAP        | `swap(s[0], s[1])`             | Swap top two values            |
|  8 | OVER        | `push(s[1])`                   | Copy second value to top       |
|  9 | ROT         | `(s[2, 1, 0]) -> (s[1, 0, 2])` | Rotate top three values        |
| 10 | ZERO        | `push(0)`                      | Push zero                      |
| 11 | ADD         | `push(s[1] + s[0])`            | Addition                       |
| 12 | SUB         | `push(s[1] - s[0])`            | Subtraction                    |
| 13 | MUL         | `push(s[1] * s[0])`            | Multiplication                 |
| 14 | DIV         | `push(s[1] / s[0])`            | Division                       |
| 15 | MOD         | `push(s[1] % s[0])`            | Modulo                         |
| 16 | EQ          | `push(s[1] == s[0])`           | Equality test                  |
| 17 | NE          | `push(s[1] != s[0])`           | Inequality test                |
| 18 | LT          | `push(s[1] < s[0])`            | Less than                      |
| 19 | GT          | `push(s[1] > s[0])`            | Greater than                   |
| 20 | LE          | `push(s[1] <= s[0])`           | Less or equal                  |
| 21 | GE          | `push(s[1] >= s[0])`           | Greater or equal               |
| 22 | AND         | `push(s[1] & s[0])`            | Bitwise AND                    |
| 23 | OR          | `push(s[1] | s[0])`            | Bitwise OR                     |
| 24 | XOR         | `push(s[1] ^ s[0])`            | Bitwise XOR                    |
| 25 | NOT         | `push(!s[0])`                  | Logical NOT                    |
| 26 | INC         | `push(s[0] + 1)`               | Increment                      |
| 27 | DEC         | `push(s[0] - 1)`               | Decrement                      |
| 28 | NEG         | `push(-s[0])`                  | Negate                         |
| 29 | ABS         | `push(abs(s[0]))`              | Absolute value                 |
| 30 | CLAMP       | `push(min(max(s[2], s[1]), s[0]))` | Clamp value between s[1] and s[0] |
| 31 | JMP addr    | `pc += addr`                   | Unconditional jump (relative)  |
| 32 | JZ addr     | `if(s[0]==0) pc+=addr`         | Jump if zero                   |
| 33 | JNZ addr    | `if(s[0]!=0) pc+=addr`         | Jump if non-zero               |
| 34 | CALL addr   | `push(ret); pc+=addr`          | Call subroutine                |
| 35 | CALLZ addr  | `if(s[0]==0) call`             | Conditional call if zero       |
| 36 | CALLNZ addr | `if(s[0]!=0) call`             | Conditional call if non-zero   |
| 37 | RET         | `pc = pop()`                   | Return from subroutine         |
| 38 | HALT        | `stop`                         | Stop execution                 |
| 39 | SLEEP       | `delay(pop())`                 | Sleep for s[0] microseconds    |
| -- | ----------- | ------------------------------ | ------------------------------ |
|    | LED MODULE                                                                    |
| -- | ----------- | ------------------------------ | ------------------------------ |
| 64 | LED0 c      | `led(c)`                       | LED call with 0 args           |
| 65 | LED1 c      | `led(c,pop())`                 | LED call with 1 arg (s[0])     |
| 66 | LED2 c      | `led(c,pop(),pop())`           | LED call with 2 args (s[0], s[1])  |
| 67 | LEDN c u8   | `led(c,pop(), ...u8)`          | LED call with `u8` stack values (each i16)   |

* Note about module calling conventions, because stack pushes are last-in-first-out (LIFO), arguments
have to be pushed in reverse order. *

## Pixelscript

Pixelscript is a name given to a subset of lua that compiles to RPLed bytecode using the rpled-compile tool.

### Example

```lua
pixelscript = {
    name = "Blinky"
    modules = {"LED"}
    entrypoint = "main"
    params = {
        SPEED = RANGE(1, 100, 50) -- Speed parameter from 1 to 100, default 50
    }
}

function main()
    -- Get the number of pixels
    local num_pixels = led.get_num_pixels()
    -- calculate the delay based on SPEED param
    local delay = 1000 / SPEED  -- in milliseconds
    local middle = num_pixels / 2

    while true do
        led.clear()
        -- sleep in microseconds
        sleep(delay * 1000)  
        led.set_pixel(middle, 255, 0, 0)  -- Set middle pixel to red
        sleep(delay * 1000)
        led.set_pixel(middle, 0, 0, 0)    -- Turn off middle pixel
        sleep(delay * 1000)
        led.fill(0, num_pixels - 1, 0, 0, 255)  -- Fill all pixels with blue
        sleep(delay * 1000)
    end
end
```


### Limitations compared to full Lua

* No tables or complex data structures (only scalars)
* No coroutines, generators, or yielding
* Limited standard library (only VM modules + essentials)
* No nested functions and no closures (no captured outer locals)
* No dynamic code loading (`load`, `dofile`, `eval`)
* No metatables or operator overloading
* No reflection / introspection APIs

## Program Structure

RPLed bytecode programs have a simple header followed by the program instructions and data.

| Offset | Size  | Description                          |
| ------ | ----- | ------------------------------------ |
| 0      | 3     | PXS                                  |
| 3      | 1     | Version (currently 0)                |
| 4      | 2     | Heap size                            |
| 6      | 1     | Remaining Header Length              |
| 7      | 1     | Number of modules (n_mod)            |
| 8      | n_mod | [Module id, ...]                     |
| 8+n_mod| to header_length | Program name (null-terminated string) |