INSERT INTO assets (name, unit_value) VALUES
    ('Bitcoin (BTC)', 65000.00),
    ('Ethereum (ETH)', 3500.00),
    ('Tesouro Selic 2029', 14000.00),
    ('Apple (AAPL)', 175.50),
    ('Nvidia (NVDA)', 850.00),
    ('Microsoft (MSFT)', 420.00)
ON CONFLICT DO NOTHING;
