CREATE TABLE groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    creator_number TEXT NOT NULL,
    FOREIGN KEY(creator_number) REFERENCES users(number) ON DELETE CASCADE,
    UNIQUE(name, creator_number)
);
CREATE TABLE group_members (
    group_id INTEGER NOT NULL,
    member_number TEXT NOT NULL,
    FOREIGN KEY(group_id) REFERENCES groups(id) ON DELETE CASCADE,
    FOREIGN KEY(member_number) REFERENCES users(number),
    PRIMARY KEY(group_id, member_number)
);