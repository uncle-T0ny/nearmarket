const TelegramBot = require('node-telegram-bot-api');
const {getOrder} = require("./utils");
const {CONTRACT, BOT_TOKEN} = require("./config");
const {signURL, fromPrecision, formatOrderList} = require("./utils");


const bot = new TelegramBot(BOT_TOKEN, {polling: true});

const mockTokenList = [{sell_token: 'xabr.allbridge.testnet', sell_amount: '7000000000000000000000000', buy_token: 'xabr.allbridge.testnet', buy_amount: '53000000000000000000000000', order_id: '1'},
    {sell_token: 'xabr.allbridge.testnet', sell_amount: '7000000000000000000000000', buy_token: 'xabr.allbridge.testnet', buy_amount: '53000000000000000000000000', order_id: '2'},
    {sell_token: 'xabr.allbridge.testnet', sell_amount: '7000000000000000000000000', buy_token: 'xabr.allbridge.testnet', buy_amount: '53000000000000000000000000', order_id: '3'}]

// Get order list
bot.onText(/\/get_order_list$/, async (msg, match) => {
    const chatId = msg.chat.id;

    // const result = await contractQuery(CONTRACT, "get_order_list",{});
    const result = mockTokenList;
    bot.sendMessage(chatId, await formatOrderList(result));
});


// Get filtered order list
bot.onText(/\/get_order_list ([a-z0-9._\-]+) ([a-z0-9._\-]+)/, async (msg, match) => {
    const chatId = msg.chat.id;

    // const result = await contractQuery(CONTRACT, "get_order_list",{});
    const result = mockTokenList;
    bot.sendMessage(chatId, await formatOrderList(result));
});


// Create order
bot.onText(/\/create_order sell (\d+) ([a-z0-9._\-]+) for  (\d+) ([a-z0-9._\-]+)/, async (msg, match) => {
    const chatId = msg.chat.id;
    const [_, sell_amount, sell_token, buy_amount, buy_token] = match;
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