const nearApi = require('near-api-js');
const Big = require("big.js");
const {CALLBACK_URL, NETWORK, CONTRACT, PROVIDER} = require("./config");
const precisionMap = {};

async function signURL (receiverId, method, args={}, attachedDeposit='0', gas=30000000000000, meta) {
    const accountId = 'account';
    const deposit  	= typeof attachedDeposit =='string'?attachedDeposit:nearApi.utils.format.parseNearAmount(''+attachedDeposit)
    const actions  	= [method === '!transfer'? nearApi.transactions.transfer(deposit) : nearApi.transactions.functionCall(method, Buffer.from(JSON.stringify(args)), gas, deposit)]
    const keypair 	= nearApi.utils.KeyPair.fromRandom('ed25519')
    const block 	= await PROVIDER.block({finality:'final'})
    const txs 		= [nearApi.transactions.createTransaction(accountId, keypair.publicKey, receiverId, 1, actions, nearApi.utils.serialize.base_decode(block.header.hash))]
    const newUrl 	= new URL('sign',`https://wallet.${NETWORK}.near.org/`);
    newUrl.searchParams.set('transactions', txs.map(transaction => nearApi.utils.serialize.serialize(nearApi.transactions.SCHEMA, transaction)).map(serialized => Buffer.from(serialized).toString('base64')).join(','))
    newUrl.searchParams.set('callbackUrl', CALLBACK_URL)
    if (meta) newUrl.searchParams.set('meta', meta)
    return newUrl.href
}


async function contractQuery(contract, method, args = "") {
    const rawResult = await PROVIDER.query({
        request_type: "call_function",
        account_id: contract,
        method_name: method,
        args_base64: Buffer.from(JSON.stringify(args)).toString('base64'),
        finality: "optimistic",
    });
    return JSON.parse(Buffer.from(rawResult.result).toString());
}

async function getTokenPrecision(tokenAddress) {
    if (!precisionMap[tokenAddress]) {
        const result = await contractQuery(tokenAddress, 'ft_metadata');
        precisionMap[tokenAddress] = result.decimals;
    }

    return precisionMap[tokenAddress];
}

async function getOrder(orderId) {
    return contractQuery(CONTRACT, 'get_order', {orderId});
}

async function toPrecision(value, tokenAddress, fixed = 6) {
    const precision = await getTokenPrecision(tokenAddress);
    return Big(value).div(Big(10).pow(precision)).round(fixed).toFixed();
}

async function fromPrecision(value, tokenAddress) {
    const precision = await getTokenPrecision(tokenAddress);
    return Big(value).mul(Big(10).pow(precision)).toFixed();
}

async function formatOrderList(orderList) {
    const inline_keyboard = [];
    for (const {sell_amount, sell_token, buy_amount, buy_token, order_id} of orderList) {
        inline_keyboard.push([{
            text: `Sell ${await toPrecision(sell_amount, sell_token)} ${sell_token} for ${await toPrecision(buy_amount, buy_token)} ${buy_token}`,
            switch_inline_query_current_chat: `match ${order_id}`,
        }]);
    }
    return {
        reply_markup: {
            inline_keyboard
        }
    }
}

module.exports = {
    signURL,
    contractQuery,
    getTokenPrecision,
    getOrder,
    toPrecision,
    fromPrecision,
    formatOrderList
}