// Copyright (c) 2016, 2018, 2021 vergen developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `vergen` build feature implementation

use crate::config::{Config, Instructions};
#[cfg(feature = "build")]
use {
    crate::{
        config::VergenKey,
        feature::{add_entry, TimeZone, TimestampKind},
    },
    getset::{Getters, MutGetters},
    std::env,
    time::{format_description, macros::format_description, OffsetDateTime},
};

/// Configuration for the `VERGEN_BUILD_*` instructions
///
/// # Instructions
/// The following instructions can be generated:
///
/// | Instruction | Default |
/// | ----------- | :-----: |
/// | `cargo:rustc-env=VERGEN_BUILD_DATE=2021-02-12` | |
/// | `cargo:rustc-env=VERGEN_BUILD_TIME=11:22:34` | |
/// | `cargo:rustc-env=VERGEN_BUILD_TIMESTAMP=2021-02-12T01:54:15.134750+00:00` | * |
/// | `cargo:rustc-env=VERGEN_BUILD_SEMVER=4.2.0` | * |
///
/// * If the `timestamp` field is false, the date/time instructions will not be generated.
/// * If the `semver` field is false, the semver instruction will not be generated.
/// * **NOTE** - By default, the date/time related instructions will use [`UTC`](TimeZone::Utc).
/// * **NOTE** - The date/time instruction output is determined by the [`kind`](TimestampKind) field and can be any combination of the three.
///
/// # Example
///
/// ```
/// # use anyhow::Result;
/// use vergen::{vergen, Config};
#[cfg_attr(feature = "build", doc = r##"use vergen::{TimestampKind, TimeZone};"##)]
///
/// # pub fn main() -> Result<()> {
/// let mut config = Config::default();
#[cfg_attr(
    feature = "build",
    doc = r##"
// Generate all three date/time instructions
*config.build_mut().kind_mut() = TimestampKind::All;
// Change the date/time instructions to show `Local` time
*config.build_mut().timezone_mut() = TimeZone::Local;

// Generate the instructions
vergen(config)?;
"##
)]
/// # Ok(())
/// # }
#[cfg(feature = "build")]
#[derive(Clone, Copy, Debug, Getters, MutGetters)]
#[getset(get = "pub(crate)", get_mut = "pub")]
pub struct Build {
    /// Enable/Disable the build output
    enabled: bool,
    /// Enable/Disable the `VERGEN_BUILD_DATE`, `VERGEN_BUILD_TIME`, and `VERGEN_BUILD_TIMESTAMP` instructions.
    timestamp: bool,
    /// The timezone to use for the date/time instructions.
    timezone: TimeZone,
    /// The kind of date/time instructions to output.
    kind: TimestampKind,
    /// Enable/Disable the `VERGEN_BUILD_SEMVER` instruction.
    semver: bool,
}

#[cfg(feature = "build")]
impl Default for Build {
    fn default() -> Self {
        Self {
            enabled: true,
            timestamp: true,
            timezone: TimeZone::Utc,
            kind: TimestampKind::Timestamp,
            semver: true,
        }
    }
}

#[cfg(feature = "build")]
impl Build {
    pub(crate) fn has_enabled(self) -> bool {
        self.enabled && (self.timestamp || self.semver)
    }
}

#[cfg(feature = "build")]
pub(crate) fn configure_build(instructions: &Instructions, config: &mut Config) {
    let build_config = instructions.build();

    if build_config.has_enabled() {
        if *build_config.timestamp() {
            match build_config.timezone() {
                TimeZone::Utc => {
                    add_config_entries(config, *build_config, &OffsetDateTime::now_utc());
                }
                TimeZone::Local => {
                    add_config_entries(
                        config,
                        *build_config,
                        &OffsetDateTime::now_local().expect("unable to retrieve local datetime"),
                    );
                }
            };
        }

        if *build_config.semver() {
            add_entry(
                config.cfg_map_mut(),
                VergenKey::BuildSemver,
                env::var("CARGO_PKG_VERSION").ok(),
            );
        }
    }
}

#[cfg(feature = "build")]
fn add_config_entries(config: &mut Config, build_config: Build, now: &OffsetDateTime) {
    match build_config.kind() {
        TimestampKind::DateOnly => add_date_entry(config, now),
        TimestampKind::TimeOnly => add_time_entry(config, now),
        TimestampKind::DateAndTime => {
            add_date_entry(config, now);
            add_time_entry(config, now);
        }
        TimestampKind::Timestamp => add_timestamp_entry(config, now),
        TimestampKind::All => {
            add_date_entry(config, now);
            add_time_entry(config, now);
            add_timestamp_entry(config, now);
        }
    }
}

#[cfg(feature = "build")]
fn add_date_entry(config: &mut Config, now: &OffsetDateTime) {
    add_entry(
        config.cfg_map_mut(),
        VergenKey::BuildDate,
        now.format(format_description!("[year]-[month]-[day]")).ok(),
    );
}

#[cfg(feature = "build")]
fn add_time_entry(config: &mut Config, now: &OffsetDateTime) {
    add_entry(
        config.cfg_map_mut(),
        VergenKey::BuildTime,
        now.format(format_description!("[hour]-[minute]-[second]"))
            .ok(),
    );
}

#[cfg(feature = "build")]
fn add_timestamp_entry(config: &mut Config, now: &OffsetDateTime) {
    add_entry(
        config.cfg_map_mut(),
        VergenKey::BuildTimestamp,
        now.format(&format_description::well_known::Rfc3339).ok(),
    );
}

#[cfg(not(feature = "build"))]
pub(crate) fn configure_build(_instructions: &Instructions, _config: &mut Config) {}

#[cfg(all(test, feature = "build"))]
mod test {
    use crate::{
        config::Instructions,
        feature::{TimeZone, TimestampKind},
    };

    #[test]
    fn build_config() {
        let mut config = Instructions::default();
        assert!(config.build().timestamp());
        assert_eq!(config.build().timezone(), &TimeZone::Utc);
        assert_eq!(config.build().kind(), &TimestampKind::Timestamp);
        *config.build_mut().kind_mut() = TimestampKind::All;
        assert_eq!(config.build().kind(), &TimestampKind::All);
    }

    #[test]
    fn not_enabled() {
        let mut config = Instructions::default();
        *config.build_mut().enabled_mut() = false;
        assert!(!config.build().has_enabled());
    }

    #[test]
    fn no_timestamp() {
        let mut config = Instructions::default();
        *config.build_mut().timestamp_mut() = false;
        assert!(config.build().has_enabled());
    }

    #[test]
    fn nothing() {
        let mut config = Instructions::default();
        *config.build_mut().timestamp_mut() = false;
        *config.build_mut().semver_mut() = false;
        assert!(!config.build().has_enabled());
    }
}

#[cfg(all(test, not(feature = "build")))]
mod test {}
