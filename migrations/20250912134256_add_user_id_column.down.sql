-- Add down migration script here
ALTER TABLE users DROP CONSTRAINT users_email_unique;
ALTER TABLE users DROP CONSTRAINT users_pkey;
ALTER TABLE users ADD CONSTRAINT users_pkey PRIMARY KEY (email);
ALTER TABLE users DROP COLUMN id;
