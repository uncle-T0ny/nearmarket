const TelegramBot = require('node-telegram-bot-api');
const {loginUrl} = require("./utils");
const {contractQuery} = require("./utils");
const {getOrder} = require("./utils");
const {CONTRACT, BOT_TOKEN} = require("./config");
const {signURL, fromPrecision, formatOrderList} = require("./utils");
const querystring = require('querystring');

const bot = new TelegramBot(BOT_TOKEN, {polling: true});
const http = require("http");
const {formatPairList} = require("./utils");
const {PublicKey} = require("near-api-js/lib/utils");
const {CALLBACK_URL} = require("./config");

async function getUser(chatId) {
    const user = userMap[chatId];
    if (!user) {
        await bot.sendMessage(chatId, `Please [login](${loginUrl(chatId)}) first`, {parse_mode: 'MarkdownV2'});
        throw new Error('User not found');
    }
    return user;
}

async function sendTransaction(chatId, contract, method, args= {}, depositAddresses = []) {
    const user = await getUser(chatId);
    const url = await signURL(user, contract, method, args, depositAddresses)
    await bot.sendMessage(chatId, `[Send transaction](${url})`, {parse_mode: 'MarkdownV2'});
}


// login
bot.onText(/\/login$/, async (msg, match) => {
    const chatId = msg.chat.id;
    bot.sendMessage(chatId, `Please follow the [Login URL](${loginUrl(chatId)})`, {parse_mode: 'MarkdownV2'});
});

// Get pairs
bot.onText(/\/list$/, async (msg, match) => {
    const chatId = msg.chat.id;

    const result = await contractQuery(CONTRACT, "get_pairs",{});
    bot.sendMessage(chatId, 'Pairs:', await formatPairList(result));
});

bot.on("callback_query", async function callback(callBackQuery) {
    const chatId = callBackQuery.message.chat.id;
    const [action, message] = callBackQuery.data.split(' ');
    if (action === 'orders') {
        const [sellToken, buyToken] = message.split('#');
        const result = await contractQuery(CONTRACT, "get_orders", {sell_token: sellToken, buy_token: buyToken});
        bot.sendMessage(chatId, 'Orders:', await formatOrderList(result));
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
            {depositContract: order.sell_token, depositAddress: order.maker}]);
    }
})


// Get filtered order list
bot.onText(/\/get_orders_([a-z0-9._\-]+)#([a-z0-9._\-]+)/, async (msg, match) => {
    const chatId = msg.chat.id;
    const [sellToken, buyToken] = match.slice(1);
    const result = await contractQuery(CONTRACT, "get_orders",{sell_token: sellToken, buy_token: buyToken});
    console.log(result);
    bot.sendMessage(chatId, 'Orders:', await formatOrderList(result));
});


// Create order
bot.onText(/\/sell (\d+) ([a-z0-9._\-]+) for (\d+) ([a-z0-9._\-]+)/, async (msg, match) => {
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

// Match order
bot.onText(/\/match_(\d+)/, async (msg, match) => {
    const chatId = msg.chat.id;
    const orderId = match[1];
    const order = await getOrder(orderId);
    await sendTransaction(chatId, order.buy_token, 'ft_transfer_call', {
        "receiver_id": CONTRACT,
        "amount": order.buy_amount,
        "msg": {order_id: orderId}
    });
});


// Cancel order
bot.onText(/\/cancel (\d+)/, async (msg, match) => {
    const chatId = msg.chat.id;
    const orderId = match[1];

    await sendTransaction(chatId, CONTRACT, 'cancel_order', {
        "order_id": orderId,
    });
});

const userMap = {};

http.createServer(function(request, response) {
    const [path, query] = request.url.split('?');
    const [_, chatId, result] = path.split('/');
    const data = querystring.parse(query);

    if (result === 'success') {
        userMap[chatId] = {accountId: data.account_id, key: PublicKey.fromString(data.all_keys)};
        bot.sendMessage(chatId, `Hello ${data.account_id}`);
    } else {
        bot.sendMessage(chatId, `Something went wrong`);
    }

    response.writeHead(302, {
        'Location': CALLBACK_URL
    });
    response.end();
}).listen(3000);