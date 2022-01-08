import 'dotenv/config';

import * as nearApi from 'near-api-js';

const exit = (msg: string) => {
    console.log(msg);
    process.exit(0);
};

const NETWORK = process.env.NETWORK || exit('NETWORK NOT DEFINED');
const BOT_TOKEN = process.env.BOT_TOKEN || exit('TOKEN NOT DEFINED');
const BOT_NAME = process.env.BOT_NAME || exit('BOT_NAME NOT DEFINED');
const CONTRACT = process.env.CONTRACT || exit('CONTRACT NOT DEFINED');
const SERVER_URL = process.env.SERVER_URL || exit('SERVER_URL NOT DEFINED');
const EXPLORER_URL = process.env.EXPLORER_URL || exit('EXPLORER_URL NOT DEFINED');
const PORT = process.env.PORT || exit('PROT NOT DEFINED');
const USDT_TOKEN_ADDRESS = process.env.USDT_TOKEN_ADDRESS || exit('USDT_TOKEN_ADDRESS NOT DEFINED');
const RPC = `https://rpc.${NETWORK}.near.org`;
const CALLBACK_URL = `https://t.me/${BOT_NAME}`;
const PROVIDER = new nearApi.providers.JsonRpcProvider({ url: RPC });


export {
    NETWORK,
    BOT_TOKEN,
    BOT_NAME,
    CONTRACT,
    RPC,
    CALLBACK_URL,
    PROVIDER,
    SERVER_URL,
    EXPLORER_URL,
    PORT,
    USDT_TOKEN_ADDRESS,
}