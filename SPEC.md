# Dice Roller Specification

## Overview
A CLI dice roller supporting Roll20-style dice notation with comprehensive modifiers and operations.

## 1. Basic Dice Notation

### 1.1 Standard Dice (NdX)
- Format: `NdX` where N is number of dice and X is number of sides
- Example: `3d6` rolls three 6-sided dice
- N must be >= 0, X must be >= 1
- Support for single die: `1d20` or `d20`

### 1.2 FATE/Fudge Dice (NdF)
- Format: `NdF` where N is number of FATE dice
- FATE dice have three sides: -1, 0, +1
- Example: `4dF` rolls four FATE dice

### 1.3 Computed Dice
- Number of dice can be computed: `(2+3)d6`
- Number of sides can be computed: `3d(4+2)`
- Computed dice count rounds to nearest integer

## 2. Mathematical Operations

### 2.1 Basic Operators
- Addition: `+` (e.g., `1d20+5`)
- Subtraction: `-` (e.g., `2d6-2`)
- Multiplication: `*` (e.g., `3d4*2`)
- Division: `/` (e.g., `4d6/2`)
- Modulus: `%` (e.g., `1d20%5`)
- Exponentiation: `**` (e.g., `2d4**2`)

### 2.2 Mathematical Functions
- `floor(x)` - Round down to nearest integer
- `ceil(x)` - Round up to nearest integer
- `round(x)` - Round to nearest integer (0.5 rounds up)
- `abs(x)` - Absolute value

### 2.3 Operator Precedence
1. Parentheses `()`
2. Function calls (floor, ceil, round, abs)
3. Exponentiation `**`
4. Multiplication `*`, Division `/`, Modulus `%` (left-to-right)
5. Addition `+`, Subtraction `-` (left-to-right)

## 3. Roll Modifiers

### 3.1 Exploding Dice (!CP)
- `NdX!` - Explode on maximum (e.g., `3d6!` explodes on 6)
- `NdX!>Y` - Explode on Y or higher (e.g., `3d6!>5`)
- `NdX!Y` - Explode on exactly Y (e.g., `3d6!6`)
- `NdX!<Y` - Explode on Y or lower
- Each explosion can trigger additional explosions

### 3.2 Compounding Exploding Dice (!!CP)
- `NdX!!` - Compound exploding dice (Shadowrun style)
- Additional rolls are added to the original die
- Compare Point can be specified: `5d6!!>5`

### 3.3 Penetrating Dice (!pCP)
- `NdX!p` - Penetrating exploding dice (HackMaster style)
- Each additional roll has -1 applied
- Compare Point can be specified: `5d6!p>5`

### 3.4 Keep/Drop Dice
- `NdXkhY` - Keep highest Y dice (e.g., `4d6kh3`)
- `NdXklY` - Keep lowest Y dice (e.g., `4d6kl1`)
- `NdXdhY` - Drop highest Y dice (e.g., `8d6dh2`)
- `NdXdlY` - Drop lowest Y dice (e.g., `4d6dl1`)
- Shortcuts: `k` = `kh`, `d` = `dl`

### 3.5 Rerolling Dice (rCP)
- `NdXr<Y` - Reroll dice Y or lower (e.g., `2d10r<2`)
- `NdXrY` - Reroll on exactly Y (e.g., `8d6r1`)
- `NdXr>Y` - Reroll on Y or higher
- Multiple reroll conditions: `8d6r2r4r6`
- Rerolls continue until condition not met

### 3.6 Reroll Once (roCP)
- `NdXro<Y` - Reroll once on Y or lower
- Only rerolls each die once regardless of result

### 3.7 Target Number/Successes (CP)
- `NdX>Y` - Count successes for rolls >= Y (e.g., `3d6>3`)
- `NdX<Y` - Count successes for rolls <= Y (e.g., `10d6<4`)
- `NdX=Y` or `NdXY` - Count successes for rolls equal to Y

### 3.8 Failures (fCP)
- Must be used with success checks
- `NdX>YfZ` - Count failures for rolls equal to Z
- `NdX>Yf<Z` - Count failures for rolls <= Z
- `NdX>Yf>Z` - Count failures for rolls >= Z
- Failures subtract from successes

### 3.9 Critical Success/Failure (cs/cf)
- `NdXcs>Y` - Mark critical success on Y or higher
- `NdXcf<Y` - Mark critical failure on Y or lower
- Visual indicator only, doesn't affect total
- Multiple markers: `1d20cs20cs19cf1cf2`

### 3.10 Sorting (sa/sd)
- `NdXsa` - Sort results ascending
- `NdXsd` - Sort results descending
- Default is unsorted

### 3.11 Modifier Order of Operations
1. Exploding, Compounding, Penetrating, Rerolls (while rolling)
2. Keep, Drop, Success, Failure, Sorting (after all dice settled)

## 4. Grouped Rolls

### 4.1 Basic Groups
- Format: `{Roll1, Roll2, ...}` or `{Roll1+Roll2+...}`
- Comma-separated: Treats each as separate roll
- Plus-separated: Combines into single pool
- Example: `{4d6, 3d8}kh1` keeps highest group total
- Example: `{4d6+3d8}kh1` keeps highest single die

### 4.2 Group Modifiers
- Keep/Drop works across groups: `{4d6, 3d8, 5d10}kh2`
- Success checks on groups: `{3d20+5}>21`
- Multiple expressions: `{4d6+2d8, 3d20+3, 5d10+1}>40`

## 5. Inline Rolls

### 5.1 Inline Roll Syntax
- Format: `[[roll]]`
- Can be nested: `[[2d6 + [[1d4]]]]`
- Can be used in computed dice: `[[2d6]]d[[1d4]]`

### 5.2 Labels
- Inline labels: `2d10+5[Fire Damage]`
- Appears as tooltip on results
- Multiple labels: `2d10+5[Fire] + 3d6[Ice]`
- Labels can follow plain numbers: `1d20+2[Strength]`
- Labels never change the result

## 6. Output Format

### 6.1 Display Requirements
- Show individual die results
- Show final total
- Indicate dropped/kept dice
- Mark critical successes/failures
- Show exploded dice
- Display success counts

### 6.2 Result Components
```
Total: X
Rolls: [d1, d2, d3, ...]
Kept: [...]
Dropped: [...]
Successes: N
Failures: N
```

## 7. Implementation Phases

### Phase 1: Core Functionality
- Basic dice notation (NdX, NdF)
- Math operators (+, -, *, /, %)
- Simple rolls without modifiers

### Phase 2: Basic Modifiers
- Exploding dice (!)
- Keep/Drop (kh, kl, dh, dl)
- Target numbers (>, <, =)

### Phase 3: Advanced Modifiers
- Compounding (!!)
- Penetrating (!p)
- Rerolls (r, ro)
- Failures (f)
- Critical markers (cs, cf)
- Sorting (sa, sd)

### Phase 4: Advanced Features
- Grouped rolls
- Inline rolls
- Computed dice
- Math functions
- Labels

## 8. Error Handling

### 8.1 Parse Errors
- Unparsed trailing input (e.g. `1d20+2kh`)
- Invalid syntax: Clear error message with position
- Invalid dice notation: Explain valid formats
- Mismatched parentheses: Show location

### 8.2 Runtime Errors
- Division by zero: Error message
- Invalid operations: Type errors
- Infinite loops: Max iteration limits

## 9. CLI Interface

### 9.1 Basic Usage
```bash
dice_roller "1d20+5"
dice_roller "4d6kh3"
dice_roller "8d6!>5"
```

### 9.2 Options
- `-v, --verbose` - Show detailed roll breakdown
- `-q, --quiet` - Show only final total
- `-n, --count N` - Repeat roll N times
- `--no-color` - Disable colored output
- `-h, --help` - Show help
- `--version` - Show version

### 9.3 Interactive Mode
```bash
dice_roller
> 1d20+5
> 4d6kh3
> exit
```

## 10. Testing Requirements

### 10.1 Unit Tests
- Parser correctness
- Each modifier type
- Math operations
- Edge cases (0d6, 1d1, etc.)

### 10.2 Integration Tests
- Complex roll combinations
- Multiple modifiers
- Grouped rolls
- Error handling

### 10.3 Property Tests
- Statistical distribution verification
- Deterministic behavior with seed
- No panics on any input

## 11. Future Enhancements

- Roll macros/saved rolls
- Roll history
- Statistical analysis
- Web API
- Roll templates
- Multiple output formats (JSON, CSV)
