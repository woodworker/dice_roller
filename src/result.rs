/// Represents the result of a dice roll

#[derive(Debug, Clone, PartialEq)]
pub struct RollResult {
    /// The final total value
    pub total: i64,

    /// Detailed breakdown of the roll
    pub details: RollDetails,

    /// Original expression string
    pub expression: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RollDetails {
    /// A simple literal value
    Literal(i64),

    /// Result of a dice roll
    Dice(DiceRollResult),

    /// Result of FATE dice
    FateDice(FateDiceResult),

    /// Result of Daggerheart Duality Dice
    DualityDice(DualityDiceResult),

    /// Result of a binary operation
    BinaryOp {
        op: String,
        left: Box<RollDetails>,
        right: Box<RollDetails>,
        result: i64,
    },

    /// Result of a function
    Function {
        name: String,
        arg: Box<RollDetails>,
        result: i64,
    },

    /// Result of a grouped roll
    Group {
        rolls: Vec<RollDetails>,
        result: i64,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiceRollResult {
    /// Number of dice rolled
    pub count: usize,

    /// Number of sides per die
    pub sides: i64,

    /// All individual die rolls
    pub rolls: Vec<DieRoll>,

    /// Dice that were kept (after keep/drop modifiers)
    pub kept: Vec<usize>,

    /// Dice that were dropped
    pub dropped: Vec<usize>,

    /// Number of successes (if using success counting)
    pub successes: Option<i64>,

    /// Number of failures (if using failure counting)
    pub failures: Option<i64>,

    /// Final total of this dice roll
    pub total: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FateDiceResult {
    /// Number of dice rolled
    pub count: usize,

    /// Individual rolls
    pub rolls: Vec<i64>,

    /// Final total
    pub total: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DualityDiceResult {
    /// The Hope d12
    pub hope: i64,

    /// The Fear d12
    pub fear: i64,
}

/// Which Duality Die decided the roll
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Duality {
    Hope,
    Fear,
    /// Both dice show the same number
    Critical,
}

impl DualityDiceResult {
    pub fn total(&self) -> i64 {
        self.hope + self.fear
    }

    pub fn duality(&self) -> Duality {
        match self.hope.cmp(&self.fear) {
            std::cmp::Ordering::Greater => Duality::Hope,
            std::cmp::Ordering::Less => Duality::Fear,
            std::cmp::Ordering::Equal => Duality::Critical,
        }
    }

    /// Short description, e.g. "with Hope (Hope 9, Fear 5)"
    pub fn describe(&self) -> String {
        let outcome = match self.duality() {
            Duality::Hope => "with Hope",
            Duality::Fear => "with Fear",
            Duality::Critical => "Critical Success",
        };
        format!("{} (Hope {}, Fear {})", outcome, self.hope, self.fear)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DieRoll {
    /// The value rolled
    pub value: i64,

    /// Whether this die exploded
    pub exploded: bool,

    /// Whether this die was rerolled
    pub rerolled: bool,

    /// Whether this is a critical success
    pub critical_success: bool,

    /// Whether this is a critical failure
    pub critical_failure: bool,

    /// If this die was part of a compound explosion, the original rolls
    pub compound_parts: Vec<i64>,
}

impl DieRoll {
    pub fn new(value: i64) -> Self {
        Self {
            value,
            exploded: false,
            rerolled: false,
            critical_success: false,
            critical_failure: false,
            compound_parts: Vec::new(),
        }
    }
}

impl RollResult {
    pub fn new(total: i64, details: RollDetails, expression: String) -> Self {
        Self {
            total,
            details,
            expression,
        }
    }

    /// The Duality Dice of this roll, if the expression contains `dD`
    pub fn duality_dice(&self) -> Option<&DualityDiceResult> {
        fn find(details: &RollDetails) -> Option<&DualityDiceResult> {
            match details {
                RollDetails::DualityDice(d) => Some(d),
                RollDetails::BinaryOp { left, right, .. } => find(left).or_else(|| find(right)),
                RollDetails::Function { arg, .. } => find(arg),
                RollDetails::Group { rolls, .. } => rolls.iter().find_map(find),
                _ => None,
            }
        }
        find(&self.details)
    }

    /// Format the result as a human-readable string
    pub fn format(&self, verbose: bool) -> String {
        if verbose {
            self.format_verbose()
        } else {
            format!("{}", self.total)
        }
    }

    fn format_verbose(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("Expression: {}\n", self.expression));
        output.push_str(&format!("Total: {}\n", self.total));
        if let Some(duality) = self.duality_dice() {
            output.push_str(&format!("Duality: {}\n", duality.describe()));
        }
        output.push_str(&self.format_details(&self.details, 0));
        output
    }

    fn format_details(&self, details: &RollDetails, indent: usize) -> String {
        let prefix = "  ".repeat(indent);
        match details {
            RollDetails::Literal(n) => format!("{}Literal: {}\n", prefix, n),
            RollDetails::Dice(d) => self.format_dice_result(d, indent),
            RollDetails::FateDice(f) => self.format_fate_result(f, indent),
            RollDetails::DualityDice(d) => format!(
                "{}Duality Dice: Hope {}, Fear {}\n{}Total: {}\n",
                prefix,
                d.hope,
                d.fear,
                prefix,
                d.total()
            ),
            RollDetails::BinaryOp {
                op,
                left,
                right,
                result,
            } => {
                let mut s = format!("{}Binary Op: {} = {}\n", prefix, op, result);
                s.push_str(&self.format_details(left, indent + 1));
                s.push_str(&self.format_details(right, indent + 1));
                s
            }
            RollDetails::Function { name, arg, result } => {
                let mut s = format!("{}Function: {}() = {}\n", prefix, name, result);
                s.push_str(&self.format_details(arg, indent + 1));
                s
            }
            RollDetails::Group { rolls, result } => {
                let mut s = format!("{}Group: total = {}\n", prefix, result);
                for roll in rolls {
                    s.push_str(&self.format_details(roll, indent + 1));
                }
                s
            }
        }
    }

    fn format_dice_result(&self, dice: &DiceRollResult, indent: usize) -> String {
        let prefix = "  ".repeat(indent);
        let mut s = format!("{}Dice: {}d{}\n", prefix, dice.count, dice.sides);

        s.push_str(&format!("{}Rolls: [", prefix));
        for (i, roll) in dice.rolls.iter().enumerate() {
            if i > 0 {
                s.push_str(", ");
            }

            let mut markers = Vec::new();
            if roll.exploded {
                markers.push("!");
            }
            if roll.rerolled {
                markers.push("r");
            }
            if roll.critical_success {
                markers.push("★");
            }
            if roll.critical_failure {
                markers.push("✗");
            }
            if dice.dropped.contains(&i) {
                markers.push("dropped");
            }

            if markers.is_empty() {
                s.push_str(&format!("{}", roll.value));
            } else {
                s.push_str(&format!("{}({})", roll.value, markers.join(",")));
            }
        }
        s.push_str("]\n");

        if let Some(successes) = dice.successes {
            s.push_str(&format!("{}Successes: {}\n", prefix, successes));
        }
        if let Some(failures) = dice.failures {
            s.push_str(&format!("{}Failures: {}\n", prefix, failures));
        }

        s.push_str(&format!("{}Total: {}\n", prefix, dice.total));
        s
    }

    fn format_fate_result(&self, fate: &FateDiceResult, indent: usize) -> String {
        let prefix = "  ".repeat(indent);
        let mut s = format!("{}FATE Dice: {}dF\n", prefix, fate.count);
        s.push_str(&format!("{}Rolls: {:?}\n", prefix, fate.rolls));
        s.push_str(&format!("{}Total: {}\n", prefix, fate.total));
        s
    }
}

impl DiceRollResult {
    pub fn new(count: usize, sides: i64) -> Self {
        Self {
            count,
            sides,
            rolls: Vec::new(),
            kept: Vec::new(),
            dropped: Vec::new(),
            successes: None,
            failures: None,
            total: 0,
        }
    }
}

impl FateDiceResult {
    pub fn new(count: usize) -> Self {
        Self {
            count,
            rolls: Vec::new(),
            total: 0,
        }
    }
}
