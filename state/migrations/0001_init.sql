-- 1 "accounts" table
-- ---------------------------------------------------------------------------
-- This table is used for storing cryptographic primitives.
CREATE TABLE accounts
(
    id bigint NOT NULL,
    password_hash text NOT NULL,
    identity_key bytea NOT NULL,
    encrypted_private_key bytea NOT NULL,
    encrypted_master_key bytea NOT NULL,

    CONSTRAINT accounts_id_pkey
        PRIMARY KEY (id)
);

-- 2 "account_flags" table
-- ---------------------------------------------------------------------------
-- Boolean flags for accounts.
CREATE TABLE account_flags
(
    id bigint NOT NULL,
    is_verified bool NOT NULL,

    CONSTRAINT account_flags_id_pkey
        PRIMARY KEY (id),

    CONSTRAINT account_flags_id_fk
        FOREIGN KEY (id) REFERENCES accounts (id)
        ON DELETE CASCADE
);


-- 3 "usernames" table
-- ---------------------------------------------------------------------------
-- One or more usernames may be associated to the account, or none. (usually
-- account deletion is pending if no username exists)
CREATE TABLE usernames (
    username varchar(20) NOT NULL,
    account_id bigint NOT NULL,
    is_primary bool NOT NULL,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at timestamptz,

    CONSTRAINT usernames_username_pkey
        PRIMARY KEY (username),

    -- Primary usernames cannot expire.
    CONSTRAINT usernames_primary_username_cannot_expire_check
        CHECK (NOT (is_primary AND expires_at IS NOT NULL)),

    -- Prohibit uppercase letters in usernames.
    CONSTRAINT usernames_username_lowercase_check
        CHECK (username = LOWER(username)),

    CONSTRAINT usernames_account_id_fk
        FOREIGN KEY (account_id) REFERENCES accounts (id)
        ON DELETE CASCADE
);

-- An accout can have at most one primary username.
CREATE UNIQUE INDEX usernames_at_most_one_primary_username_per_account_check
    ON usernames (account_id, is_primary)
    WHERE is_primary;

-- account_id -> username index
CREATE INDEX usernames_account_id_idx
    ON usernames (account_id);


-- 4 "emails" table
-- ---------------------------------------------------------------------------
-- Verified emails addresses of accounts.
CREATE TABLE emails (
    email varchar(254) NOT NULL,
    account_id bigint NOT NULL,
    is_primary bool NOT NULL,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT emails_email_pkey
        PRIMARY KEY (email),

    -- Prohibit uppercase letters in emails similar to usernames.
    CONSTRAINT emails_email_lowercase_check
        CHECK (email = LOWER(email)),

    CONSTRAINT emails_account_id_fk
        FOREIGN KEY (account_id) REFERENCES accounts (id)
        ON DELETE CASCADE
);

-- An accout can have at most one primary email.
CREATE UNIQUE INDEX emails_at_most_one_primary_email_per_account_check
    ON emails (account_id, is_primary)
    WHERE is_primary;

-- account_id -> email index
CREATE INDEX email_account_id_idx
    ON emails (account_id);
