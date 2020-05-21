use super::{
    system_profile::{self, DeveloperTools},
    xcode_plugin,
};
use crate::{
    opts::{Clobbering, Interactivity},
    util::{self, cli::TextWrapper},
};
use std::fmt::{self, Display};

#[derive(Debug)]
pub enum Error {
    XcodeGenPresenceCheckFailed(bossy::Error),
    XcodeGenInstallFailed(bossy::Error),
    IosDeployPresenceCheckFailed(bossy::Error),
    IosDeployInstallFailed(bossy::Error),
    VersionLookupFailed(system_profile::Error),
    XcodeRustPluginInstallFailed(xcode_plugin::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::XcodeGenPresenceCheckFailed(err) => {
                write!(f, "Failed to check for presence of `xcodegen`: {}", err)
            }
            Self::XcodeGenInstallFailed(err) => write!(f, "Failed to install `xcodegen`: {}", err),
            Self::IosDeployPresenceCheckFailed(err) => {
                write!(f, "Failed to check for presence of `ios-deploy`: {}", err)
            }
            Self::IosDeployInstallFailed(err) => {
                write!(f, "Failed to install `ios-deploy`: {}", err)
            }
            Self::VersionLookupFailed(err) => write!(f, "{}", err),
            Self::XcodeRustPluginInstallFailed(err) => write!(f, "{}", err),
        }
    }
}

pub fn install(
    wrapper: &TextWrapper,
    interactivity: Interactivity,
    clobbering: Clobbering,
) -> Result<(), Error> {
    let xcodegen_found =
        util::command_present("xcodegen").map_err(Error::XcodeGenPresenceCheckFailed)?;
    if !xcodegen_found || clobbering.allowed() {
        bossy::Command::impure("brew")
            // reinstall works even if it's not installed yet,
            // and will upgrade if it's already installed!
            .with_args(&["reinstall", "xcodegen"])
            .run_and_wait()
            .map_err(Error::XcodeGenInstallFailed)?;
    }
    let ios_deploy_found =
        util::command_present("ios-deploy").map_err(Error::IosDeployPresenceCheckFailed)?;
    if !ios_deploy_found || clobbering.allowed() {
        bossy::Command::impure("brew")
            .with_args(&["reinstall", "ios-deploy"])
            .run_and_wait()
            .map_err(Error::IosDeployInstallFailed)?;
    }
    // we definitely don't want to install this on CI...
    if interactivity.full() {
        let tool_info = DeveloperTools::new().map_err(Error::VersionLookupFailed)?;
        xcode_plugin::install(wrapper, interactivity, clobbering, tool_info.version)
            .map_err(Error::XcodeRustPluginInstallFailed)?;
    }
    Ok(())
}