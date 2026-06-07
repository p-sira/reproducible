//! Environment context.

use std::fmt;
use std::process::Command;

/// Testing environment context data container.
///
/// Stores details about the hardware and software configuration under which the tests
/// or benchmarks were executed.
///
/// It is generally recommended to use the `current_env!()` macro instead of calling this directly,
/// so the crate name and version are automatically populated from the environment.
///
/// # Examples
///
/// ```
/// use reproducible::env::Env;
///
/// let env = Env {
///     rust_version: "1.90.0".to_string(),
///     platform: "x86_64-unknown-linux-gnu".to_string(),
///     package_name: "ellip".to_string(),
///     package_version: "1.1.0".to_string(),
///     cpu: "AMD Ryzen 5 4600H with Radeon Graphics".to_string(),
///     clock_speed: 3_000_000_000,
///     total_memory: 16_000_000_000,
/// };
///
/// assert_eq!(
///     env.to_string(),
///     "AMD Ryzen 5 4600H with Radeon Graphics @3.0 GHz RAM 16 GB running x86_64-unknown-linux-gnu rustc 1.90.0 using ellip v1.1.0"
/// );
/// ```
#[derive(Debug, Clone)]
pub struct Env {
    pub rust_version: String,
    /// Target triple (e.g., `x86_64-unknown-linux-gnu`)
    pub platform: String,
    pub package_name: String,
    pub package_version: String,
    pub cpu: String,
    /// Clock speed in Hz
    pub clock_speed: u64,
    /// Total memory in bytes
    pub total_memory: u64,
}

#[macro_export]
/// Create [Env] from the current environment.
///
/// This macro automatically populates [Env] with the crate name and version
/// from the environment.
macro_rules! current_env {
    () => {
        $crate::env::Env::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    };
}

impl Env {
    /// Creates a new `Env` by querying system information and `rustc`.
    ///
    /// # Examples
    ///
    /// ```
    /// use reproducible::current_env;
    ///
    /// let mut env = current_env!();
    /// env.clock_speed = 0; // Overriding fields if necessary
    /// assert_eq!(env.package_name, "reproducible"); // Since we're executing inside reproducible's tests
    /// ```
    pub fn new(package_name: impl Into<String>, package_version: impl Into<String>) -> Self {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();
        let cpu = match sys.cpus().first() {
            Some(cpu) => cpu.brand(),
            None => "Unknown CPU",
        }
        .to_string();

        let clock_speed = sys
            .cpus()
            .first()
            .map_or(0, |cpu| cpu.frequency() * 1_000_000);

        let rust_version = Command::new("rustc")
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .and_then(|s| s.split_whitespace().nth(1).map(|s| s.to_owned()))
            .unwrap_or_else(|| "unknown".to_string());

        let total_memory = sys.total_memory();

        let platform = Command::new("rustc")
            .arg("-vV")
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .and_then(|s| {
                s.lines()
                    .find(|line| line.starts_with("host:"))
                    .and_then(|line| line.strip_prefix("host: "))
                    .map(|s| s.to_owned())
            })
            .unwrap_or_else(|| format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS));

        Env {
            rust_version,
            platform,
            package_name: package_name.into(),
            package_version: package_version.into(),
            cpu,
            clock_speed,
            total_memory,
        }
    }
}

impl fmt::Display for Env {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cpu_str = format_cpu_with_clock_speed(&self.cpu, self.clock_speed);
        write!(
            f,
            "{} RAM {} GB running {} rustc {} using {} v{}",
            cpu_str,
            self.total_memory / 1_000_000_000,
            self.platform,
            self.rust_version,
            self.package_name,
            self.package_version
        )
    }
}

/// Formats a CPU model name and its clock speed into a human-readable string.
///
/// ```
/// use reproducible::env::format_cpu_with_clock_speed;
///
/// assert_eq!(
///     format_cpu_with_clock_speed("Intel Core i5", 800_000_000),
///     "Intel Core i5 @800 MHz"
/// );
/// ```
pub fn format_cpu_with_clock_speed(cpu: &str, clock_speed: u64) -> String {
    if clock_speed == 0 {
        return cpu.to_string();
    }

    if clock_speed > 1_000_000_000 {
        format!("{} @{:.1} GHz", cpu, (clock_speed as f64) / 1_000_000_000.0)
    } else {
        format!("{} @{} MHz", cpu, clock_speed / 1_000_000)
    }
}
