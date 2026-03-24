use std::fmt;

#[derive(Debug, Clone, Copy)]
pub enum Signal {
    Kill,
    Term,
}

#[derive(Debug)]
pub struct KillResult {
    pub port: u16,
    pub pids: Vec<u32>,
    pub error: Option<String>,
}

pub struct PortSpec;

impl PortSpec {
    /// Parse port arguments: supports single ports (3000) and ranges (3000-3010)
    pub fn parse_all(args: &[String]) -> Result<Vec<u16>, PortParseError> {
        let mut ports = Vec::new();
        for arg in args {
            if let Some((start, end)) = arg.split_once('-') {
                let start: u16 = start
                    .trim()
                    .parse()
                    .map_err(|_| PortParseError::Invalid(arg.clone()))?;
                let end: u16 = end
                    .trim()
                    .parse()
                    .map_err(|_| PortParseError::Invalid(arg.clone()))?;
                if start == 0 || end == 0 {
                    return Err(PortParseError::Zero);
                }
                if start > end {
                    return Err(PortParseError::InvalidRange(start, end));
                }
                if (end - start) > 1000 {
                    return Err(PortParseError::RangeTooLarge(start, end));
                }
                for p in start..=end {
                    ports.push(p);
                }
            } else {
                // Could be comma-separated: 3000,3001,3002
                for part in arg.split(',') {
                    let p: u16 = part
                        .trim()
                        .parse()
                        .map_err(|_| PortParseError::Invalid(part.to_string()))?;
                    if p == 0 {
                        return Err(PortParseError::Zero);
                    }
                    ports.push(p);
                }
            }
        }
        Ok(ports)
    }
}

#[derive(Debug)]
pub enum PortParseError {
    Invalid(String),
    Zero,
    InvalidRange(u16, u16),
    RangeTooLarge(u16, u16),
}

impl fmt::Display for PortParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(s) => write!(f, "invalid port: '{s}'"),
            Self::Zero => write!(f, "port 0 is not valid"),
            Self::InvalidRange(s, e) => write!(f, "invalid range: {s} > {e}"),
            Self::RangeTooLarge(s, e) => write!(f, "range {s}-{e} too large (max 1000 ports)"),
        }
    }
}

impl std::error::Error for PortParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_port() {
        let args = vec!["3000".to_string()];
        let ports = PortSpec::parse_all(&args).unwrap();
        assert_eq!(ports, vec![3000]);
    }

    #[test]
    fn parse_multiple_ports() {
        let args = vec!["3000".to_string(), "8080".to_string()];
        let ports = PortSpec::parse_all(&args).unwrap();
        assert_eq!(ports, vec![3000, 8080]);
    }

    #[test]
    fn parse_port_range() {
        let args = vec!["3000-3003".to_string()];
        let ports = PortSpec::parse_all(&args).unwrap();
        assert_eq!(ports, vec![3000, 3001, 3002, 3003]);
    }

    #[test]
    fn parse_comma_separated() {
        let args = vec!["3000,3001,3002".to_string()];
        let ports = PortSpec::parse_all(&args).unwrap();
        assert_eq!(ports, vec![3000, 3001, 3002]);
    }

    #[test]
    fn parse_mixed() {
        let args = vec!["3000".to_string(), "4000-4002".to_string(), "5000,5001".to_string()];
        let ports = PortSpec::parse_all(&args).unwrap();
        assert_eq!(ports, vec![3000, 4000, 4001, 4002, 5000, 5001]);
    }

    #[test]
    fn reject_zero_port() {
        let args = vec!["0".to_string()];
        assert!(PortSpec::parse_all(&args).is_err());
    }

    #[test]
    fn reject_invalid_port() {
        let args = vec!["abc".to_string()];
        assert!(PortSpec::parse_all(&args).is_err());
    }

    #[test]
    fn reject_reversed_range() {
        let args = vec!["5000-3000".to_string()];
        assert!(PortSpec::parse_all(&args).is_err());
    }

    #[test]
    fn reject_huge_range() {
        let args = vec!["1-5000".to_string()];
        assert!(PortSpec::parse_all(&args).is_err());
    }
}
