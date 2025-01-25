-- Створюємо таблицю (якщо її немає)
CREATE TABLE IF NOT EXISTS "CryptoAssets" (
  "id" INTEGER PRIMARY KEY,           -- (NET_TYPE << 24) + (CHAIN_CODE << 16) + TOKEN_ID
  "net_type"   INTEGER NOT NULL,      -- 1 = EVM mainnet, 2 = EVM testnet, 3 = non-EVM main, 4 = non-EVM test, ...
  "chain_code" INTEGER NOT NULL,      -- конкретна мережа в межах net_type
  "token_id"   INTEGER NOT NULL,      -- ідентифікатор токена/монети в межах мережі
  "symbol"     TEXT NOT NULL,         -- умовне скорочення (ETH, BTC, USDT і т. д.)
  "description" TEXT
);

-- Додаємо записи (прикладна вибірка з EVM main/testnet і non-EVM)
INSERT INTO "CryptoAssets" (id, net_type, chain_code, token_id, symbol, description) VALUES

-- 1) EVM MAINNET (net_type = 1)
-- 1.1 Ethereum mainnet (chain_code = 1)
(16842752, 1, 1, 0, 'ETH',  'Ethereum'),
(16842753, 1, 1, 1, 'USDT', 'ERC20 Tether'),
(16842754, 1, 1, 2, 'USDC', 'ERC20 USD Coin'),
(16842755, 1, 1, 3, 'DAI',  'ERC20 Dai'),
(16842756, 1, 1, 4, 'WBTC', 'Wrapped BTC (ERC20)'),
(16842757, 1, 1, 5, 'LINK', 'Chainlink Token'),
(16842758, 1, 1, 6, 'SHIB', 'Shiba Inu'),
(16842759, 1, 1, 7, 'UNI',  'Uniswap Token'),

-- 1.2 BNB Chain (chain_code = 2)
(16908288, 1, 2, 0, 'BNB',  'BNB Chain'),
(16908289, 1, 2, 1, 'USDT', 'BEP20 Tether'),
(16908290, 1, 2, 2, 'USDC', 'BEP20 USD Coin'),
(16908291, 1, 2, 3, 'BUSD', 'Binance USD (BEP20)'),
(16908292, 1, 2, 4, 'CAKE','PancakeSwap Token'),

-- 1.3 Polygon mainnet (chain_code = 3)
(16973824, 1, 3, 0, 'MATIC','Polygon'),
(16973825, 1, 3, 1, 'USDT', 'USDT на Polygon'),
(16973826, 1, 3, 2, 'USDC', 'USDC на Polygon'),
(16973827, 1, 3, 3, 'DAI',  'DAI на Polygon'),

-- 1.4 Avalanche C-Chain (chain_code = 4)
(17039360, 1, 4, 0, 'AVAX', 'Avalanche C-Chain'),
(17039361, 1, 4, 1, 'USDT', 'USDT на Avalanche'),
(17039362, 1, 4, 2, 'USDC', 'USDC на Avalanche'),

-- 1.5 Arbitrum One (chain_code = 5)
(17104896, 1, 5, 0, 'ETH',  'Bridged ETH на Arbitrum'),
(17104897, 1, 5, 1, 'USDT', 'USDT на Arbitrum'),
(17104898, 1, 5, 2, 'USDC', 'USDC на Arbitrum'),

-- 1.6 Optimism (chain_code = 6)
(17170432, 1, 6, 0, 'ETH',  'Bridged ETH на Optimism'),
(17170433, 1, 6, 1, 'USDT', 'USDT на Optimism'),
(17170434, 1, 6, 2, 'USDC', 'USDC на Optimism'),

-- 1.7 Fantom (chain_code = 7)
(17235968, 1, 7, 0, 'FTM',  'Fantom mainnet token'),
(17235969, 1, 7, 1, 'USDT','USDT на Fantom'),
(17235970, 1, 7, 2, 'USDC','USDC на Fantom'),

-- (Приклад додаткових EVM mainnets)
(17301504, 1, 8, 0, 'CRO', 'Cronos mainnet'),
(17367040, 1, 9, 0, 'KCS', 'KuCoin Community Chain mainnet'),
(17432576, 1,10, 0, 'XDAI','Gnosis Chain (ex xDai)'),
(17498112, 1,11, 0, 'ETH', 'Base mainnet від Coinbase'),

-- 2) EVM TESTNET (net_type = 2)
-- 2.1 Ethereum testnets
(33619968, 2, 1, 0, 'ETH',  'Goerli ETH'),
(33619969, 2, 1, 1, 'USDT', 'Goerli USDT'),
(33619970, 2, 1, 2, 'USDC', 'Goerli USDC'),
(33685504, 2, 2, 0, 'ETH',  'Sepolia ETH'),

-- 2.2 BNB Testnet (chain_code = 3)
(33751040, 2, 3, 0, 'BNB',  'BNB testnet'),

-- 2.3 Polygon Mumbai (chain_code = 4)
(33816576, 2, 4, 0, 'MATIC', 'Polygon Mumbai'),

-- 2.4 Avalanche Fuji (chain_code = 5)
(33882112, 2, 5, 0, 'AVAX', 'Avalanche Fuji test'),

-- 2.5 Arbitrum / Optimism test (chain_code = 6,7)
(33947648, 2, 6, 0, 'ETH', 'Arbitrum testnet ETH'),
(34013184, 2, 7, 0, 'ETH', 'Optimism testnet ETH'),

-- 3) NON-EVM MAINNET (net_type = 3)
(50331648, 3, 1, 0, 'BTC', 'Bitcoin mainnet'),
(50397184, 3, 2, 0, 'TRX', 'Tron mainnet'),
(50462720, 3, 3, 0, 'XRP', 'Ripple mainnet'),
(50528256, 3, 4, 0, 'SOL', 'Solana mainnet'),
(50593792, 3, 5, 0, 'DOT', 'Polkadot mainnet'),
(50659328, 3, 6, 0, 'DOGE','Dogecoin'),
(50724864, 3, 7, 0, 'ADA', 'Cardano'),
(50790400, 3, 8, 0, 'XTZ', 'Tezos'),
(50855936, 3, 9, 0, 'XLM', 'Stellar'),
(50921472, 3,10, 0, 'NEO', 'NEO mainnet'),
(50987008, 3,11, 0, 'LTC', 'Litecoin'),
(51052544, 3,12, 0, 'BCH', 'Bitcoin Cash'),

-- 4) NON-EVM TESTNET (net_type = 4)
(67108864, 4, 1, 0, 'tBTC','Bitcoin testnet'),
(67174400, 4, 2, 0, 'TRX', 'Tron Shasta testnet'),
(67239936, 4, 3, 0, 'SOL', 'Solana devnet/testnet');

-- Після виконання: перевірте, що записи з'явилися
-- SELECT * FROM "CryptoAssets";
