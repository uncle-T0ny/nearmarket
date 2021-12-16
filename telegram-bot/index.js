const TelegramBot = require('node-telegram-bot-api');
const {contractQuery} = require("./utils");
const {getOrder} = require("./utils");
const {CONTRACT, BOT_TOKEN} = require("./config");
const {signURL, fromPrecision, formatOrderList} = require("./utils");


const bot = new TelegramBot(BOT_TOKEN, {polling: true});

// Get pairs
bot.onText(/\/get_pairs$/, async (msg, match) => {
    const chatId = msg.chat.id;

    const result = await contractQuery(CONTRACT, "get_pairs",{});
    bot.sendMessage(chatId, 'Pairs:', {
        reply_markup: {
            inline_keyboard: result.map(pair => ([{
                text: pair,
                callback_data: 'orders ' + pair
            }]))
        }
    });
});

bot.on("callback_query", async function callback(callBackQuery) {
    const chatId = callBackQuery.message.chat.id;
    const [action, message] = callBackQuery.data.split(' ');
    if (action === 'orders') {
        const [sellToken, buyToken] = message.data.split('#');
        const result = await contractQuery(CONTRACT, "get_orders", {sell_token: sellToken, buy_token: buyToken});
        console.log(result);
        bot.sendMessage(chatId, 'Orders:', await formatOrderList(result));
    } else if (action === 'match') {
        const orderId = message;
        const order = await getOrder(orderId);
        const url = await signURL(order.buy_token, 'ft_transfer_call', {
            "receiver_id": CONTRACT,
            "amount": order.buy_amount,
            "msg": {order_id: orderId}
        }, "1");
        bot.sendMessage(chatId, `[Send transaction](${url})`, {parse_mode: 'MarkdownV2'});
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
bot.onText(/\/create_order sell (\d+) ([a-z0-9._\-]+) for (\d+) ([a-z0-9._\-]+)/, async (msg, match) => {
    const chatId = msg.chat.id;
    const [sell_amount, sell_token, buy_amount, buy_token] = match.slice(1);
    const url = await signURL(sell_token, 'ft_transfer_call', {
        "receiver_id": CONTRACT,
        "amount": sell_amount,
        "msg": {buy_amount: fromPrecision(buy_amount, buy_token),
            buy_token}
    }, "1");
    // send a message to the chat acknowledging receipt of their message
    bot.sendMessage(chatId, `[Send transaction](${url})`, {parse_mode: 'MarkdownV2'});
});

// Match order
bot.onText(/\/match_(\d+)/, async (msg, match) => {
    const chatId = msg.chat.id;
    const orderId = match[1];
    const order = await getOrder(orderId);
    const url = await signURL(order.buy_token, 'ft_transfer_call', {
        "receiver_id": CONTRACT,
        "amount": order.buy_amount,
        "msg": {order_id: orderId}
    }, "1");
    bot.sendMessage(chatId, `[Send transaction](${url})`, {parse_mode: 'MarkdownV2'});
});


// Cancel order
bot.onText(/\/cancel (\d+)/, async (msg, match) => {
    const chatId = msg.chat.id;
    const orderId = match[1];

    const url = await signURL(CONTRACT, 'cancel_order', {
        "order_id": orderId,
    }, "1");
    bot.sendMessage(chatId, `[Send transaction](${url})`, {parse_mode: 'MarkdownV2'});
});