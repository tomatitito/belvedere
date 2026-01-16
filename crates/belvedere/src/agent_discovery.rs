use std::path::{Path, PathBuf};

/// Agent roles in the Gas Town ecosystem
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgentRole {
    /// Primary AI coordinator
    Mayor,
    /// Ephemeral worker agents
    Polecat,
    /// Personal workspace agent
    Crew,
    /// Witness for specific rigs
    Witness,
    /// Deacon daemon
    Deacon,
    /// Unknown/unrecognized role
    Unknown,
}

impl AgentRole {
    /// Parse agent role from a directory name component
    fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "mayor" => AgentRole::Mayor,
            "polecat" => AgentRole::Polecat,
            "crew" => AgentRole::Crew,
            "witness" => AgentRole::Witness,
            "deacon" => AgentRole::Deacon,
            _ => AgentRole::Unknown,
        }
    }
}

impl std::fmt::Display for AgentRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentRole::Mayor => write!(f, "Mayor"),
            AgentRole::Polecat => write!(f, "Polecat"),
            AgentRole::Crew => write!(f, "Crew"),
            AgentRole::Witness => write!(f, "Witness"),
            AgentRole::Deacon => write!(f, "Deacon"),
            AgentRole::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Represents a discovered agent directory
#[derive(Debug, Clone, PartialEq)]
pub struct AgentDirectory {
    /// Full path to the agent directory
    pub path: PathBuf,
    /// The agent's role type
    pub role: AgentRole,
    /// The full instance name (e.g., "polecat-1", "mayor", "crew-alice")
    pub instance_name: String,
    /// Optional instance identifier (e.g., "1" from "polecat-1", "alice" from "crew-alice")
    pub instance_id: Option<String>,
}

impl AgentDirectory {
    /// Parse an agent directory from a path
    ///
    /// Directory names follow the pattern: `{role}[-{instance}]`
    /// Examples:
    /// - `mayor` → Mayor role, no instance
    /// - `polecat-1` → Polecat role, instance "1"
    /// - `crew-alice` → Crew role, instance "alice"
    /// - `witness-backend` → Witness role, instance "backend"
    pub fn from_path(path: PathBuf) -> Option<Self> {
        let dir_name = path.file_name()?.to_str()?.to_string();

        // Split on first hyphen to separate role from instance
        let parts: Vec<&str> = dir_name.splitn(2, '-').collect();

        let role = AgentRole::from_name(parts[0]);
        let instance_id = parts.get(1).map(|s| s.to_string());

        Some(AgentDirectory {
            path,
            role,
            instance_name: dir_name,
            instance_id,
        })
    }
}

/// Discovers agent directories from known locations
pub struct AgentDiscovery {
    /// Root directory for Gas Town (e.g., ~/gt/)
    gastown_root: Option<PathBuf>,
}

impl AgentDiscovery {
    /// Create a new agent discovery instance
    pub fn new(gastown_root: Option<PathBuf>) -> Self {
        Self { gastown_root }
    }

    /// Discover all agent directories
    ///
    /// Scans:
    /// - Standalone agents: `~/.gazetown/agents/`
    /// - In-rig agents: `<rig>/.agents/`
    pub fn discover_agents(&self) -> Vec<AgentDirectory> {
        let mut agents = Vec::new();

        // Discover standalone agents
        if let Some(standalone) = self.discover_standalone_agents() {
            agents.extend(standalone);
        }

        // Discover in-rig agents
        if let Some(rig_agents) = self.discover_rig_agents() {
            agents.extend(rig_agents);
        }

        agents
    }

    /// Discover standalone agents from ~/.gazetown/agents/
    fn discover_standalone_agents(&self) -> Option<Vec<AgentDirectory>> {
        let home = dirs::home_dir()?;
        let agents_dir = home.join(".gazetown").join("agents");

        if !agents_dir.exists() {
            return None;
        }

        Some(self.scan_agents_directory(&agents_dir))
    }

    /// Discover in-rig agents
    fn discover_rig_agents(&self) -> Option<Vec<AgentDirectory>> {
        let root = self.gastown_root.as_ref()?;

        if !root.exists() {
            return None;
        }

        let mut agents = Vec::new();

        // Scan for rig directories (any subdirectory with .agents folder)
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_dir() {
                        let agents_dir = entry.path().join(".agents");
                        if agents_dir.exists() {
                            agents.extend(self.scan_agents_directory(&agents_dir));
                        }
                    }
                }
            }
        }

        Some(agents)
    }

    /// Scan a specific agents directory for agent subdirectories
    fn scan_agents_directory(&self, agents_dir: &Path) -> Vec<AgentDirectory> {
        let mut agents = Vec::new();

        if let Ok(entries) = std::fs::read_dir(agents_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_dir() {
                        if let Some(agent) = AgentDirectory::from_path(entry.path()) {
                            // Filter out unknown roles unless we want to keep them for debugging
                            if agent.role != AgentRole::Unknown {
                                agents.push(agent);
                            }
                        }
                    }
                }
            }
        }

        agents
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_role_from_name() {
        assert_eq!(AgentRole::from_name("mayor"), AgentRole::Mayor);
        assert_eq!(AgentRole::from_name("MAYOR"), AgentRole::Mayor);
        assert_eq!(AgentRole::from_name("polecat"), AgentRole::Polecat);
        assert_eq!(AgentRole::from_name("crew"), AgentRole::Crew);
        assert_eq!(AgentRole::from_name("witness"), AgentRole::Witness);
        assert_eq!(AgentRole::from_name("deacon"), AgentRole::Deacon);
        assert_eq!(AgentRole::from_name("unknown"), AgentRole::Unknown);
    }

    #[test]
    fn test_agent_directory_from_path_mayor() {
        let path = PathBuf::from("/home/user/.gazetown/agents/mayor");
        let agent = AgentDirectory::from_path(path.clone()).unwrap();

        assert_eq!(agent.path, path);
        assert_eq!(agent.role, AgentRole::Mayor);
        assert_eq!(agent.instance_name, "mayor");
        assert_eq!(agent.instance_id, None);
    }

    #[test]
    fn test_agent_directory_from_path_polecat_with_instance() {
        let path = PathBuf::from("/rig/.agents/polecat-1");
        let agent = AgentDirectory::from_path(path.clone()).unwrap();

        assert_eq!(agent.path, path);
        assert_eq!(agent.role, AgentRole::Polecat);
        assert_eq!(agent.instance_name, "polecat-1");
        assert_eq!(agent.instance_id, Some("1".to_string()));
    }

    #[test]
    fn test_agent_directory_from_path_crew_with_name() {
        let path = PathBuf::from("/rig/.agents/crew-alice");
        let agent = AgentDirectory::from_path(path.clone()).unwrap();

        assert_eq!(agent.path, path);
        assert_eq!(agent.role, AgentRole::Crew);
        assert_eq!(agent.instance_name, "crew-alice");
        assert_eq!(agent.instance_id, Some("alice".to_string()));
    }

    #[test]
    fn test_agent_directory_from_path_witness_with_context() {
        let path = PathBuf::from("/rig/.agents/witness-backend");
        let agent = AgentDirectory::from_path(path.clone()).unwrap();

        assert_eq!(agent.path, path);
        assert_eq!(agent.role, AgentRole::Witness);
        assert_eq!(agent.instance_name, "witness-backend");
        assert_eq!(agent.instance_id, Some("backend".to_string()));
    }

    #[test]
    fn test_agent_directory_from_path_unknown_role() {
        let path = PathBuf::from("/rig/.agents/unknown-role");
        let agent = AgentDirectory::from_path(path.clone()).unwrap();

        assert_eq!(agent.role, AgentRole::Unknown);
        assert_eq!(agent.instance_name, "unknown-role");
        assert_eq!(agent.instance_id, Some("role".to_string()));
    }

    #[test]
    fn test_agent_directory_display() {
        assert_eq!(AgentRole::Mayor.to_string(), "Mayor");
        assert_eq!(AgentRole::Polecat.to_string(), "Polecat");
        assert_eq!(AgentRole::Crew.to_string(), "Crew");
    }
}
