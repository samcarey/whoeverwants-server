CREATE TABLE contacts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    submitter_number TEXT NOT NULL,
    contact_name TEXT NOT NULL,
    contact_user_number TEXT NOT NULL,
    FOREIGN KEY(submitter_number) REFERENCES users(number) ON DELETE CASCADE,
    FOREIGN KEY(contact_user_number) REFERENCES users(number),
    UNIQUE(submitter_number, contact_user_number)
);