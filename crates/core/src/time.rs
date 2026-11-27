pub fn format_time(seconds: f64) -> String {
    let hours = (seconds / 3600.0).floor();
    let minutes = ((seconds % 3600.0) / 60.0).floor();
    let whole_seconds = (seconds % 60.0).floor();
    let hundredths = ((seconds % 1.0) * 100.0).floor();

    format!("{hours:02.0}:{minutes:02.0}:{whole_seconds:02.0}.{hundredths:02.0}")
}

pub fn parse_time_input(input: &str) -> Option<f64> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let parts: Vec<_> = trimmed.split(':').collect();
    if !(1..=3).contains(&parts.len()) {
        return None;
    }

    let numbers: Option<Vec<f64>> = parts
        .iter()
        .map(|part| {
            let value = part.trim();
            if value.is_empty() {
                return None;
            }
            let number = value.parse::<f64>().ok()?;
            (number >= 0.0).then_some(number)
        })
        .collect();
    let numbers = numbers?;

    Some(match numbers.as_slice() {
        [seconds] => *seconds,
        [minutes, seconds] => minutes * 60.0 + seconds,
        [hours, minutes, seconds] => hours * 3600.0 + minutes * 60.0 + seconds,
        _ => unreachable!("the number of parts was checked above"),
    })
}

#[cfg(test)]
mod tests {
    use super::{format_time, parse_time_input};

    #[test]
    fn parses_the_supported_forms() {
        assert_eq!(parse_time_input("90"), Some(90.0));
        assert_eq!(parse_time_input("90.5"), Some(90.5));
        assert_eq!(parse_time_input("01:30"), Some(90.0));
        assert_eq!(parse_time_input("1:30.5"), Some(90.5));
        // Los dos casos del oráculo que faltaban (time.test.ts).
        assert_eq!(parse_time_input("00:01:30"), Some(90.0));
        assert_eq!(parse_time_input("01:01:01"), Some(3661.0));
        assert_eq!(parse_time_input("01:01:01.001"), Some(3661.001));
    }

    #[test]
    fn rejects_empty_and_invalid_input() {
        for input in ["", "   ", "abc", "01:", ":01", "01:02:03:04"] {
            assert_eq!(parse_time_input(input), None, "{input:?}");
        }
    }

    #[test]
    fn roundtrips_displayed_times() {
        for seconds in [0.0, 1.5, 90.5, 3661.001] {
            let formatted = format_time(seconds);
            let parsed = parse_time_input(&formatted).expect("formatted time parses");
            assert!((parsed - seconds).abs() < 0.01, "{seconds} -> {formatted}");
        }
    }

    #[test]
    fn keeps_decimal_seconds_as_seconds() {
        assert_eq!(parse_time_input("1.5"), Some(1.5));
        assert_eq!(parse_time_input("00:01:30.50"), Some(90.5));
    }
}
