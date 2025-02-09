
        CREATE TABLE IF NOT EXISTS transactions (
            transaction_hash bytea PRIMARY KEY,           -- 32 байти, наприклад, хеш транзакції
            transaction_type SMALLINT NOT NULL,             -- тип транзакції (1 або 2)
            currency_id INTEGER NOT NULL,                   -- ідентифікатор валюти
            amount bytea NOT NULL,                          -- 32-байтове значення, збережене без конвертації в текст
            timestamp BIGINT NOT NULL,                      -- Unix-час у мілісекундах
            sender_address bytea,                           -- адреса відправника (20 байт)
            sender_output_index INTEGER,                    -- порядковий номер вихідної транзакції відправника
            recipient_address bytea NOT NULL,               -- адреса отримувача (20 байт)
            sender_signature bytea,                         -- підпис відправника (65 байт)
            source_transaction_hash bytea                   -- хеш поповнення, якщо є (32 байти)
        );

-- Приклади індексів для поліпшення продуктивності пошуку
        CREATE INDEX IF NOT EXISTS idx_sender_address ON transactions(sender_address);
        CREATE INDEX IF NOT EXISTS idx_recipient_address ON transactions(recipient_address);
        