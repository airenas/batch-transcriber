-- Your SQL goes here
ALTER TABLE
    work_data
ADD
    COLUMN error_msg TEXT NOT NULL DEFAULT '';

ALTER TABLE
    work_data
ADD
    COLUMN upload_time TIMESTAMP NULL;