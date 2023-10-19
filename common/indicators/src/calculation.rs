use anyhow::{bail, Error};

pub fn simple_moving_average(values: &[f64], period: u64) -> Result<Vec<f64>, Error> {
    if values.is_empty() || values.len() < period as usize {
        bail!("Moving average period too long for this set of values.")
    }
    let mut tail_pointer = 0usize;
    let mut head_pointer = period as usize;
    let mut sma_values = vec![0.0; period as usize];
    while head_pointer <= values.len() {
        let range_avg = average(&values[tail_pointer..head_pointer]);
        sma_values.push(range_avg);
        tail_pointer += 1;
        head_pointer += 1;
    }
    Ok(sma_values)
}

pub fn exponential_moving_average(values: &[f64], period: u64) -> Result<Vec<f64>, Error> {
    if values.is_empty() || values.len() < period as usize {
        bail!("Exponential moving average period too long for this set of values.")
    }
    let mut ema_values = Vec::new();
    if values.len() >= period as usize {
        let sma: f64 = values.iter().take(period as usize).sum::<f64>() / period as f64;
        ema_values.push(sma);

        let multiplier: f64 = 2.0 / ((period as f64) + 1.0);
        for price in values.iter().skip(period as usize) {
            let ema_previous = *ema_values.last().unwrap();
            let ema_current = ((price - ema_previous) * multiplier) + ema_previous;
            ema_values.push(ema_current);
        }
    }
    Ok(ema_values)
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
        let result = simple_moving_average(&values, 7).expect("Error during moving average calculation");
        assert_eq!(result.len(), 12);
        assert_eq!(result.first().unwrap(), &0.0);
        assert_eq!(result.get(1).unwrap(), &0.0);
        assert_eq!(result.get(2).unwrap(), &0.0);
        assert_eq!(result.get(3).unwrap(), &0.0);
        assert_eq!(result.get(4).unwrap(), &0.0);
        assert_eq!(result.get(5).unwrap(), &0.0);
        assert_eq!(result.get(6).unwrap(), &0.0);
        assert_eq!(result.get(7).unwrap(), &3.0);
        assert_eq!(result.get(8).unwrap(), &4.0);
        assert_eq!(result.get(9).unwrap(), &5.0);
        assert_eq!(result.get(10).unwrap(), &6.0);
        assert_eq!(result.get(11).unwrap(), &7.0);
    }
}
