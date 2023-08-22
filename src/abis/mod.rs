use ethers::prelude::abigen;

abigen!(IERC20, "src/abis/IERC20.json");
abigen!(SWAP_ROUTER, "src/abis/SwapRouter.json");
abigen!(UNISWAP_V2_FACTORY, "src/abis/UniswapV2Factory.json");
abigen!(UNISWAP_V2_ROUTER02, "src/abis/UniswapV2Router02.json");
abigen!(BUY_BOT, "src/abis/BuyBot.json");
abigen!(BLACKLIST, "src/abis/Blacklist.json");
