-- This file should undo anything in `up.sql`
ALTER TABLE
    work_data DROP COLUMN upload_time;

ALTER TABLE
    work_data DROP COLUMN error_msg;
