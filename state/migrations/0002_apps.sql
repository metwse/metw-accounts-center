-- 1 "apps" table
-- ---------------------------------------------------------------------------
-- This applications are for authorizing 3rd party services.
CREATE TABLE apps (
    app_id bigint NOT NULL,
    owner_account_id bigint NOT NULL,

    name varchar(32) NOT NULL,
    client_secret_hash bytea NOT NULL,

    CONSTRAINT apps_app_id_pkey
        PRIMARY KEY (app_id),

    CONSTRAINT apps_owner_account_id_fk
        FOREIGN KEY (owner_account_id) REFERENCES accounts (account_id)
        ON DELETE CASCADE,

    CONSTRAINT apps_client_secret_hash_length_check
        CHECK (octet_length(client_secret_hash) = 32)
);

-- 2 "app_redirect_urls" table
-- ---------------------------------------------------------------------------
-- Allowed redirection URLs for the application.
CREATE TABLE app_redirect_urls (
    app_id bigint NOT NULL,
    redirect_url text NOT NULL,

    CONSTRAINT app_redirect_urls_app_id_redirect_url_pkey
        PRIMARY KEY (app_id, redirect_url),

    CONSTRAINT app_redirect_urls_app_id_fk
        FOREIGN KEY (app_id) REFERENCES apps (app_id)
        ON DELETE CASCADE
);

-- 3 "account_app_authorizations" table
-- ---------------------------------------------------------------------------
-- Authorized applications by accounts.
CREATE TABLE account_app_authorizations (
    account_id bigint NOT NULL,
    app_id bigint NOT NULL,

    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,

    key_encryption_algorithm jsonb NOT NULL,
    master_key_encrypted_key bytea,

    CONSTRAINT account_app_authorizations_account_id_app_id_pkey
        PRIMARY KEY (account_id, app_id),

    CONSTRAINT account_app_authorizations_account_id_fk
        FOREIGN KEY (account_id) REFERENCES accounts (account_id)
        ON DELETE CASCADE,

    CONSTRAINT account_app_authorizations_app_id_fk
        FOREIGN KEY (app_id) REFERENCES apps (app_id)
        ON DELETE CASCADE
);
