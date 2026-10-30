//! Explicit parent evidence; gateway children and logical splits are different relations.
use super::*;
use std::collections::HashMap;

/// One ordinary directory entry with its explicitly linked logical splits.
/// It is a candidate for binding, not evidence of physical existence or control support.
/// ```compile_fail
/// fn export(value: &homeverb_mijia::ModernDeviceGroup) {
///     let _ = serde_json::to_string(value);
/// }
/// ```
pub struct ModernDeviceGroup {
    entry: ModernDirectoryEntry,
    logical_children: Vec<ModernDirectoryEntry>,
}
impl ModernDeviceGroup {
    pub fn entry(&self) -> &ModernDirectoryEntry {
        &self.entry
    }
    pub fn logical_children(&self) -> &[ModernDirectoryEntry] {
        &self.logical_children
    }
}
impl fmt::Debug for ModernDeviceGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ModernDeviceGroup([REDACTED])")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModernUnresolvedReason {
    MissingKind,
    UnknownKind,
    MissingParent,
    UnsupportedParent,
}
/// Retains an unresolved entry without leaking its provider identity in diagnostics.
pub struct ModernUnresolvedEntry {
    entry: ModernDirectoryEntry,
    reason: ModernUnresolvedReason,
}
impl ModernUnresolvedEntry {
    pub fn entry(&self) -> &ModernDirectoryEntry {
        &self.entry
    }
    pub fn reason(&self) -> ModernUnresolvedReason {
        self.reason
    }
}
impl fmt::Debug for ModernUnresolvedEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ModernUnresolvedEntry([REDACTED])")
    }
}
/// Every original entry is retained in exactly one group or unresolved record.
/// ```compile_fail
/// fn export(value: &homeverb_mijia::ModernDeviceInventory) {
///     let _ = serde_json::to_string(value);
/// }
/// ```
pub struct ModernDeviceInventory {
    pub groups: Vec<ModernDeviceGroup>,
    pub unresolved: Vec<ModernUnresolvedEntry>,
    pub issues: Vec<Error>,
    pub complete: bool,
}
impl fmt::Debug for ModernDeviceInventory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ModernDeviceInventory([REDACTED])")
    }
}
fn ordinary(kind: Option<DirectoryKind>) -> bool {
    matches!(kind, Some(DirectoryKind(0 | 6 | 16 | 18)))
}
async fn checkpoint(index: usize, operation: &Operation) -> Result<()> {
    operation
        .run(async {
            if index.is_multiple_of(64) {
                tokio::task::yield_now().await;
            }
            Ok(())
        })
        .await
}
impl ModernDirectory {
    /// Local bounded reconciliation only. Never reads a device or publishes a card.
    pub async fn reconcile(self, operation: &Operation) -> Result<ModernDeviceInventory> {
        operation.run(self.reconcile_inner(operation)).await
    }
    async fn reconcile_inner(self, operation: &Operation) -> Result<ModernDeviceInventory> {
        if self.complete || self.entries.len() > MAX_ENTRIES || self.issues.len() > 64 {
            return Err(invalid());
        }
        let mut index = HashMap::new();
        for (i, entry) in self.entries.iter().enumerate() {
            checkpoint(i, operation).await?;
            if index.insert(entry.did.as_str(), i).is_some()
                || self.entries.first().is_some_and(|first| {
                    first.subject != entry.subject
                        || first.encrypted_subject != entry.encrypted_subject
                })
            {
                return Err(crate::protected::conflict());
            }
        }
        // Validate every explicit in-directory parent edge, including gateway links.
        // Three-color iterative traversal is linear and does not recurse on wire data.
        let mut colors = vec![0_u8; self.entries.len()];
        for start in 0..self.entries.len() {
            checkpoint(start, operation).await?;
            let mut current = Some(start);
            let mut path = vec![];
            while let Some(i) = current {
                if colors[i] == 2 {
                    break;
                }
                if colors[i] == 1 {
                    return Err(crate::protected::conflict());
                }
                colors[i] = 1;
                path.push(i);
                checkpoint(path.len(), operation).await?;
                current = self.entries[i]
                    .parent
                    .as_ref()
                    .and_then(|parent| index.get(parent.as_str()).copied());
            }
            for i in path {
                colors[i] = 2;
            }
        }
        let mut owners = Vec::with_capacity(self.entries.len());
        for (i, entry) in self.entries.iter().enumerate() {
            checkpoint(i, operation).await?;
            use ModernUnresolvedReason::*;
            let owner = match entry.kind {
                None => Err(MissingKind),
                kind if ordinary(kind) => Ok(i),
                Some(DirectoryKind(21)) => {
                    match entry
                        .parent
                        .as_ref()
                        .and_then(|parent| index.get(parent.as_str()))
                    {
                        None => Err(MissingParent),
                        Some(parent_index) => {
                            let parent = &self.entries[*parent_index];
                            if !ordinary(parent.kind) {
                                Err(UnsupportedParent)
                            } else {
                                if entry.model != parent.model
                                    || matches!((&entry.spec_type, &parent.spec_type), (Some(a), Some(b)) if a != b)
                                {
                                    return Err(crate::protected::conflict());
                                }
                                Ok(*parent_index)
                            }
                        }
                    }
                }
                _ => Err(UnknownKind),
            };
            owners.push(owner);
        }
        // Allocate each group at its original index, so child-before-parent pages
        // are equivalent to parent-before-child pages and every entry is moved once.
        let mut roots: Vec<Option<ModernDirectoryEntry>> =
            (0..owners.len()).map(|_| None).collect();
        let mut children: Vec<Vec<ModernDirectoryEntry>> =
            (0..owners.len()).map(|_| vec![]).collect();
        let mut unresolved = vec![];
        for (i, (entry, owner)) in self.entries.into_iter().zip(owners).enumerate() {
            checkpoint(i, operation).await?;
            match owner {
                Ok(owner) if owner == i => roots[i] = Some(entry),
                Ok(owner) => children[owner].push(entry),
                Err(reason) => unresolved.push(ModernUnresolvedEntry { entry, reason }),
            }
        }
        let mut issues = self.issues;
        if !unresolved.is_empty()
            && issues.len() < 64
            && !issues.iter().any(|e| e.code == "MIJIA_INVENTORY_PARTIAL")
        {
            issues.push(partial());
        }
        let groups = roots
            .into_iter()
            .zip(children)
            .filter_map(|(entry, logical_children)| {
                entry.map(|entry| ModernDeviceGroup {
                    entry,
                    logical_children,
                })
            })
            .collect();
        operation
            .run(async {
                Ok(ModernDeviceInventory {
                    groups,
                    unresolved,
                    issues,
                    complete: false,
                })
            })
            .await
    }
}
