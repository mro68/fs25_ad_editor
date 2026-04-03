//! Session-Backup fuer aktive Tool-Edits.

use crate::app::group_registry::GroupRecord;

use super::ToolEditRecord;

/// Laufende Tool-Edit-Session mit Registry- und Payload-Backup.
#[derive(Debug, Clone)]
pub struct ActiveToolEditSession {
    /// Urspruengliche Record-ID der bearbeiteten Gruppe.
    pub record_id: u64,
    /// Backup des neutralen Session-Records.
    pub group_record_backup: GroupRecord,
    /// Backup des tool-spezifischen Edit-Snapshots.
    pub tool_edit_backup: ToolEditRecord,
}
