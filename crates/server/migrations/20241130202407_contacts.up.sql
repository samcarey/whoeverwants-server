CREATE TABLE contacts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    submitter_number TEXT NOT NULL,
    contact_name TEXT NOT NULL,
    contact_number TEXT NOT NULL,
    FOREIGN KEY(submitter_number) REFERENCES users(number) ON DELETE CASCADE,
    UNIQUE(submitter_number, contact_number)
);