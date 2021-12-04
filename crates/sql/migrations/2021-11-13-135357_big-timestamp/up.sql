/*
Note: we can't add a new "not null" column if there
are existing rows, so in this migration we simply
truncate the existing database - this is OK
because we don't really care about the operations history.
*/
DELETE FROM operations;
ALTER TABLE operations ADD COLUMN timestamp BIGINT NOT NULL;
ALTER TABLE operations DROP COLUMN date;
