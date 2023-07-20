pub fn size_to_string(size: usize) -> String {
    match size {
        gb if gb >= 1000 * 1000 * 1000 => format!("{:.2} GB", gb as f64 / 1000000000.0),
        mb if mb >= 1000 * 1000 => format!("{:.2} MB", mb as f64 / 1000000.0),
        kb if kb >= 1000 => format!("{:.2} KB", kb as f64 / 1000.0),
        b => format!("{} B", b),
    }
}

pub fn speed_to_string(size: usize) -> String {
    let speed = size_to_string(size);
    format!("{}/s", speed)
}

pub fn percent_to_string(percent: f64) -> String {
    format!("{:.1}%", percent * 100.0)
}

pub fn eta_to_string(eta: usize) -> String {
    let days = eta / (60 * 60 * 24);
    let hours = (eta - days * 60 * 60 * 24) / (60 * 60);
    let minutes = (eta - days * 60 * 60 * 24 - hours * 60 * 60) / 60;
    let seconds = eta - days * 60 * 60 * 24 - hours * 60 * 60 - minutes * 60;

    match (days, hours, minutes, seconds) {
        (d, _, _, _) if d != 0 => format!(">= {}d", d),
        (_, h, _, _) if h != 0 => format!(">= {}h", h),
        (_, _, m, s) if m != 0 => format!("{}m {}s", m, s),
        (_, _, _, s) => format!("{}s", s),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_size_to_string() {
        assert_eq!("1.20 MB", size_to_string(1200000));
        assert_eq!("1.22 GB", size_to_string(1220000170));
        assert_eq!("120 B", size_to_string(120));
    }

    #[test]
    fn test_percent_to_string() {
        assert_eq!("22.4%", percent_to_string(0.2242));
    }

    #[test]
    fn test_eta_to_string() {
        assert_eq!("1m 40s", eta_to_string(100));
        assert_eq!(">= 2h", eta_to_string(2 * 60 * 60 + 500));
        assert_eq!(">= 1d", eta_to_string(24 * 60 * 60 + 666));
    }
}
