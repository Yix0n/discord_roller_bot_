use std::collections::HashMap;
use lazy_static::lazy_static;
use rand::Rng;
use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub enum ExprType {
    Number(f64),
    Operator(String),
    // roll amount, sides
    StandardDice(u32, u32),
    // roll amount, sides, exploding value (0 = explode on max)
    ExplodingDice(u32, u32, u32),
    // roll amount, sides, results to get
    HighestDice(u32, u32, u32),
    // roll amount, sides, results to get
    LowestDice(u32, u32, u32),
    Variable(String),
    ParenthesisOpen,
    ParenthesisClose,
}

#[derive(Debug, Clone)]
enum ExprValue {
    Number(f64),
    DiceResult(DiceResult),
    Operator(String),
    Variable(String),
    ParenthesisOpen,
    ParenthesisClose,
}

#[derive(Debug, Clone)]
enum Value {
    Number(f64),
    #[warn(unused_variables)]
    Variable(String),
}

lazy_static! {
    static ref DICE_REGEX: Regex = Regex::new(
         r"^(\d*)d(\d+)(?:!(\d*))?(?:\?([hl])(\d*)?)?$"
    ).unwrap();
}

#[derive(Debug, Clone)]
pub struct DiceResult {
    pub notation: String,
    pub rolls: Vec<u64>,
    pub total: f64,
}

pub struct ComputeResult {
    pub original_notation: String,
    pub tokenized_result: String,
    pub total: f64,
    pub variable_results: Option<HashMap<String, VariableComputeResult>>,
    pub dice_results: Vec<DiceResult>,
}

pub struct VariableComputeResult {
    pub expression: String,
    pub total: f64,
    pub sub_results: Vec<ComputeResult>,
    pub dice_results: Vec<DiceResult>,
}

pub struct Expression {
    pub expression: String,
    pub tokens: Vec<ExprType>,
}

impl Expression {
    pub fn new(expression: String) -> Result<Expression, String> {
        let expr = expression
            .to_lowercase()
            .replace("+", " + ")
            .replace("-", " - ")
            .replace("*", " * ")
            .replace("/", " / ")
            .replace("(", " ( ")
            .replace(")", " ) ")
            .replace("^", " ^ ")
            .replace("%", " % ")
            ;


        let parts = expr.split_whitespace().collect::<Vec<&str>>();
        let mut tokens = Vec::new();

        for part in parts {
            let token = Self::parse_token(part)?;
            tokens.push(token);
        }

        Ok(Expression {
            expression,
            tokens,
        })
    }

    fn parse_token(token: &str) -> Result<ExprType, String> {
        if token == "(" {
            return Ok(ExprType::ParenthesisOpen);
        }
        if token == ")" {
            return Ok(ExprType::ParenthesisClose);
        }

        if token == "+" || token == "-" || token == "*" || token == "/" || token == "^" || token == "%" {
            return Ok(ExprType::Operator(token.to_string()));
        }

        if let Ok(num) = token.parse::<f64>() {
            return Ok(ExprType::Number(num));
        }

        if let Ok(num) = token.parse::<f64>() {
            return Ok(ExprType::Number(num));
        }

        if token.contains('d') {
            if DICE_REGEX.is_match(token) {
                return Self::parse_dice_expr(token);
            }
        }

        Ok(ExprType::Variable(token.to_string()))
    }

    fn parse_dice_expr(statement: &str) -> Result<ExprType, String> {
        if let Some(caps) = DICE_REGEX.captures(statement) {
            let count_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let count = if count_str.is_empty() { 1 } else {
                match count_str.parse::<u32>() {
                    Ok(c) if c > 0 => c,
                    _ => return Err("Invalid dice count".to_string()),
                }
            };

            let sides = match caps.get(2) {
                Some(m) => match m.as_str().parse::<u32>() {
                    Ok(s) if s > 0 => s,
                    _ => return Err("Invalid dice sides".to_string()),
                },
                None => return Err("Missing dice sides".to_string()),
            };

            let has_explode = caps.get(3).is_some();
            let has_modifier = caps.get(4).is_some();

            if has_explode && has_modifier {
                return Err("Cannot have both explode and modifier".to_string());
            }

            // Exploding dice
            if let Some(explode_match) = caps.get(3) {
                let explode_value = explode_match.as_str();
                return if explode_value.is_empty() {
                    Ok(ExprType::ExplodingDice(count, sides, sides))
                } else {
                    match explode_value.parse::<u32>() {
                        Ok(v) if v >= 1 && v <= sides => {
                            Ok(ExprType::ExplodingDice(count, sides, v))
                        }
                        _ => Err(format!("Explode value must be between 1 and {}", sides)),
                    }
                }
            }

            // Highest/Lowest modifier
            if let Some(modifier_match) = caps.get(4) {
                let modifier = modifier_match.as_str();
                let num = if let Some(num_match) = caps.get(5) {
                    let num_str = num_match.as_str();
                    if num_str.is_empty() {
                        1
                    } else {
                        match num_str.parse::<u8>() {
                            Ok(n) if n >= 1 => n,
                            _ => return Err(format!("Invalid number for {}",
                                                    if modifier == "h" { "highest" } else { "lowest" })),
                        }
                    }
                } else {
                    1
                };

                if num > count as u8 {
                    return Err(format!("Cannot take {} {} from {} dice",
                                       num, if modifier == "h" { "highest" } else { "lowest" }, count));
                }

                return match modifier {
                    "h" => Ok(ExprType::HighestDice(count, sides, num as u32)),
                    "l" => Ok(ExprType::LowestDice(count, sides, num as u32)),
                    _ => Err("Invalid modifier".to_string()),
                };
            }

            Ok(ExprType::StandardDice(count, sides))
        } else {
            Err(format!("Invalid dice expression: {}", statement))
        }
    }

    pub fn compute(&self) -> Result<ComputeResult, String> {
        let mut rng = rand::rng();
        let mut variables: HashMap<String, Vec<ExprType>> = HashMap::new();
        let mut main_tokens = Vec::new();

        // Separate variables from main expression
        for token in &self.tokens {
            match token {
                ExprType::Variable(var_name) => {
                    variables.entry(var_name.clone()).or_insert_with(Vec::new);
                    main_tokens.push(ExprType::Variable(var_name.clone()));
                }
                _ => main_tokens.push(token.clone()),
            }
        }

        let (main_result, main_tokenized, main_dice_results) = self.evaluate_expression(&main_tokens, &mut rng)?;

        let variable_results = if !variables.is_empty() {
            let mut var_results = HashMap::new();

            for (var_name, _) in variables {
                let var_tokens = self.extract_variable_tokens(&var_name)?;

                if !var_tokens.is_empty() {
                    let (var_total, var_tokenized, var_dice_results) = self.evaluate_expression(&var_tokens, &mut rng)?;

                    let sub_results = self.compute_variable_subexpressions(&var_tokens)?;

                    var_results.insert(var_name, VariableComputeResult {
                        expression: var_tokenized,
                        total: var_total,
                        sub_results,
                        dice_results: var_dice_results,
                    });
                }
            }

            Some(var_results)
        } else {
            None
        };

        Ok(ComputeResult {
            original_notation: self.expression.clone(),
            tokenized_result: main_tokenized,
            total: main_result,
            variable_results,
            dice_results: main_dice_results,
        })
    }

    fn evaluate_expression(&self, tokens: &[ExprType], rng: &mut impl Rng) -> Result<(f64, String, Vec<DiceResult>), String> {
        let mut output_queue = Vec::new();
        let mut operator_stack = Vec::new();
        let mut all_dice_results = Vec::new();

        for token in tokens {
            match token {
                ExprType::Number(n) => {
                    output_queue.push(ExprValue::Number(*n));
                }
                ExprType::StandardDice(count, sides) => {
                    let dice_result = self.roll_dice(*count, *sides, rng);
                    all_dice_results.push(dice_result.clone());
                    output_queue.push(ExprValue::DiceResult(dice_result));
                }
                ExprType::ExplodingDice(count, sides, explode_on) => {
                    let dice_result = self.roll_exploding_dice(*count, *sides, *explode_on, rng);
                    all_dice_results.push(dice_result.clone());
                    output_queue.push(ExprValue::DiceResult(dice_result));
                }
                ExprType::HighestDice(count, sides, keep) => {
                    let dice_result = self.roll_highest_dice(*count, *sides, *keep, rng);
                    all_dice_results.push(dice_result.clone());
                    output_queue.push(ExprValue::DiceResult(dice_result));
                }
                ExprType::LowestDice(count, sides, keep) => {
                    let dice_result = self.roll_lowest_dice(*count, *sides, *keep, rng);
                    all_dice_results.push(dice_result.clone());
                    output_queue.push(ExprValue::DiceResult(dice_result));
                }
                ExprType::Operator(op) => {
                    while let Some(top) = operator_stack.last() {
                        if let ExprValue::Operator(top_op) = top {
                            if self.precedence(op) <= self.precedence(top_op) {
                                output_queue.push(operator_stack.pop().unwrap());
                            } else {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                    operator_stack.push(ExprValue::Operator(op.clone()));
                }
                ExprType::ParenthesisOpen => {
                    operator_stack.push(ExprValue::ParenthesisOpen);
                }
                ExprType::ParenthesisClose => {
                    while let Some(top) = operator_stack.last() {
                        if let ExprValue::ParenthesisOpen = top {
                            operator_stack.pop();
                            break;
                        }
                        output_queue.push(operator_stack.pop().unwrap());
                    }
                }
                ExprType::Variable(var_name) => {
                    output_queue.push(ExprValue::Variable(var_name.clone()));
                }
            }
        }

        while let Some(op) = operator_stack.pop() {
            output_queue.push(op);
        }

        let mut eval_stack: Vec<Value> = Vec::new();
        let mut expr_stack: Vec<String> = Vec::new();

        for token in output_queue {
            match token {
                ExprValue::Number(n) => {
                    eval_stack.push(Value::Number(n));
                    expr_stack.push(n.to_string());
                }
                ExprValue::DiceResult(dice_result) => {
                    let total = dice_result.total;
                    eval_stack.push(Value::Number(total));
                    // Format: {3d20 [12, 15, 8] = 35}
                    let dice_str = format!("{{{}{} [{}] = {}}}",
                                           dice_result.notation,
                                           if dice_result.rolls.len() > 1 { "" } else { "" },
                                           dice_result.rolls.iter().map(|r| r.to_string()).collect::<Vec<_>>().join(", "),
                                           dice_result.total
                    );
                    expr_stack.push(dice_str);
                }
                ExprValue::Operator(op) => {
                    if eval_stack.len() < 2 {
                        return Err(format!("Not enough operands for operator {}", op));
                    }
                    let right = eval_stack.pop().unwrap();
                    let left = eval_stack.pop().unwrap();

                    let result = match (left, right) {
                        (Value::Number(a), Value::Number(b)) => {
                            match op.as_str() {
                                "+" => Value::Number(a + b),
                                "-" => Value::Number(a - b),
                                "*" => Value::Number(a * b),
                                "/" => {
                                    if b == 0.0 {
                                        return Err("Division by zero".to_string());
                                    }
                                    Value::Number(a / b)
                                }
                                "^" => Value::Number(a.powf(b)),
                                "%" => Value::Number(a % b),
                                _ => return Err(format!("Unknown operator: {}", op)),
                            }
                        }
                        _ => return Err("Type mismatch in operation".to_string()),
                    };

                    eval_stack.push(result);

                    let right_expr = expr_stack.pop().unwrap();
                    let left_expr = expr_stack.pop().unwrap();

                    let need_parens = self.needs_parentheses(&left_expr, &right_expr, &op, &expr_stack);

                    let combined_expr = if need_parens {
                        format!("({} {} {})", left_expr, op, right_expr)
                    } else {
                        format!("{} {} {}", left_expr, op, right_expr)
                    };

                    expr_stack.push(combined_expr);
                }
                ExprValue::Variable(var_name) => {
                    expr_stack.push(format!("{{{}}}", var_name));
                    eval_stack.push(Value::Variable(var_name));
                }
                ExprValue::ParenthesisOpen | ExprValue::ParenthesisClose => {
                    return Err("Unexpected parenthesis in evaluation".to_string());
                }
            }
        }

        if eval_stack.len() != 1 {
            return Err("Invalid expression".to_string());
        }

        let total = match &eval_stack[0] {
            Value::Number(n) => *n,
            Value::Variable(_) => 0.0,
        };

        let final_tokenized = expr_stack.into_iter().next().unwrap_or_else(|| "".to_string());

        Ok((total, final_tokenized, all_dice_results))
    }

    fn needs_parentheses(&self, left_expr: &str, right_expr: &str, op: &str, _: &[String]) -> bool {
        let left_is_complex = left_expr.contains(' ') || left_expr.starts_with('(');
        let right_is_complex = right_expr.contains(' ') || right_expr.starts_with('(');

        match op {
            "+" => {
                if right_expr.trim().starts_with('-') && right_is_complex {
                    true
                } else {
                    false
                }
            }
            "-" => {
                right_is_complex
            }
            "*" | "/" | "%" => {
                (left_expr.contains('+') || left_expr.contains('-')) && left_is_complex ||
                    (right_expr.contains('+') || right_expr.contains('-')) && right_is_complex
            }
            "^" => {
                right_is_complex ||
                    (left_expr.contains('+') || left_expr.contains('-') ||
                        left_expr.contains('*') || left_expr.contains('/')) && left_is_complex
            }
            _ => left_is_complex || right_is_complex,
        }
    }

    fn roll_dice(&self, count: u32, sides: u32, rng: &mut impl Rng) -> DiceResult {
        let mut rolls = Vec::new();
        let mut total = 0;

        for _ in 0..count {
            let roll = rng.random_range(1..=sides) as u64;
            rolls.push(roll);
            total += roll;
        }

        let notation = if count == 1 {
            format!("d{}", sides)
        } else {
            format!("{}d{}", count, sides)
        };

        DiceResult {
            notation,
            rolls,
            total: total as f64,
        }
    }

    fn roll_exploding_dice(&self, count: u32, sides: u32, explode_on: u32, rng: &mut impl Rng) -> DiceResult {
        let mut all_rolls = Vec::new();
        let mut total = 0;

        for _ in 0..count {
            loop {
                let roll = rng.random_range(1..=sides) as u64;
                all_rolls.push(roll);
                total += roll;

                if (roll as u32) < explode_on {
                    break;
                }
            }
        }

        let notation = if explode_on == sides {
            format!("{}d{}!{}", count, sides, if count == 1 { "" } else { "" })
        } else {
            format!("{}d{}!{}", count, sides, explode_on)
        };

        DiceResult {
            notation,
            rolls: all_rolls,
            total: total as f64,
        }
    }

    fn roll_highest_dice(&self, count: u32, sides: u32, keep: u32, rng: &mut impl Rng) -> DiceResult {
        let mut rolls: Vec<u64> = (0..count).map(|_| rng.random_range(1..=sides) as u64).collect();
        rolls.sort_by(|a, b| b.cmp(a));

        let total: u64 = rolls.iter().take(keep as usize).sum();

        let notation = format!("{}d{}?h{}", count, sides, keep);

        DiceResult {
            notation,
            rolls,
            total: total as f64,
        }
    }

    fn roll_lowest_dice(&self, count: u32, sides: u32, keep: u32, rng: &mut impl Rng) -> DiceResult {
        let mut rolls: Vec<u64> = (0..count).map(|_| rng.random_range(1..=sides) as u64).collect();
        rolls.sort();

        let total: u64 = rolls.iter().take(keep as usize).sum();

        let notation = format!("{}d{}?l{}", count, sides, keep);

        DiceResult {
            notation,
            rolls,
            total: total as f64,
        }
    }

    fn extract_variable_tokens(&self, var_name: &str) -> Result<Vec<ExprType>, String> {
        let mut var_tokens = Vec::new();
        let mut i = 0;

        while i < self.tokens.len() {
            if let ExprType::Variable(name) = &self.tokens[i] {
                if name == var_name {
                    let mut j = i + 1;
                    let mut depth = 0;

                    while j < self.tokens.len() {
                        match &self.tokens[j] {
                            ExprType::Variable(_) => break,
                            ExprType::ParenthesisOpen => depth += 1,
                            ExprType::ParenthesisClose => {
                                if depth == 0 {
                                    break;
                                }
                                depth -= 1;
                            }
                            _ => {}
                        }
                        var_tokens.push(self.tokens[j].clone());
                        j += 1;
                    }
                    i = j;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        Ok(var_tokens)
    }

    fn compute_variable_subexpressions(&self, tokens: &[ExprType]) -> Result<Vec<ComputeResult>, String> {
        let mut results = Vec::new();
        let mut rng = rand::rng();
        let mut current_tokens = Vec::new();

        for token in tokens {
            match token {
                ExprType::StandardDice(_, _) | ExprType::ExplodingDice(_, _, _) |
                ExprType::HighestDice(_, _, _) | ExprType::LowestDice(_, _, _) => {
                    if !current_tokens.is_empty() {
                        let (total, tokenized, dice_results) = self.evaluate_expression(&current_tokens, &mut rng)?;
                        results.push(ComputeResult {
                            original_notation: format!("{}", total),
                            tokenized_result: tokenized,
                            total,
                            variable_results: None,
                            dice_results,
                        });
                        current_tokens.clear();
                    }

                    let (total, tokenized, dice_results) = self.evaluate_expression(&[token.clone()], &mut rng)?;
                    results.push(ComputeResult {
                        original_notation: format!("{:?}", token),
                        tokenized_result: tokenized,
                        total,
                        variable_results: None,
                        dice_results,
                    });
                }
                _ => {
                    current_tokens.push(token.clone());
                }
            }
        }

        if !current_tokens.is_empty() {
            let (total, tokenized, dice_results) = self.evaluate_expression(&current_tokens, &mut rng)?;
            results.push(ComputeResult {
                original_notation: format!("{}", total),
                tokenized_result: tokenized,
                total,
                variable_results: None,
                dice_results,
            });
        }

        Ok(results)
    }

    fn precedence(&self, op: &str) -> u8 {
        match op {
            "+" | "-" => 1,
            "*" | "/" | "%" => 2,
            "^" => 3,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod dice_parsing_tests {
        use super::*;

        mod standard_dice {
            use super::*;

            #[test]
            fn test_d6_without_count() {
                let result = Expression::parse_dice_expr("d6").unwrap();
                assert_eq!(result, ExprType::StandardDice(1, 6));
            }

            #[test]
            fn test_1d6() {
                let result = Expression::parse_dice_expr("1d6").unwrap();
                assert_eq!(result, ExprType::StandardDice(1, 6));
            }

            #[test]
            fn test_2d10() {
                let result = Expression::parse_dice_expr("2d10").unwrap();
                assert_eq!(result, ExprType::StandardDice(2, 10));
            }

            #[test]
            fn test_3d20() {
                let result = Expression::parse_dice_expr("3d20").unwrap();
                assert_eq!(result, ExprType::StandardDice(3, 20));
            }

            #[test]
            fn test_10d100() {
                let result = Expression::parse_dice_expr("10d100").unwrap();
                assert_eq!(result, ExprType::StandardDice(10, 100));
            }

            #[test]
            fn test_max_values() {
                let result = Expression::parse_dice_expr("100d1000").unwrap();
                assert_eq!(result, ExprType::StandardDice(100, 1000));
            }
        }

        mod exploding_dice {
            use super::*;

            #[test]
            fn test_exploding_on_max() {
                let result = Expression::parse_dice_expr("1d6!").unwrap();
                assert_eq!(result, ExprType::ExplodingDice(1, 6, 6));
            }

            #[test]
            fn test_exploding_on_custom_value() {
                let result = Expression::parse_dice_expr("1d6!5").unwrap();
                assert_eq!(result, ExprType::ExplodingDice(1, 6, 5));
            }

            #[test]
            fn test_exploding_on_min_value() {
                let result = Expression::parse_dice_expr("1d20!1").unwrap();
                assert_eq!(result, ExprType::ExplodingDice(1, 20, 1));
            }

            #[test]
            fn test_exploding_with_count() {
                let result = Expression::parse_dice_expr("3d10!8").unwrap();
                assert_eq!(result, ExprType::ExplodingDice(3, 10, 8));
            }

            #[test]
            fn test_exploding_without_count() {
                let result = Expression::parse_dice_expr("d20!15").unwrap();
                assert_eq!(result, ExprType::ExplodingDice(1, 20, 15));
            }

            #[test]
            fn test_exploding_double_digit() {
                let result = Expression::parse_dice_expr("2d100!50").unwrap();
                assert_eq!(result, ExprType::ExplodingDice(2, 100, 50));
            }
        }

        mod highest_dice {
            use super::*;

            #[test]
            fn test_highest_single() {
                let result = Expression::parse_dice_expr("2d6?h").unwrap();
                assert_eq!(result, ExprType::HighestDice(2, 6, 1));
            }

            #[test]
            fn test_highest_three() {
                let result = Expression::parse_dice_expr("5d20?h3").unwrap();
                assert_eq!(result, ExprType::HighestDice(5, 20, 3));
            }

            #[test]
            fn test_highest_all() {
                let result = Expression::parse_dice_expr("4d10?h4").unwrap();
                assert_eq!(result, ExprType::HighestDice(4, 10, 4));
            }

            #[test]
            fn test_highest_without_count() {
                let result = Expression::parse_dice_expr("d12?h").unwrap();
                assert_eq!(result, ExprType::HighestDice(1, 12, 1));
            }

            #[test]
            fn test_highest_two_digit() {
                let result = Expression::parse_dice_expr("10d100?h5").unwrap();
                assert_eq!(result, ExprType::HighestDice(10, 100, 5));
            }

            #[test]
            fn test_highest_mid_value() {
                let result = Expression::parse_dice_expr("8d6?h3").unwrap();
                assert_eq!(result, ExprType::HighestDice(8, 6, 3));
            }
        }

        mod lowest_dice {
            use super::*;

            #[test]
            fn test_lowest_single() {
                let result = Expression::parse_dice_expr("2d6?l").unwrap();
                assert_eq!(result, ExprType::LowestDice(2, 6, 1));
            }

            #[test]
            fn test_lowest_three() {
                let result = Expression::parse_dice_expr("5d20?l3").unwrap();
                assert_eq!(result, ExprType::LowestDice(5, 20, 3));
            }

            #[test]
            fn test_lowest_all() {
                let result = Expression::parse_dice_expr("4d10?l4").unwrap();
                assert_eq!(result, ExprType::LowestDice(4, 10, 4));
            }

            #[test]
            fn test_lowest_without_count() {
                let result = Expression::parse_dice_expr("d12?l").unwrap();
                assert_eq!(result, ExprType::LowestDice(1, 12, 1));
            }

            #[test]
            fn test_lowest_two_digit() {
                let result = Expression::parse_dice_expr("10d100?l5").unwrap();
                assert_eq!(result, ExprType::LowestDice(10, 100, 5));
            }

            #[test]
            fn test_lowest_mid_value() {
                let result = Expression::parse_dice_expr("8d6?l3").unwrap();
                assert_eq!(result, ExprType::LowestDice(8, 6, 3));
            }
        }

        mod invalid_inputs {
            use super::*;

            #[test]
            fn test_zero_sides() {
                assert!(Expression::parse_dice_expr("d0").is_err());
                assert!(Expression::parse_dice_expr("1d0").is_err());
                assert!(Expression::parse_dice_expr("10d0").is_err());
            }

            #[test]
            fn test_zero_count() {
                assert!(Expression::parse_dice_expr("0d6").is_err());
                assert!(Expression::parse_dice_expr("0d20").is_err());
            }

            #[test]
            fn test_invalid_explode_value_zero() {
                assert!(Expression::parse_dice_expr("1d6!0").is_err());
            }

            #[test]
            fn test_explode_value_too_high() {
                assert!(Expression::parse_dice_expr("1d6!7").is_err());
                assert!(Expression::parse_dice_expr("1d20!21").is_err());
            }

            #[test]
            fn test_highest_zero() {
                assert!(Expression::parse_dice_expr("2d6?h0").is_err());
            }

            #[test]
            fn test_highest_too_high() {
                assert!(Expression::parse_dice_expr("2d6?h3").is_err());
                assert!(Expression::parse_dice_expr("5d20?h6").is_err());
            }

            #[test]
            fn test_lowest_zero() {
                assert!(Expression::parse_dice_expr("2d6?l0").is_err());
            }

            #[test]
            fn test_lowest_too_high() {
                assert!(Expression::parse_dice_expr("2d6?l3").is_err());
                assert!(Expression::parse_dice_expr("5d20?l6").is_err());
            }

            #[test]
            fn test_invalid_modifier() {
                assert!(Expression::parse_dice_expr("2d6?x").is_err());
                assert!(Expression::parse_dice_expr("2d6?test").is_err());
            }

            #[test]
            fn test_malformed_expression() {
                assert!(Expression::parse_dice_expr("d").is_err());
                assert!(Expression::parse_dice_expr("d!").is_err());
                assert!(Expression::parse_dice_expr("?h").is_err());
                assert!(Expression::parse_dice_expr("").is_err());
            }

            #[test]
            fn test_invalid_characters() {
                assert!(Expression::parse_dice_expr("d6@").is_err());
                assert!(Expression::parse_dice_expr("1d6#").is_err());
            }
        }

        mod combined_modifiers {
            use super::*;

            #[test]
            fn test_exploding_and_highest() {
                assert!(Expression::parse_dice_expr("1d6!?h").is_err());
                assert!(Expression::parse_dice_expr("1d6?h!").is_err());
            }

            #[test]
            fn test_exploding_and_lowest() {
                assert!(Expression::parse_dice_expr("1d6!?l").is_err());
                assert!(Expression::parse_dice_expr("1d6?l!").is_err());
            }
        }
    }

    mod tokenization_tests {
        use super::*;

        #[test]
        fn test_simple_expression() {
            let expr = Expression::new("2d6 + 5".to_string()).unwrap();
            assert_eq!(expr.tokens.len(), 3);
            match &expr.tokens[0] {
                ExprType::StandardDice(2, 6) => (),
                _ => panic!("Expected StandardDice(2,6)"),
            }
            match &expr.tokens[1] {
                ExprType::Operator(op) => assert_eq!(op, "+"),
                _ => panic!("Expected + operator"),
            }
            match &expr.tokens[2] {
                ExprType::Number(5.0) => (),
                _ => panic!("Expected Number(5)"),
            }
        }

        #[test]
        fn test_expression_with_variables() {
            let expr = Expression::new("strength + 2d6".to_string()).unwrap();
            assert_eq!(expr.tokens.len(), 3);
            match &expr.tokens[0] {
                ExprType::Variable(var) => assert_eq!(var, "strength"),
                _ => panic!("Expected Variable"),
            }
        }

        #[test]
        fn test_expression_with_float() {
            let expr = Expression::new("2.5 * 2d6".to_string()).unwrap();
            assert_eq!(expr.tokens.len(), 3);
            match &expr.tokens[0] {
                ExprType::Number(2.5) => (),
                _ => panic!("Expected Float(2.5)"),
            }
        }

        #[test]
        fn test_nested_parentheses() {
            let expr = Expression::new("(2d6 + (3d20 - 5))".to_string()).unwrap();
            assert_eq!(expr.tokens.len(), 9);
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn test_whitespace_handling() {
            let expr = Expression::new("  2d6  +  5  ".to_string()).unwrap();
            assert_eq!(expr.tokens.len(), 3);
        }

        #[test]
        fn test_case_insensitivity() {
            let expr = Expression::new("2D6?H".to_string()).unwrap();
            match &expr.tokens[0] {
                ExprType::HighestDice(2, 6, 1) => (),
                _ => panic!("Expected HighestDice(2,6,1)"),
            }
        }

        #[test]
        fn test_large_numbers() {
            let result = Expression::parse_dice_expr("999d999").unwrap();
            assert_eq!(result, ExprType::StandardDice(999, 999));
        }

        #[test]
        fn test_boundary_values() {
            let result = Expression::parse_dice_expr("1d1").unwrap();
            assert_eq!(result, ExprType::StandardDice(1, 1));

            let result = Expression::parse_dice_expr("1d1!1").unwrap();
            assert_eq!(result, ExprType::ExplodingDice(1, 1, 1));
        }
    }

    mod performance_tests {
        use super::*;

        #[test]
        fn test_many_expressions() {
            let expressions = vec![
                "d6", "2d10", "3d20", "4d6!", "5d8!4", "6d12?h", "7d20?h3",
                "8d6?l", "9d10?l4", "10d100", "1d1000!500", "20d6?h10",
            ];

            for expr in expressions {
                let result = Expression::parse_dice_expr(expr);
                assert!(result.is_ok(), "Failed to parse: {}", expr);
            }
        }
    }

    mod parsing_verification_tests {
        use super::*;

        #[test]
        fn test_basic_arithmetic_with_dice() {
            let expr = Expression::new("2d10 + 3".to_string()).unwrap();

            assert_eq!(expr.tokens.len(), 3);

            match &expr.tokens[0] {
                ExprType::StandardDice(count, sides) => {
                    assert_eq!(*count, 2);
                    assert_eq!(*sides, 10);
                }
                _ => panic!("Expected StandardDice(2,10), got {:?}", expr.tokens[0]),
            }

            match &expr.tokens[1] {
                ExprType::Operator(op) => assert_eq!(op, "+"),
                _ => panic!("Expected Operator(+), got {:?}", expr.tokens[1]),
            }

            match &expr.tokens[2] {
                ExprType::Number(value) => assert_eq!(*value, 3.0),
                _ => panic!("Expected Number(3), got {:?}", expr.tokens[2]),
            }
        }

        #[test]
        fn test_complex_expression_with_all_operators() {
            let expr = Expression::new("2d6 + 5 * 3d8 - 10 / 2".to_string()).unwrap();

            assert_eq!(expr.tokens.len(), 9);

            assert!(matches!(expr.tokens[0], ExprType::StandardDice(2, 6)));
            assert!(matches!(expr.tokens[1], ExprType::Operator(ref op) if op == "+"));
            assert!(matches!(expr.tokens[2], ExprType::Number(5.0)));
            assert!(matches!(expr.tokens[3], ExprType::Operator(ref op) if op == "*"));
            assert!(matches!(expr.tokens[4], ExprType::StandardDice(3, 8)));
            assert!(matches!(expr.tokens[5], ExprType::Operator(ref op) if op == "-"));
            assert!(matches!(expr.tokens[6], ExprType::Number(10.0)));
            assert!(matches!(expr.tokens[7], ExprType::Operator(ref op) if op == "/"));
            assert!(matches!(expr.tokens[8], ExprType::Number(2.0)));
        }

        #[test]
        fn test_expression_with_parentheses() {
            let expr = Expression::new("(2d10 + 3) * 2".to_string()).unwrap();

            assert_eq!(expr.tokens.len(), 7);

            assert!(matches!(expr.tokens[0], ExprType::ParenthesisOpen));
            assert!(matches!(expr.tokens[1], ExprType::StandardDice(2, 10)));
            assert!(matches!(expr.tokens[2], ExprType::Operator(ref op) if op == "+"));
            assert!(matches!(expr.tokens[3], ExprType::Number(3.0)));
            assert!(matches!(expr.tokens[4], ExprType::ParenthesisClose));
            assert!(matches!(expr.tokens[5], ExprType::Operator(ref op) if op == "*"));
            assert!(matches!(expr.tokens[6], ExprType::Number(2.0)));
        }

        #[test]
        fn test_expression_with_exploding_dice() {
            let expr = Expression::new("3d6! + 5".to_string()).unwrap();

            assert_eq!(expr.tokens.len(), 3);

            match &expr.tokens[0] {
                ExprType::ExplodingDice(count, sides, explode_on) => {
                    assert_eq!(*count, 3);
                    assert_eq!(*sides, 6);
                    assert_eq!(*explode_on, 6);
                }
                _ => panic!("Expected ExplodingDice(3,6,6), got {:?}", expr.tokens[0]),
            }

            assert!(matches!(expr.tokens[1], ExprType::Operator(ref op) if op == "+"));
            assert!(matches!(expr.tokens[2], ExprType::Number(5.0)));
        }

        #[test]
        fn test_expression_with_custom_exploding_value() {
            let expr = Expression::new("2d20!15 - 10".to_string()).unwrap();

            assert_eq!(expr.tokens.len(), 3);

            match &expr.tokens[0] {
                ExprType::ExplodingDice(count, sides, explode_on) => {
                    assert_eq!(*count, 2);
                    assert_eq!(*sides, 20);
                    assert_eq!(*explode_on, 15);
                }
                _ => panic!("Expected ExplodingDice(2,20,15), got {:?}", expr.tokens[0]),
            }
        }

        #[test]
        fn test_expression_with_highest_dice() {
            let expr = Expression::new("4d6?h2 * 3".to_string()).unwrap();

            assert_eq!(expr.tokens.len(), 3);

            match &expr.tokens[0] {
                ExprType::HighestDice(count, sides, keep) => {
                    assert_eq!(*count, 4);
                    assert_eq!(*sides, 6);
                    assert_eq!(*keep, 2);
                }
                _ => panic!("Expected HighestDice(4,6,2), got {:?}", expr.tokens[0]),
            }
        }

        #[test]
        fn test_expression_with_lowest_dice() {
            let expr = Expression::new("5d10?l3 + 2d6".to_string()).unwrap();

            assert_eq!(expr.tokens.len(), 3);

            match &expr.tokens[0] {
                ExprType::LowestDice(count, sides, keep) => {
                    assert_eq!(*count, 5);
                    assert_eq!(*sides, 10);
                    assert_eq!(*keep, 3);
                }
                _ => panic!("Expected LowestDice(5,10,3), got {:?}", expr.tokens[0]),
            }
        }

        #[test]
        fn test_expression_with_variables_and_numbers() {
            let expr = Expression::new("strength + 5 * dexterity - 2d8".to_string()).unwrap();

            assert_eq!(expr.tokens.len(), 7);

            match &expr.tokens[0] {
                ExprType::Variable(name) => assert_eq!(name, "strength"),
                _ => panic!("Expected Variable(strength)"),
            }

            assert!(matches!(expr.tokens[1], ExprType::Operator(ref op) if op == "+"));

            match &expr.tokens[2] {
                ExprType::Number(value) => assert_eq!(*value, 5.0),
                _ => panic!("Expected Number(5)"),
            }

            assert!(matches!(expr.tokens[3], ExprType::Operator(ref op) if op == "*"));

            match &expr.tokens[4] {
                ExprType::Variable(name) => assert_eq!(name, "dexterity"),
                _ => panic!("Expected Variable(dexterity)"),
            }

            assert!(matches!(expr.tokens[5], ExprType::Operator(ref op) if op == "-"));

            match &expr.tokens[6] {
                ExprType::StandardDice(count, sides) => {
                    assert_eq!(*count, 2);
                    assert_eq!(*sides, 8);
                }
                _ => panic!("Expected StandardDice(2,8)"),
            }
        }

        #[test]
        fn test_expression_with_floats() {
            let expr = Expression::new("2.5 * 1.5 + 2d6".to_string()).unwrap();

            assert_eq!(expr.tokens.len(), 5);

            match &expr.tokens[0] {
                ExprType::Number(value) => assert_eq!(*value, 2.5),
                _ => panic!("Expected Float(2.5)"),
            }

            match &expr.tokens[2] {
                ExprType::Number(value) => assert_eq!(*value, 1.5),
                _ => panic!("Expected Float(1.5)"),
            }
        }

        #[test]
        fn test_very_complex_expression() {
            let expr = Expression::new("2d6?h + 5 * (3d20!15 - 10) / 2d4".to_string()).unwrap();

            assert!(expr.tokens.len() > 0);

            let mut found_highest = false;
            let mut found_exploding = false;

            for token in expr.tokens.iter() {
                match token {
                    ExprType::HighestDice(2, 6, 1) => found_highest = true,
                    ExprType::ExplodingDice(3, 20, 15) => found_exploding = true,
                    _ => (),
                }
            }

            assert!(found_highest, "Should contain HighestDice(2,6,1)");
            assert!(found_exploding, "Should contain ExplodingDice(3,20,15)");
        }

        #[test]
        fn test_whitespace_variations() {
            let test_cases = vec![
                "2d10+3",
                "2d10 + 3",
                "  2d10  +  3  ",
            ];

            for expr_str in test_cases {
                let expr = Expression::new(expr_str.to_string()).unwrap();
                assert_eq!(expr.tokens.len(), 3);
                assert!(matches!(expr.tokens[0], ExprType::StandardDice(2, 10)));
                assert!(matches!(expr.tokens[1], ExprType::Operator(ref op) if op == "+"));
                assert!(matches!(expr.tokens[2], ExprType::Number(3.0)));
            }
        }

        #[test]
        fn test_multiple_operations() {
            let expr = Expression::new("2d10 + 3d6 - 5 * 2".to_string()).unwrap();
            assert_eq!(expr.tokens.len(), 7);

            for (i, token) in expr.tokens.iter().enumerate() {
                match i {
                    0 => assert!(matches!(token, ExprType::StandardDice(2, 10))),
                    1 => assert!(matches!(token, ExprType::Operator(op) if op == "+")),
                    2 => assert!(matches!(token, ExprType::StandardDice(3, 6))),
                    3 => assert!(matches!(token, ExprType::Operator(op) if op == "-")),
                    4 => assert!(matches!(token, ExprType::Number(5.0))),
                    5 => assert!(matches!(token, ExprType::Operator(op) if op == "*")),
                    6 => assert!(matches!(token, ExprType::Number(2.0))),
                    _ => panic!("Unexpected token at index {}", i),
                }
            }
        }
    }
}