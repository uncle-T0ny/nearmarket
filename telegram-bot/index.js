const TelegramBot = require('node-telegram-bot-api');
const {CONTRACT, BOT_TOKEN, EXPLORER_URL, CALLBACK_URL, PORT} = require("./config");
const {signURL, fromPrecision, formatOrderList, contractQuery, getOrder, loginUrl, formatPairList} = require("./utils");
const querystring = require('querystring');
const http = require("http");
const {getPair} = require("./utils");
const {PublicKey} = require("near-api-js/lib/utils");

const bot = new TelegramBot(BOT_TOKEN, {polling: true});

async function getUser(chatId) {
    const user = userMap[chatId];
    if (!user) {
        await bot.sendMessage(chatId, `Please [login](${loginUrl(chatId)}) first`, {parse_mode: 'MarkdownV2'});
        throw new Error('User not found');
    }
    return user;
}

async function sendTransaction(chatId, contract, method, args= {}, depositAddresses = [], deposit = '1') {
    const user = await getUser(chatId);
    const url = await signURL(user, contract, method, args, depositAddresses, deposit)
    await bot.sendMessage(chatId, `[Click to send transaction](${url})`, {parse_mode: 'MarkdownV2'});
}


// login
bot.onText(/\/login$/, async (msg, match) => {
    const chatId = msg.chat.id;
    bot.sendMessage(chatId, `Please follow the [Login URL](${loginUrl(chatId)})`, {parse_mode: 'MarkdownV2'});
});

// Get pairs
bot.onText(/\/orders$/, async (msg, match) => {
    const chatId = msg.chat.id;

    const result = await contractQuery(CONTRACT, "get_pairs",{});
    if (!result || !result.length) {
        bot.sendMessage(chatId, 'No proposals');
    } else {
        bot.sendMessage(chatId, 'Proposals:', await formatPairList(result));
    }
});

bot.on("callback_query", async function callback(callBackQuery) {
    const chatId = callBackQuery.message.chat.id;
    const [action, message] = callBackQuery.data.split(' ');
    if (action === 'orders') {
        const pair = getPair(message);
        const [sellToken, buyToken] = pair.split('#');
        const result = await contractQuery(CONTRACT, "get_orders", {sell_token: sellToken, buy_token: buyToken});
        if (!result || !result.length) {
            bot.sendMessage(chatId, 'No orders');
        } else {
            bot.sendMessage(chatId, 'Orders:', await formatOrderList(result));
        }
    } else if (action === 'match') {
        const order_id = message;
        const order = await getOrder(order_id);
        const {buy_token, buy_amount} = order;
        const user = await getUser(chatId);
        await sendTransaction(chatId, buy_token, 'ft_transfer_call', {
            "receiver_id": CONTRACT,
            "amount": buy_amount,
            "msg": JSON.stringify({order_id})
        }, [
            {depositContract: buy_token, depositAddress: CONTRACT},
            {depositContract: order.sell_token, depositAddress: user.accountId},
            {depositContract: order.buy_token, depositAddress: order.maker}]);
    }
})


// // Get filtered order list
// bot.onText(/\/get_orders_([a-z0-9._\-]+)#([a-z0-9._\-]+)/, async (msg, match) => {
//     const chatId = msg.chat.id;
//     const [sellToken, buyToken] = match.slice(1);
//     const result = await contractQuery(CONTRACT, "get_orders",{sell_token: sellToken, buy_token: buyToken});
//     console.log(result);
//     bot.sendMessage(chatId, 'Orders:', await formatOrderList(result));
// });
//

// Create order
bot.onText(/\/sell ([\d\.]+) ([a-z0-9._\-]+) for ([\d\.]+) ([a-z0-9._\-]+)/, async (msg, match) => {
    const chatId = msg.chat.id;
    const [sell_amount, sell_token, buy_amount, buy_token] = match.slice(1);
    await sendTransaction(chatId, sell_token, 'ft_transfer_call', {
        "receiver_id": CONTRACT,
        "amount": await fromPrecision(sell_amount, sell_token),
        "msg": JSON.stringify({
            buy_amount: await fromPrecision(buy_amount, buy_token),
            buy_token,
            sell_token,
            sell_amount: await fromPrecision(sell_amount, sell_token),
        })
    }, [{depositContract: sell_token, depositAddress: CONTRACT}]);
});

bot.onText(/\/sell$/, async (msg, match) => {
    const chatId = msg.chat.id;
    bot.sendMessage(chatId, "\/sell [sell_amount] [sell_token_address] for [buy_amount] [buy_token_address]");
});

// Match order
// bot.onText(/\/match_(\d+)/, async (msg, match) => {
//     const chatId = msg.chat.id;
//     const orderId = match[1];
//     const order = await getOrder(orderId);
//     await sendTransaction(chatId, order.buy_token, 'ft_transfer_call', {
//         "receiver_id": CONTRACT,
//         "amount": order.buy_amount,
//         "msg": {order_id: orderId}
//     });
// });


// Cancel order
bot.onText(/\/cancel (\d+)/, async (msg, match) => {
    const chatId = msg.chat.id;
    const orderId = match[1];
    const order = await getOrder(orderId);

    await sendTransaction(chatId, CONTRACT, 'remove_order', {
        sell_token: order.sell_token,
        buy_token: order.buy_token,
        order_id: orderId,
    }, [{depositContract: order.sell_token, depositAddress: order.maker}], '0');
});

const userMap = {};

http.createServer(async function(request, response) {
    const [path, query] = request.url.split('?');
    const [_, chatId, result] = path.split('/');
    const data = querystring.parse(query);

    if (result === 'login') {
        console.log(data);
        userMap[chatId] = {accountId: data.account_id, key: PublicKey.fromString(data.all_keys), chatId};
        console.log(userMap[chatId]);
        await bot.sendMessage(chatId, `Hello [${data.account_id}](${EXPLORER_URL}/accounts/${data.account_id})`, {parse_mode: 'Markdown'});
    } else if (result === 'transaction') {
        const transactionHashes = data.transactionHashes.split(',')
        for(const hash of transactionHashes) {
            await bot.sendMessage(chatId, `Success [${hash}](${EXPLORER_URL}/transactions/${hash})`, {parse_mode: 'Markdown'});
        }
    } else {
        await bot.sendMessage(chatId, `Something went wrong`);
    }

    response.writeHead(302, {
        'Location': CALLBACK_URL
    });
    response.end();
}).listen(PORT);