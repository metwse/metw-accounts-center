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

    -- REF: I-AR-1
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

    -- REF: R-AR-3
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

    -- REF: I-AR-7
    CONSTRAINT usernames_username_pkey
        PRIMARY KEY (username),

    -- Primary usernames cannot expire.
    -- REF: I-AR-4
    CONSTRAINT usernames_primary_username_cannot_expire_check
        CHECK (NOT (is_primary AND expires_at IS NOT NULL)),

    -- Prohibit uppercase letters in usernames.
    -- REF: I-AR-5
    CONSTRAINT usernames_username_lowercase_check
        CHECK (username = LOWER(username)),

    -- REF: R-AR-2
    CONSTRAINT usernames_account_id_fk
        FOREIGN KEY (account_id) REFERENCES accounts (id)
        ON DELETE CASCADE
);

-- An account can have at most one primary username.
-- REF: I-AR-2
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

    -- REF: I-AR-8
    CONSTRAINT emails_email_pkey
        PRIMARY KEY (email),

    -- Prohibit uppercase letters in emails similar to usernames.
    -- REF: I-AR-6
    CONSTRAINT emails_email_lowercase_check
        CHECK (email = LOWER(email)),

    -- REF: R-AR-1
    CONSTRAINT emails_account_id_fk
        FOREIGN KEY (account_id) REFERENCES accounts (id)
        ON DELETE CASCADE
);

-- An account can have at most one primary email.
-- REF: I-AR-3
CREATE UNIQUE INDEX emails_at_most_one_primary_email_per_account_check
    ON emails (account_id, is_primary)
    WHERE is_primary;

-- account_id -> email index
CREATE INDEX email_account_id_idx
    ON emails (account_id);
