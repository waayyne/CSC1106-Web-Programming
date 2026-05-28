-- This SQL migration script inserts a permanent admin user into the users table if it doesn't already exist, 
-- Ensuring that there is always an admin account available for managing the application.

INSERT INTO users (username, first_name, last_name, name, email, password_hash, phone_number, role)
VALUES (
    'BankAdmin',
    'PeterThe',
    'AdminParker',
    'Spidey',
    'admin@bank.com',
    '$argon2id$v=19$m=19456,t=2,p=1$ayVX+Sf9MqhXftI3uNH/yQ$XSrEUDamS2nLmtJQXJcZnlwVXJiYBLTAF2FFLcPVSfU',
    '1234567890',
    'admin'
)
ON CONFLICT (username) DO NOTHING; -- This ensures that if the 'BankAdmin' user already exists, the insert will be ignored, preventing duplicate entries.