/// Abstract Syntax Tree for dice notation expressions

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A literal integer value
    Literal(i64),

    /// A dice roll: NdX
    Dice {
        count: Box<Expr>,
        sides: Box<Expr>,
        modifiers: Vec<Modifier>,
    },

    /// FATE/Fudge dice: NdF
    FateDice {
        count: Box<Expr>,
        modifiers: Vec<Modifier>,
    },

    /// Daggerheart Duality Dice: dD (one Hope d12 and one Fear d12)
    DualityDice,

    /// Binary operation
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    /// Unary operation
    UnaryOp { op: UnaryOperator, expr: Box<Expr> },

    /// Function call
    Function { name: FunctionName, arg: Box<Expr> },

    /// Grouped rolls: {Roll1, Roll2, ...}
    Group {
        rolls: Vec<Expr>,
        modifiers: Vec<GroupModifier>,
    },

    /// Inline roll: [[...]]
    Inline {
        expr: Box<Expr>,
        label: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Exponent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Negate,
    Plus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionName {
    Floor,
    Ceil,
    Round,
    Abs,
}

/// Modifiers that can be applied to dice rolls
#[derive(Debug, Clone, PartialEq)]
pub enum Modifier {
    /// Exploding dice: !CP
    Exploding(ComparePoint),

    /// Compounding exploding: !!CP
    Compounding(ComparePoint),

    /// Penetrating exploding: !pCP
    Penetrating(ComparePoint),

    /// Keep highest N: khN
    KeepHigh(usize),

    /// Keep lowest N: klN
    KeepLow(usize),

    /// Drop highest N: dhN
    DropHigh(usize),

    /// Drop lowest N: dlN
    DropLow(usize),

    /// Reroll: rCP
    Reroll(ComparePoint),

    /// Reroll once: roCP
    RerollOnce(ComparePoint),

    /// Target number / Successes: >N, <N, =N
    Success(ComparePoint),

    /// Failures: fCP
    Failure(ComparePoint),

    /// Critical success marker: cs>N
    CriticalSuccess(ComparePoint),

    /// Critical failure marker: cf<N
    CriticalFailure(ComparePoint),

    /// Sort ascending: sa
    SortAscending,

    /// Sort descending: sd
    SortDescending,

    /// Label for the roll
    Label(String),
}

/// Group modifiers that apply to grouped rolls
#[derive(Debug, Clone, PartialEq)]
pub enum GroupModifier {
    KeepHigh(usize),
    KeepLow(usize),
    DropHigh(usize),
    DropLow(usize),
    Success(ComparePoint),
    Failure(ComparePoint),
}

/// Compare point for modifiers
#[derive(Debug, Clone, PartialEq)]
pub struct ComparePoint {
    pub op: CompareOp,
    pub value: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Equal,
    GreaterOrEqual,
    LessOrEqual,
}

impl ComparePoint {
    pub fn new(op: CompareOp, value: i64) -> Self {
        Self { op, value }
    }

    /// Create a default compare point (equal to max/6 for d6, etc.)
    pub fn default_explode() -> Self {
        Self {
            op: CompareOp::Equal,
            value: 0, // Will be set to max sides when rolling
        }
    }

    /// Check if a value matches this compare point
    pub fn matches(&self, value: i64) -> bool {
        match self.op {
            CompareOp::Equal => value == self.value,
            CompareOp::GreaterOrEqual => value >= self.value,
            CompareOp::LessOrEqual => value <= self.value,
        }
    }
}

impl Expr {
    /// Helper to create a literal expression
    pub fn literal(value: i64) -> Self {
        Expr::Literal(value)
    }

    /// Helper to create a binary operation
    pub fn binary_op(op: BinaryOperator, left: Expr, right: Expr) -> Self {
        Expr::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// Helper to create a dice roll
    pub fn dice(count: Expr, sides: Expr, modifiers: Vec<Modifier>) -> Self {
        Expr::Dice {
            count: Box::new(count),
            sides: Box::new(sides),
            modifiers,
        }
    }

    /// Helper to create FATE dice
    pub fn fate_dice(count: Expr, modifiers: Vec<Modifier>) -> Self {
        Expr::FateDice {
            count: Box::new(count),
            modifiers,
        }
    }
}
