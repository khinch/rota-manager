-- Add up migration script here
ALTER TABLE users ADD COLUMN id UUID NOT NULL DEFAULT gen_random_uuid();

-- Make id the new primary key and remove primary key from email
ALTER TABLE users DROP CONSTRAINT users_pkey;
ALTER TABLE users ADD CONSTRAINT users_pkey PRIMARY KEY (id);
ALTER TABLE users ADD CONSTRAINT users_email_unique UNIQUE (email);
