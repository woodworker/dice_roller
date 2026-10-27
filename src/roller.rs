use crate::ast::*;
use crate::error::{DiceError, Result};
use crate::result::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

const MAX_ITERATIONS: usize = 10000;

pub struct DiceRoller {
    rng: StdRng,
}

impl DiceRoller {
    pub fn new() -> Self {
        Self {
            rng: StdRng::from_entropy(),
        }
    }

    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    pub fn roll(&mut self, expr: &Expr) -> Result<RollResult> {
        let details = self.evaluate(expr)?;
        let total = self.extract_total(&details);
        let expression = format!("{:?}", expr); // TODO: Better formatting

        Ok(RollResult::new(total, details, expression))
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<RollDetails> {
        match expr {
            Expr::Literal(n) => Ok(RollDetails::Literal(*n)),

            Expr::Dice {
                count,
                sides,
                modifiers,
            } => self.evaluate_dice(count, sides, modifiers),

            Expr::FateDice { count, modifiers } => self.evaluate_fate_dice(count, modifiers),

            Expr::DualityDice => Ok(RollDetails::DualityDice(DualityDiceResult {
                hope: self.roll_die(12),
                fear: self.roll_die(12),
            })),

            Expr::BinaryOp { op, left, right } => self.evaluate_binary_op(*op, left, right),

            Expr::UnaryOp { op, expr } => self.evaluate_unary_op(*op, expr),

            Expr::Function { name, arg } => self.evaluate_function(*name, arg),

            Expr::Group { rolls, modifiers } => self.evaluate_group(rolls, modifiers),

            Expr::Inline { expr, label: _ } => self.evaluate(expr),
        }
    }

    fn extract_total(&self, details: &RollDetails) -> i64 {
        match details {
            RollDetails::Literal(n) => *n,
            RollDetails::Dice(d) => d.total,
            RollDetails::FateDice(f) => f.total,
            RollDetails::DualityDice(d) => d.total(),
            RollDetails::BinaryOp { result, .. } => *result,
            RollDetails::Function { result, .. } => *result,
            RollDetails::Group { result, .. } => *result,
        }
    }

    fn evaluate_binary_op(
        &mut self,
        op: BinaryOperator,
        left: &Expr,
        right: &Expr,
    ) -> Result<RollDetails> {
        let left_details = self.evaluate(left)?;
        let right_details = self.evaluate(right)?;

        let left_val = self.extract_total(&left_details);
        let right_val = self.extract_total(&right_details);

        let result = match op {
            BinaryOperator::Add => left_val + right_val,
            BinaryOperator::Subtract => left_val - right_val,
            BinaryOperator::Multiply => left_val * right_val,
            BinaryOperator::Divide => {
                if right_val == 0 {
                    return Err(DiceError::DivisionByZero);
                }
                left_val / right_val
            }
            BinaryOperator::Modulo => {
                if right_val == 0 {
                    return Err(DiceError::DivisionByZero);
                }
                left_val % right_val
            }
            BinaryOperator::Exponent => {
                if right_val < 0 {
                    return Err(DiceError::EvaluationError(
                        "Negative exponents not supported".to_string(),
                    ));
                }
                left_val.pow(right_val as u32)
            }
        };

        Ok(RollDetails::BinaryOp {
            op: format!("{:?}", op),
            left: Box::new(left_details),
            right: Box::new(right_details),
            result,
        })
    }

    fn evaluate_unary_op(&mut self, op: UnaryOperator, expr: &Expr) -> Result<RollDetails> {
        let details = self.evaluate(expr)?;
        let val = self.extract_total(&details);

        match op {
            UnaryOperator::Negate => Ok(RollDetails::Literal(-val)),
            UnaryOperator::Plus => Ok(details),
        }
    }

    fn evaluate_function(&mut self, name: FunctionName, arg: &Expr) -> Result<RollDetails> {
        let arg_details = self.evaluate(arg)?;
        let arg_val = self.extract_total(&arg_details) as f64;

        let result = match name {
            FunctionName::Floor => arg_val.floor() as i64,
            FunctionName::Ceil => arg_val.ceil() as i64,
            FunctionName::Round => arg_val.round() as i64,
            FunctionName::Abs => arg_val.abs() as i64,
        };

        Ok(RollDetails::Function {
            name: format!("{:?}", name),
            arg: Box::new(arg_details),
            result,
        })
    }

    fn evaluate_dice(
        &mut self,
        count_expr: &Expr,
        sides_expr: &Expr,
        modifiers: &[Modifier],
    ) -> Result<RollDetails> {
        // Evaluate count and sides
        let count_details = self.evaluate(count_expr)?;
        let count = self.extract_total(&count_details);

        if count < 0 {
            return Err(DiceError::InvalidDiceCount(count));
        }
        let count = count as usize;

        let sides_details = self.evaluate(sides_expr)?;
        let sides = self.extract_total(&sides_details);

        if sides < 1 {
            return Err(DiceError::InvalidDiceSides(sides));
        }

        let mut result = DiceRollResult::new(count, sides);

        // Roll the dice
        for _ in 0..count {
            let roll = self.roll_die(sides);
            result.rolls.push(DieRoll::new(roll));
        }

        // Apply modifiers in order
        self.apply_dice_modifiers(&mut result, modifiers)?;

        // Calculate final total
        result.total = self.calculate_dice_total(&result);

        Ok(RollDetails::Dice(result))
    }

    fn evaluate_fate_dice(
        &mut self,
        count_expr: &Expr,
        _modifiers: &[Modifier],
    ) -> Result<RollDetails> {
        let count_details = self.evaluate(count_expr)?;
        let count = self.extract_total(&count_details);

        if count < 0 {
            return Err(DiceError::InvalidDiceCount(count));
        }
        let count = count as usize;

        let mut result = FateDiceResult::new(count);

        for _ in 0..count {
            let roll = self.rng.gen_range(-1..=1);
            result.rolls.push(roll);
        }

        result.total = result.rolls.iter().sum();

        Ok(RollDetails::FateDice(result))
    }

    fn evaluate_group(
        &mut self,
        rolls: &[Expr],
        modifiers: &[GroupModifier],
    ) -> Result<RollDetails> {
        let mut roll_details = Vec::new();
        let mut roll_totals = Vec::new();

        for roll_expr in rolls {
            let details = self.evaluate(roll_expr)?;
            let total = self.extract_total(&details);
            roll_totals.push(total);
            roll_details.push(details);
        }

        // Apply group modifiers
        let result = self.apply_group_modifiers(&roll_totals, modifiers)?;

        Ok(RollDetails::Group {
            rolls: roll_details,
            result,
        })
    }

    fn roll_die(&mut self, sides: i64) -> i64 {
        self.rng.gen_range(1..=sides)
    }

    fn apply_dice_modifiers(
        &mut self,
        result: &mut DiceRollResult,
        modifiers: &[Modifier],
    ) -> Result<()> {
        // Phase 1: Exploding, Compounding, Penetrating, Rerolls
        for modifier in modifiers {
            match modifier {
                Modifier::Exploding(cp) => self.apply_exploding(result, cp)?,
                Modifier::Compounding(cp) => self.apply_compounding(result, cp)?,
                Modifier::Penetrating(cp) => self.apply_penetrating(result, cp)?,
                Modifier::Reroll(cp) => self.apply_reroll(result, cp, false)?,
                Modifier::RerollOnce(cp) => self.apply_reroll(result, cp, true)?,
                _ => {}
            }
        }

        // Phase 2: Keep, Drop, Success, Failure, Sorting
        for modifier in modifiers {
            match modifier {
                Modifier::KeepHigh(n) => self.apply_keep_high(result, *n),
                Modifier::KeepLow(n) => self.apply_keep_low(result, *n),
                Modifier::DropHigh(n) => self.apply_drop_high(result, *n),
                Modifier::DropLow(n) => self.apply_drop_low(result, *n),
                Modifier::Success(cp) => self.apply_success(result, cp),
                Modifier::Failure(cp) => self.apply_failure(result, cp),
                Modifier::CriticalSuccess(cp) => self.apply_critical_success(result, cp),
                Modifier::CriticalFailure(cp) => self.apply_critical_failure(result, cp),
                Modifier::SortAscending => self.apply_sort_ascending(result),
                Modifier::SortDescending => self.apply_sort_descending(result),
                _ => {}
            }
        }

        Ok(())
    }

    fn apply_exploding(&mut self, result: &mut DiceRollResult, cp: &ComparePoint) -> Result<()> {
        let cp = self.resolve_compare_point(cp, result.sides);
        let mut iterations = 0;

        let mut i = 0;
        while i < result.rolls.len() {
            if iterations > MAX_ITERATIONS {
                return Err(DiceError::MaxIterationsExceeded);
            }

            if cp.matches(result.rolls[i].value) {
                result.rolls[i].exploded = true;
                let new_roll = self.roll_die(result.sides);
                result.rolls.push(DieRoll::new(new_roll));
            }

            i += 1;
            iterations += 1;
        }

        Ok(())
    }

    fn apply_compounding(&mut self, result: &mut DiceRollResult, cp: &ComparePoint) -> Result<()> {
        let cp = self.resolve_compare_point(cp, result.sides);
        let mut iterations = 0;

        for die in result.rolls.iter_mut() {
            let mut total = die.value;
            let mut last_roll = die.value;
            die.compound_parts.push(die.value);

            // Compare each new roll, not the running total
            while cp.matches(last_roll) {
                if iterations > MAX_ITERATIONS {
                    return Err(DiceError::MaxIterationsExceeded);
                }

                last_roll = self.roll_die(result.sides);
                die.compound_parts.push(last_roll);
                total += last_roll;
                die.exploded = true;
                iterations += 1;
            }

            die.value = total;
        }

        Ok(())
    }

    fn apply_penetrating(&mut self, result: &mut DiceRollResult, cp: &ComparePoint) -> Result<()> {
        let cp = self.resolve_compare_point(cp, result.sides);
        let mut iterations = 0;

        let mut i = 0;
        while i < result.rolls.len() {
            if iterations > MAX_ITERATIONS {
                return Err(DiceError::MaxIterationsExceeded);
            }

            if cp.matches(result.rolls[i].value) {
                result.rolls[i].exploded = true;
                let new_roll = self.roll_die(result.sides) - 1;
                result.rolls.push(DieRoll::new(new_roll.max(0)));
            }

            i += 1;
            iterations += 1;
        }

        Ok(())
    }

    fn apply_reroll(
        &mut self,
        result: &mut DiceRollResult,
        cp: &ComparePoint,
        once: bool,
    ) -> Result<()> {
        let mut iterations = 0;

        for die in result.rolls.iter_mut() {
            while cp.matches(die.value) {
                if iterations > MAX_ITERATIONS {
                    return Err(DiceError::MaxIterationsExceeded);
                }

                die.rerolled = true;
                die.value = self.roll_die(result.sides);
                iterations += 1;

                if once {
                    break;
                }
            }
        }

        Ok(())
    }

    fn apply_keep_high(&mut self, result: &mut DiceRollResult, n: usize) {
        if n >= result.rolls.len() {
            result.kept = (0..result.rolls.len()).collect();
            return;
        }

        let mut indexed: Vec<_> = result
            .rolls
            .iter()
            .enumerate()
            .map(|(i, roll)| (i, roll.value))
            .collect();

        indexed.sort_by(|a, b| b.1.cmp(&a.1));

        result.kept = indexed.iter().take(n).map(|(i, _)| *i).collect();
        result.dropped = indexed.iter().skip(n).map(|(i, _)| *i).collect();
    }

    fn apply_keep_low(&mut self, result: &mut DiceRollResult, n: usize) {
        if n >= result.rolls.len() {
            result.kept = (0..result.rolls.len()).collect();
            return;
        }

        let mut indexed: Vec<_> = result
            .rolls
            .iter()
            .enumerate()
            .map(|(i, roll)| (i, roll.value))
            .collect();

        indexed.sort_by(|a, b| a.1.cmp(&b.1));

        result.kept = indexed.iter().take(n).map(|(i, _)| *i).collect();
        result.dropped = indexed.iter().skip(n).map(|(i, _)| *i).collect();
    }

    fn apply_drop_high(&mut self, result: &mut DiceRollResult, n: usize) {
        if n >= result.rolls.len() {
            result.dropped = (0..result.rolls.len()).collect();
            return;
        }

        let keep_count = result.rolls.len() - n;
        self.apply_keep_low(result, keep_count);
    }

    fn apply_drop_low(&mut self, result: &mut DiceRollResult, n: usize) {
        if n >= result.rolls.len() {
            result.dropped = (0..result.rolls.len()).collect();
            return;
        }

        let keep_count = result.rolls.len() - n;
        self.apply_keep_high(result, keep_count);
    }

    fn apply_success(&mut self, result: &mut DiceRollResult, cp: &ComparePoint) {
        let count = result
            .rolls
            .iter()
            .filter(|roll| cp.matches(roll.value))
            .count() as i64;

        result.successes = Some(count);
    }

    fn apply_failure(&mut self, result: &mut DiceRollResult, cp: &ComparePoint) {
        let count = result
            .rolls
            .iter()
            .filter(|roll| cp.matches(roll.value))
            .count() as i64;

        result.failures = Some(count);
    }

    fn apply_critical_success(&mut self, result: &mut DiceRollResult, cp: &ComparePoint) {
        for roll in result.rolls.iter_mut() {
            if cp.matches(roll.value) {
                roll.critical_success = true;
            }
        }
    }

    fn apply_critical_failure(&mut self, result: &mut DiceRollResult, cp: &ComparePoint) {
        for roll in result.rolls.iter_mut() {
            if cp.matches(roll.value) {
                roll.critical_failure = true;
            }
        }
    }

    fn apply_sort_ascending(&mut self, result: &mut DiceRollResult) {
        result.rolls.sort_by(|a, b| a.value.cmp(&b.value));
    }

    fn apply_sort_descending(&mut self, result: &mut DiceRollResult) {
        result.rolls.sort_by(|a, b| b.value.cmp(&a.value));
    }

    fn resolve_compare_point(&self, cp: &ComparePoint, sides: i64) -> ComparePoint {
        if cp.value == 0 {
            // Default explode point is max sides
            ComparePoint::new(CompareOp::Equal, sides)
        } else {
            cp.clone()
        }
    }

    fn calculate_dice_total(&self, result: &DiceRollResult) -> i64 {
        // If we have success/failure counting, return that
        if let Some(successes) = result.successes {
            let failures = result.failures.unwrap_or(0);
            return successes - failures;
        }

        // Otherwise sum kept dice
        if !result.kept.is_empty() {
            result.kept.iter().map(|&i| result.rolls[i].value).sum()
        } else if !result.dropped.is_empty() {
            result
                .rolls
                .iter()
                .enumerate()
                .filter(|(i, _)| !result.dropped.contains(i))
                .map(|(_, roll)| roll.value)
                .sum()
        } else {
            result.rolls.iter().map(|roll| roll.value).sum()
        }
    }

    fn apply_group_modifiers(&self, totals: &[i64], modifiers: &[GroupModifier]) -> Result<i64> {
        let mut totals = totals.to_vec();
        let mut successes = None;
        let mut failures = None;

        for modifier in modifiers {
            match modifier {
                GroupModifier::KeepHigh(n) => {
                    totals.sort_by(|a, b| b.cmp(a));
                    totals.truncate(*n);
                }
                GroupModifier::KeepLow(n) => {
                    totals.sort();
                    totals.truncate(*n);
                }
                GroupModifier::DropHigh(n) => {
                    totals.sort_by(|a, b| b.cmp(a));
                    totals = totals.into_iter().skip(*n).collect();
                }
                GroupModifier::DropLow(n) => {
                    totals.sort();
                    totals = totals.into_iter().skip(*n).collect();
                }
                GroupModifier::Success(cp) => {
                    let count = totals.iter().filter(|&&v| cp.matches(v)).count() as i64;
                    successes = Some(count);
                }
                GroupModifier::Failure(cp) => {
                    let count = totals.iter().filter(|&&v| cp.matches(v)).count() as i64;
                    failures = Some(count);
                }
            }
        }

        if let Some(s) = successes {
            Ok(s - failures.unwrap_or(0))
        } else {
            Ok(totals.iter().sum())
        }
    }
}

impl Default for DiceRoller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roll_basic_dice() {
        let mut roller = DiceRoller::with_seed(42);
        let expr = Expr::dice(Expr::Literal(1), Expr::Literal(20), vec![]);
        let result = roller.roll(&expr).unwrap();
        assert!(result.total >= 1 && result.total <= 20);
    }

    #[test]
    fn test_roll_with_modifier() {
        let mut roller = DiceRoller::with_seed(42);
        let expr = Expr::binary_op(
            BinaryOperator::Add,
            Expr::dice(Expr::Literal(1), Expr::Literal(20), vec![]),
            Expr::Literal(5),
        );
        let result = roller.roll(&expr).unwrap();
        assert!(result.total >= 6 && result.total <= 25);
    }

    #[test]
    fn test_fate_dice() {
        let mut roller = DiceRoller::with_seed(42);
        let expr = Expr::fate_dice(Expr::Literal(4), vec![]);
        let result = roller.roll(&expr).unwrap();
        assert!(result.total >= -4 && result.total <= 4);
    }

    #[test]
    fn test_compounding_compares_each_roll() {
        // !!>5 used to compare the running total and never stopped
        let expr = crate::Parser::new("8d6!!>5").unwrap().parse().unwrap();
        for seed in 0..100 {
            let result = DiceRoller::with_seed(seed).roll(&expr).unwrap();
            let RollDetails::Dice(dice) = result.details else {
                panic!("expected dice");
            };
            for die in dice.rolls {
                let (last, parts) = die.compound_parts.split_last().unwrap();
                assert!(parts.iter().all(|p| *p >= 5));
                assert!(*last < 5);
            }
        }
    }

    #[test]
    fn test_duality_dice() {
        for seed in 0..100 {
            let mut roller = DiceRoller::with_seed(seed);
            let expr = Expr::binary_op(BinaryOperator::Add, Expr::DualityDice, Expr::Literal(3));
            let result = roller.roll(&expr).unwrap();
            let duality = result.duality_dice().unwrap();
            assert!((1..=12).contains(&duality.hope));
            assert!((1..=12).contains(&duality.fear));
            assert_eq!(result.total, duality.hope + duality.fear + 3);
        }
    }

    #[test]
    fn test_duality_outcome() {
        let roll = |hope, fear| DualityDiceResult { hope, fear }.duality();
        assert_eq!(roll(9, 5), Duality::Hope);
        assert_eq!(roll(3, 7), Duality::Fear);
        assert_eq!(roll(6, 6), Duality::Critical);
    }
}
