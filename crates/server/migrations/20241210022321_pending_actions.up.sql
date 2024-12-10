CREATE TABLE pending_actions (
    submitter_number TEXT PRIMARY KEY NOT NULL,
    action_type TEXT NOT NULL CHECK (
        action_type IN ('deletion', 'deferred_contacts', 'group')
    ),
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    FOREIGN KEY(submitter_number) REFERENCES users(number) ON DELETE CASCADE
);
CREATE TABLE pending_deletions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pending_action_submitter TEXT NOT NULL,
    contact_id INTEGER,
    group_id INTEGER,
    FOREIGN KEY(pending_action_submitter) REFERENCES pending_actions(submitter_number) ON DELETE CASCADE,
    FOREIGN KEY(contact_id) REFERENCES contacts(id) ON DELETE CASCADE,
    FOREIGN KEY(group_id) REFERENCES groups(id) ON DELETE CASCADE,
    CHECK (
        (
            contact_id IS NULL
            AND group_id IS NOT NULL
        )
        OR (
            contact_id IS NOT NULL
            AND group_id IS NULL
        )
    )
);
CREATE TABLE pending_group_members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pending_action_submitter TEXT NOT NULL,
    contact_id INTEGER NOT NULL,
    FOREIGN KEY(pending_action_submitter) REFERENCES pending_actions(submitter_number) ON DELETE CASCADE,
    FOREIGN KEY(contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);
CREATE INDEX idx_pending_deletions_submitter ON pending_deletions(pending_action_submitter);
CREATE INDEX idx_pending_group_members_submitter ON pending_group_members(pending_action_submitter);