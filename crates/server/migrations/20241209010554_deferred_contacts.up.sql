CREATE TABLE deferred_contacts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    submitter_number TEXT NOT NULL,
    contact_name TEXT NOT NULL,
    phone_number TEXT NOT NULL,
    phone_description TEXT,
    FOREIGN KEY(submitter_number) REFERENCES users(number) ON DELETE CASCADE
);