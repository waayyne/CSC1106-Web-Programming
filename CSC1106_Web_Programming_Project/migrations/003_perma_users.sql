-- This SQL migration script inserts a permanent admin, staff, and standard user into the users table 
-- Ensuring that there are always testing accounts available for all three roles.

INSERT INTO users (username, first_name, last_name, name, email, email_verified, email_verified_at, password_hash, phone_number, role)
VALUES 
    -- 1. Permanent Admin Account
    (
        'BankAdmin',
        'PeterThe',
        'AdminParker',
        'Spidey',
        'admin@bank.com',
        true,
        current_timestamp,
        '$argon2id$v=19$m=19456,t=2,p=1$ayVX+Sf9MqhXftI3uNH/yQ$XSrEUDamS2nLmtJQXJcZnlwVXJiYBLTAF2FFLcPVSfU', -- Password: SpideyBank
        '12345678',
        'admin'
    ),
    -- 2. Permanent Staff Account
    (
        'BankStaff',
        'MilesThe',
        'StaffMorales',
        'Miles',
        'staff@bank.com',
        true,
        current_timestamp,
        '$argon2id$v=19$m=19456,t=2,p=1$3jW7/sHwKaux4jtqDo6hfA$OD2HDrFos0/JsWyzmV/BWIhj/d5jf4sysAeRxCXoZ/8', -- Password: Staff123
        '09876543',
        'staff'
    ),
    -- 3. Permanent Normal User Account
    (
        'BankUser',
        'GwenThe',
        'UserStacy',
        'Gwen',
        'user@bank.com',
        true,
        current_timestamp,
        '$argon2id$v=19$m=19456,t=2,p=1$ZtAXXXYVbJv9XevHzwMLRA$NFAerueDFtXfJAgoG8ZSZlGG3+Q1+IdOihC3XvsONS8', -- Password: Guest123
        '55555555',
        'customer' 
    )
ON CONFLICT (username) DO NOTHING; -- Skips any accounts that already exist without throwing an error

-- 4. Create a Bank Account with Play Cash for the Normal User
INSERT INTO bank_accounts (user_id, account_number, balance)
VALUES (
    (SELECT id FROM users WHERE username = 'BankUser'), 
    'RB-TEST-0001',                                     
    100000.00                                           
)
ON CONFLICT (account_number) DO UPDATE 
SET balance = EXCLUDED.balance;