use anyhow::{bail, Error};

pub fn moving_average(values: &[f64], length: u16) -> Result<Vec<f64>, Error> {
    if values.is_empty() || values.len() < length as usize {
        bail!("Moving average length too long for this set of values.")
    }
    let mut tail_pointer = 0usize;
    let mut head_pointer = length as usize;
    let mut result = Vec::new();
    while head_pointer <= values.len() {
        let range_avg = average(&values[tail_pointer..head_pointer]);
        result.push(range_avg);
        tail_pointer += 1;
        head_pointer += 1;
    }
    Ok(result)
}

fn average(values: &[f64]) -> f64 {
    let sum: f64 = values.iter().sum();
    sum / values.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moving_average() {
        let values = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let result = moving_average(&values, 7).expect("Error during moving average calculation");
        assert_eq!(result.len(), 5);
        assert_eq!(result.first().unwrap(), &3.0);
        assert_eq!(result.get(1).unwrap(), &4.0);
        assert_eq!(result.get(2).unwrap(), &5.0);
        assert_eq!(result.get(3).unwrap(), &6.0);
        assert_eq!(result.get(4).unwrap(), &7.0);
    }
}
