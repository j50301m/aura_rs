use anyhow::{Result, anyhow};
use rand::Rng;
use std::collections::HashSet;

/// Draw numbers with optional repetition
///
/// # Parameters
/// - `min`: Minimum number (inclusive, must be positive)
/// - `max`: Maximum number (inclusive, must be positive)
/// - `count`: Number of numbers to draw (must be positive)
/// - `repeatable`: Whether numbers can be repeated (true = can repeat, false = no repeat)
///
/// # Returns
/// A comma-separated string of drawn numbers
pub fn draw_number(min: i16, max: i16, count: i32, repeatable: bool) -> Result<String> {
    validate_params(min, max, count, repeatable)?;

    let numbers = if repeatable {
        draw_with_repeat(min, max, count)
    } else {
        draw_no_repeat(min, max, count)
    };

    Ok(numbers_to_string(&numbers))
}

/// Draw numbers without repetition
fn draw_no_repeat(min: i16, max: i16, count: i32) -> Vec<i16> {
    let mut result = Vec::new();
    let mut used = HashSet::new();
    let mut rng = rand::rng();

    while result.len() < count as usize {
        let num = rng.random_range(min..=max);
        if !used.contains(&num) {
            result.push(num);
            used.insert(num);
        }
    }

    result
}

/// Draw numbers with repetition allowed
fn draw_with_repeat(min: i16, max: i16, count: i32) -> Vec<i16> {
    let mut result = Vec::with_capacity(count as usize);
    let mut rng = rand::rng();

    for _ in 0..count {
        let num = rng.random_range(min..=max);
        result.push(num);
    }

    result
}

/// Convert vector of numbers to comma-separated string
fn numbers_to_string(numbers: &[i16]) -> String {
    numbers
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

/// Validate input parameters
fn validate_params(min: i16, max: i16, count: i32, repeatable: bool) -> Result<()> {
    if min <= 0 {
        return Err(anyhow!("min must be > 0"));
    }

    if max <= 0 {
        return Err(anyhow!("max must be > 0"));
    }

    if count <= 0 {
        return Err(anyhow!("count must be > 0"));
    }

    if max <= min {
        return Err(anyhow!("max must be > min"));
    }

    if !repeatable {
        let range_size = (max - min + 1) as i32;
        if count > range_size {
            return Err(anyhow!(
                "count ({}) cannot be greater than range size ({}) when repeatable is false",
                count,
                range_size
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_with_repeat() {
        let result = draw_number(1, 10, 5, true).unwrap();
        let numbers: Vec<&str> = result.split(',').collect();
        assert_eq!(numbers.len(), 5);

        // All numbers should be in range
        for num_str in numbers {
            let num: i64 = num_str.parse().unwrap();
            assert!(num >= 1 && num <= 10);
        }
    }

    #[test]
    fn test_draw_no_repeat() {
        let result = draw_number(1, 10, 5, false).unwrap();
        let numbers: Vec<&str> = result.split(',').collect();
        assert_eq!(numbers.len(), 5);

        // All numbers should be unique
        let mut unique_numbers = HashSet::new();
        for num_str in &numbers {
            let num: i64 = num_str.parse().unwrap();
            assert!(num >= 1 && num <= 10);
            assert!(unique_numbers.insert(num)); // Should be unique
        }
    }

    #[test]
    fn test_validation() {
        // Test min <= 0
        assert!(draw_number(0, 10, 5, true).is_err());
        assert!(draw_number(-1, 10, 5, true).is_err());

        // Test max <= 0
        assert!(draw_number(1, 0, 5, true).is_err());
        assert!(draw_number(1, -1, 5, true).is_err());

        // Test count <= 0
        assert!(draw_number(1, 10, 0, true).is_err());
        assert!(draw_number(1, 10, -1, true).is_err());

        // Test max <= min
        assert!(draw_number(10, 10, 5, true).is_err());
        assert!(draw_number(10, 5, 5, true).is_err());

        // Test count > range when no repeat
        assert!(draw_number(1, 5, 10, false).is_err());
    }
    #[test]
    fn test_numbers_to_string() {
        let numbers = vec![1, 2, 3, 4, 5];
        assert_eq!(numbers_to_string(&numbers), "1,2,3,4,5");
    }
}
