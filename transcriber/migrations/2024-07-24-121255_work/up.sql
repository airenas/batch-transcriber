--SQL statements for a work table
CREATE TABLE work_data(
    id TEXT NOT NULL PRIMARY KEY,
    external_ID TEXT NOT NULL,
    transcriber_url TEXT,
    try_count INT DEFAULT 0,
    created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
