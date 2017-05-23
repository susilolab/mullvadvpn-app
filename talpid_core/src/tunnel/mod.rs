use net;
use openvpn_ffi::OpenVpnPluginEvent;
use process::openvpn::OpenVpnCommand;

/// A module for all OpenVPN related tunnel management.
pub mod openvpn;

use self::openvpn::{OpenVpnEvent, OpenVpnMonitor};

mod errors {
    error_chain!{
        errors {
            /// An error indicating there was an error listening for events from the VPN tunnel.
            TunnelMonitoringError {
                description("Error while setting up or processing events from the VPN tunnel")
            }
            /// An error indicating that there was an error when trying to start up a VPN tunnel.
            TunnelStartError {
                description("Error while trying to start the tunnel")
            }
        }
    }
}
pub use self::errors::*;


/// Possible events from the VPN tunnel and the child process managing it.
pub enum TunnelEvent {
    /// Sent when the tunnel comes up and is ready for traffic.
    Up,
    /// Sent when the tunnel goes down.
    Down,
    /// Sent when the process managing the tunnel exits.
    Shutdown,
}

impl TunnelEvent {
    /// Converts an `OpenVpnEvent` to a `TunnelEvent`.
    /// Returns `None` if there is no corresponding `TunnelEvent`.
    pub fn from_openvpn_event(event: &OpenVpnEvent) -> Option<TunnelEvent> {
        match *event {
            OpenVpnEvent::PluginEvent(ref event, _) => Self::from_openvpn_plugin_event(event),
            OpenVpnEvent::Shutdown(_) => Some(TunnelEvent::Shutdown),
        }
    }

    fn from_openvpn_plugin_event(event: &OpenVpnPluginEvent) -> Option<TunnelEvent> {
        match *event {
            OpenVpnPluginEvent::Up => Some(TunnelEvent::Up),
            OpenVpnPluginEvent::RoutePredown => Some(TunnelEvent::Down),
            _ => None,
        }
    }
}


/// Abstraction for monitoring a generic VPN tunnel.
pub struct TunnelMonitor {
    monitor: OpenVpnMonitor,
}

impl TunnelMonitor {
    /// Creates a new `TunnelMonitor` with the given event callback.
    pub fn new<L>(on_event: L) -> Result<Self>
        where L: Fn(TunnelEvent) + Send + Sync + 'static
    {
        let on_openvpn_event = move |openvpn_event| {
            // FIXME: This comment must be here to make rustfmt 0.8.3 not screw up.
            match TunnelEvent::from_openvpn_event(&openvpn_event) {
                Some(tunnel_event) => on_event(tunnel_event),
                None => debug!("Ignoring OpenVpnEvent {:?}", openvpn_event),
            }
        };
        let monitor = openvpn::OpenVpnMonitor::new(on_openvpn_event, get_plugin_path())
            .chain_err(|| ErrorKind::TunnelMonitoringError)?;
        Ok(TunnelMonitor { monitor })
    }

    /// Tries to start a VPN tunnel towards the given address. Will fail if there is a tunnel
    /// running already.
    pub fn start(&self, remote: net::RemoteAddr) -> Result<()> {
        let mut cmd = OpenVpnCommand::new("openvpn");
        cmd.config(get_config_path()).remotes(remote).unwrap();
        self.monitor.start(cmd).chain_err(|| ErrorKind::TunnelStartError)
    }
}


// TODO(linus): Temporary implementation for getting plugin path during development.
fn get_plugin_path() -> &'static str {
    if cfg!(all(unix, not(target_os = "macos"))) {
        "./target/debug/libtalpid_openvpn_plugin.so"
    } else if cfg!(target_os = "macos") {
        "./target/debug/libtalpid_openvpn_plugin.dylib"
    } else if cfg!(windows) {
        "./target/debug/libtalpid_openvpn_plugin.dll"
    } else {
        panic!("Unsupported platform");
    }
}

// TODO(linus): Temporary implementation for getting hold of a config location.
// Manually place a working config here or change this string in order to test
fn get_config_path() -> &'static str {
    "./openvpn.conf"
}