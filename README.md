# Dice Roller

A comprehensive CLI dice roller built in Rust, supporting Roll20-style dice notation with extensive modifiers and operations.

## Features

### Dice Types
- **Standard Dice**: `NdX` (e.g., `3d6`, `1d20`)
- **FATE Dice**: `NdF` (e.g., `4dF`)
- **Daggerheart Duality Dice**: `dD` (Hope d12 + Fear d12, see [Daggerheart Rolls](#daggerheart-rolls))
- **Computed Dice**: `(2+3)d6`, `3d(4+2)`

### Roll Modifiers
- **Exploding Dice**: `3d6!` (explode on max), `3d6!>5` (explode on 5+)
- **Compounding**: `5d6!!` (Shadowrun-style)
- **Penetrating**: `5d6!p` (HackMaster-style, -1 on explosions)
- **Keep/Drop**:
  - `4d6kh3` - Keep highest 3
  - `4d6kl2` - Keep lowest 2
  - `8d6dh2` - Drop highest 2
  - `8d6dl1` - Drop lowest 1
- **Rerolling**:
  - `2d10r<2` - Reroll on 2 or less (unlimited)
  - `2d10ro<2` - Reroll once on 2 or less
- **Success Counting**: `10d6>5` (count successes ≥5)
- **Failures**: `10d6>5f1` (count failures on 1)
- **Critical Markers**: `1d20cs>19cf<2` (mark crits/fumbles)
- **Sorting**: `8d6sa` (ascending), `8d6sd` (descending)

### Mathematical Operations
- **Operators**: `+`, `-`, `*`, `/`, `%`, `**` (exponent)
- **Functions**: `floor()`, `ceil()`, `round()`, `abs()`
- **Parentheses**: Standard order of operations

### Advanced Features
- **Grouped Rolls**: `{4d6, 3d8}kh1`
- **Complex Expressions**: `2d10+5[Fire] + 3d6[Ice]`
- **Multiple Modifiers**: `4d6!kh3` (exploding, then keep highest 3)

## Installation

### Using Nix Flakes (Recommended)
```bash
# Enter development environment
nix develop

# Build with Nix (release build by default)
nix build

# Build debug version (with debug symbols)
nix build .#debug

# Build release explicitly
nix build .#release

# Run directly
nix run . -- "1d20+5"
```

### Using Cargo
```bash
# Build
cargo build --release

# Run
cargo run --release -- "1d20+5"

# Install
cargo install --path .
```

## Usage

### Basic Rolling
```bash
# Simple roll
dice_roller "1d20+5"

# Keep highest
dice_roller "4d6kh3"

# Exploding dice
dice_roller "3d6!"

# Success counting
dice_roller "10d6>5"

# FATE dice
dice_roller "4dF"

# Daggerheart Duality Dice
dice_roller "dD+2"
```

### Options
```bash
# Verbose output with details
dice_roller --verbose "4d6kh3"

# Quiet mode (only show total)
dice_roller --quiet "1d20+5"

# Repeat roll N times
dice_roller -n 10 "1d20"

# Deterministic rolls with seed
dice_roller --seed 42 "1d20"

# Disable colors
dice_roller --no-color "1d20"
```

### Interactive Mode
```bash
dice_roller

> 1d20+5
Roll: 1d20+5 = 18

> 4d6kh3
Roll: 4d6kh3 = 14

> help
[Shows help with examples]

> exit
Goodbye!
```

## Examples

### Character Creation (D&D)
```bash
# Roll 6 sets of 4d6, keep highest 3
dice_roller -n 6 "4d6kh3"
```

### Attack Roll with Advantage
```bash
dice_roller "{2d20}kh1+5"
```

### Shadowrun Dice Pool
```bash
dice_roller "8d6!!>5"
```

### Exploding Damage
```bash
dice_roller "2d6!+1d4"
```

### Wild Die (Savage Worlds)
```bash
dice_roller "{1d8,1d6}kh1"
```

## Daggerheart Rolls

[Daggerheart](https://www.daggerheart.com/) uses a pair of d12s called
**Duality Dice**: one Hope die and one Fear die. Roll them with `dD`.

An action roll is the Duality Dice plus your trait and any other modifiers:

```bash
dice_roller "dD+2"
# Roll: dD+2 = 14 with Hope (Hope 8, Fear 4)
```

The result shows the total and which die rolled higher:

| Result | Meaning |
|---|---|
| `with Hope` | The Hope die is higher. You gain a Hope. |
| `with Fear` | The Fear die is higher. The GM gains a Fear. |
| `Critical Success` | Both dice show the same number. The roll succeeds regardless of the total. You gain a Hope and clear a Stress; on an attack roll you deal extra damage. |

There is no Difficulty in the formula. Tell the GM your total and the
outcome ("14 with Hope"), and they decide whether it succeeds.

### Modifiers, Advantage and Disadvantage

Modifiers are plain numbers, and labels help you remember what each one is:

```bash
# Agility +2 and an Experience +2
dice_roller "dD+2[Agility]+2[Experience]"
```

Advantage adds a d6, disadvantage subtracts a d6. They cancel each other out,
so you only ever roll one of them:

```bash
# With advantage
dice_roller "dD+2[Agility]+1d6[Advantage]"
# Roll: dD+2[Agility]+1d6[Advantage] = 26 with Hope (Hope 12, Fear 9)

# With disadvantage
dice_roller "dD+2[Agility]-1d6[Disadvantage]"
```

### Reaction Rolls

Reaction rolls use the same formula (`dD+2`). They don't generate Hope or
Fear, so just ignore the "with Hope/Fear" part of the result. A critical
success on a reaction roll doesn't give Hope or clear Stress either.

## Development

### Project Structure
```
dice_roller/
├── SPEC.md              # Detailed specification
├── src/
│   ├── lib.rs           # Public API
│   ├── main.rs          # CLI interface
│   ├── ast.rs           # Abstract Syntax Tree
│   ├── lexer.rs         # Tokenization
│   ├── parser.rs        # Parsing
│   ├── roller.rs        # Dice rolling engine
│   ├── result.rs        # Result types
│   └── error.rs         # Error types
├── Cargo.toml           # Rust dependencies
└── flake.nix            # Nix flake configuration
```

### Running Tests
```bash
# With Nix
nix build  # Tests run automatically

# With Cargo
cargo test
cargo test --verbose
```

### Linting and Formatting
```bash
cargo clippy
cargo fmt
```

## Specification

See [SPEC.md](SPEC.md) for the complete dice notation specification and implementation details.

## License

MIT

## Contributing

Contributions welcome! The project follows standard Rust conventions and includes comprehensive tests.
