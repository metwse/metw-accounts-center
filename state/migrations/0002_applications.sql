-- 1 "applications" table
-- ---------------------------------------------------------------------------
-- This applications are for authorizing 3rd party services.
CREATE TABLE applications (
    application_id bigint NOT NULL,
    owner_account_id bigint NOT NULL,

    name varchar(32) NOT NULL,
    client_secret_hash bytea NOT NULL,

    CONSTRAINT applications_application_id_pkey
        PRIMARY KEY (application_id),

    CONSTRAINT applications_owner_account_id_fk
        FOREIGN KEY (owner_account_id) REFERENCES accounts (account_id)
        ON DELETE CASCADE,

    CONSTRAINT applications_client_secret_hash_length_check
        CHECK (octet_length(client_secret_hash) = 32)
);

-- 2 "application_redirect_urls" table
-- ---------------------------------------------------------------------------
-- Allowed redirection URLs for the application.
CREATE TABLE application_redirect_urls (
    application_id bigint NOT NULL,
    redirect_url text NOT NULL,

    CONSTRAINT application_redirect_urls_application_id_redirect_url_pkey
        PRIMARY KEY (application_id, redirect_url),

    CONSTRAINT application_redirect_urls_application_id_fk
        FOREIGN KEY (application_id) REFERENCES applications (application_id)
        ON DELETE CASCADE
);

-- 3 "account_application_consents" table
-- ---------------------------------------------------------------------------
-- Authorized applications by accounts.
CREATE TABLE account_application_consents (
    account_id bigint NOT NULL,
    application_id bigint NOT NULL,

    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,

    key_encryption_algorithm jsonb NOT NULL,
    master_key_encrypted_key bytea,
    master_key_id bigint,

    CONSTRAINT account_application_consents_account_id_application_id_pkey
        PRIMARY KEY (account_id, application_id),

    CONSTRAINT account_application_consents_account_id_fk
        FOREIGN KEY (account_id) REFERENCES accounts (account_id)
        ON DELETE CASCADE,

    CONSTRAINT account_application_consents_application_id_fk
        FOREIGN KEY (application_id) REFERENCES applications (application_id)
        ON DELETE CASCADE,

    CONSTRAINT account_application_consents_master_key_id_master_key_encrypted_key_both_null_check
        CHECK ((master_key_id IS NULL) = (master_key_encrypted_key IS NULL))
);
