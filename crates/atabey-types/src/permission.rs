use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use specta::Type;

/// Something an extension must declare in its manifest before the core allows it (R-33, R-34).
#[derive(Serialize, Deserialize, Type, JsonSchema, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum Permission {
    /// Call a Ulak command, e.g. `defter.query`.
    UlakCommand(String),
    /// Receive a Ulak event, e.g. `defter.appended`.
    UlakSubscribe(String),
    /// Publish a Ulak event.
    UlakEmit(String),
    /// Contribute a UI panel to a slot.
    UiPanel(PanelSlot),
    /// Read files inside the open project.
    ReadProjectFiles,
    /// Make network requests to matching hosts.
    Network(HostPattern),
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UlakCommand(name) => write!(f, "ulak.command:{name}"),
            Self::UlakSubscribe(name) => write!(f, "ulak.subscribe:{name}"),
            Self::UlakEmit(name) => write!(f, "ulak.emit:{name}"),
            Self::UiPanel(slot) => write!(f, "ui.panel:{slot}"),
            Self::ReadProjectFiles => f.write_str("project.read"),
            Self::Network(hosts) => write!(f, "network:{hosts}"),
        }
    }
}

/// Where an extension UI panel may appear (R-34).
#[derive(Serialize, Deserialize, Type, JsonSchema, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PanelSlot {
    /// A panel in the sidebar.
    Sidebar,
    /// A view in the main area.
    MainArea,
}

impl fmt::Display for PanelSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Sidebar => "sidebar",
            Self::MainArea => "main_area",
        })
    }
}

/// A host pattern for the network permission, e.g. `api.github.com` or `*.openrouter.ai`.
#[derive(Serialize, Deserialize, Type, JsonSchema, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct HostPattern(pub String);

impl fmt::Display for HostPattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
