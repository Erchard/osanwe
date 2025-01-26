CREATE TABLE transactions (
    transaction_hash TEXT PRIMARY KEY,
    transaction_type INTEGER NOT NULL,
    currency_id INTEGER NOT NULL,
    amount TEXT NOT NULL,
    decimal TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    sender_address TEXT NOT NULL,
    sender_output_index INTEGER NOT NULL,
    recipient_address TEXT NOT NULL,
    sender_signature TEXT NOT NULL,
    source_transaction_hash TEXT NOT NULL
);
