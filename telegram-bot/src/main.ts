import http from 'http';
import { PublicKey } from 'near-api-js/lib/utils';
import TelegramBot from 'node-telegram-bot-api';
import { URLSearchParams } from "url";
import {
    CONTRACT,
    BOT_TOKEN,
    EXPLORER_URL,
    CALLBACK_URL,
    PORT,
    PROVIDER,
} from './config';
import { NearApi } from './NearApi';
import { DataStorage } from './DataStorage';
import { BotApi } from './BotApi';

const storage = new DataStorage();
const tgBot = new TelegramBot(BOT_TOKEN, { polling: true });
const api = new NearApi(PROVIDER, storage);
const bot = new BotApi(api, tgBot, storage);

/*
 ********* COMMANDS ********
 */
// COMMAND start
tgBot.onText(/\/start/, (msg) => {
    const chatId = msg.chat.id;
    tgBot.sendMessage(chatId, "Welcome");
    tgBot.sendMessage(
        chatId,
        `Please follow the [Login URL](${bot.loginUrl(chatId)})`,
        { parse_mode: "MarkdownV2" }
    );
});

// COMMAND login
tgBot.onText(/\/login$/, async (msg, match) => {
    const chatId = msg.chat.id;
    tgBot.sendMessage(
        chatId,
        `Please follow the [Login URL](${bot.loginUrl(chatId)})`,
        { parse_mode: "MarkdownV2" }
    );
});

// COMMAND user balances
tgBot.onText(/\/balance$/, async (msg) => {
    const chatId = msg.chat.id;
    const user = await bot.getUser(String(chatId));
    const { accountId } = user;

    await bot.viewBalances(accountId, String(chatId));
});

// COMMAND buy
tgBot.onText(/\/buy$/, async (msg) => {
    const chatId = msg.chat.id;
    await bot.viewTokensToBuy(chatId);
});

tgBot.on("message", (message) => {
    var callback = storage.getAnswerCallback(message.chat.id);
    if (callback) {
        storage.removeAnswerCallback(message.chat.id);
        return callback(message);
    }
});

tgBot.on("callback_query", async function callback(callBackQuery) {
    if (!callBackQuery.data) {
        console.log('no callback query data');
        return;
    }

    if (!callBackQuery.message) {
        console.log('no callback query message');
        return;
    }

    const chatId = callBackQuery.message.chat.id;
    const [action, p1, p2, p3] = callBackQuery.data.split(" ");

    switch (action) {
        case "orders": {
            const pair = DataStorage.pairMap[p1];
            const [sellToken, buyToken] = pair.split("#");
            const result = await api.contractQuery(CONTRACT, "get_orders", {
                sell_token: sellToken,
                buy_token: buyToken,
            });
            if (!result || !result.length) {
                tgBot.sendMessage(chatId, "No orders");
            } else {
                tgBot.sendMessage(chatId, "Orders:", await bot.formatOrderList(result));
            }
            break;
        }
        case "match": {
            const order_id = p1;
            const order = await bot.getOrder(order_id);
            const { buy_token, buy_amount } = order;
            const user = await bot.getUser(String(chatId));
            await bot.sendTransaction(
                chatId,
                buy_token,
                "ft_transfer_call",
                {
                    receiver_id: CONTRACT,
                    amount: buy_amount,
                    msg: JSON.stringify({ order_id }),
                },
                [
                    // todo remove ts ignore
                    // @ts-ignore 
                    { depositContract: buy_token, depositAddress: CONTRACT },
                    // @ts-ignore
                    { depositContract: order.sell_token, depositAddress: user.accountId },
                    // @ts-ignore
                    { depositContract: order.buy_token, depositAddress: order.maker },
                ]
            );
            break;
        }
        case "sell": {
            const metaId = p1;
            const { tokenAccId, symbol, balance } = storage.getSellMeta(String(chatId), metaId);

            tgBot
                .sendMessage(
                    chatId,
                    `Type amount of ${symbol} to sell, max is:${balance}`,
                    {
                        reply_markup: {
                            inline_keyboard: [
                                [
                                    {
                                        text: `Use max amount`,
                                        callback_data: `on_sell_max_amount ${metaId}`,
                                    },
                                    { text: "Cancel", callback_data: "1111" },
                                ],
                            ],
                        },
                    }
                )
                .then(() => {
                    storage.setAnswerCallback(chatId, (answer: any) => {
                        const typedAmount = answer.text;
                        bot.onTokenSale(String(chatId), tokenAccId, symbol, typedAmount);
                    })
                });

            break;
        }
        case "on_sell_max_amount": {
            const metaId = p1;

            const { tokenAccId, symbol, balance } = storage.getSellMeta(String(chatId), metaId); // todo replace chatId to number
            bot.onTokenSale(String(chatId), tokenAccId, symbol, balance);
            break;
        }
    }
});


http
    .createServer(async (request, response) => {
        if (!request.url) {
            console.log('Request url not defined')
            return;
        }
        const [path, query] = request.url.split("?");
        const [_, chatId, result] = path.split("/");
        const data = new URLSearchParams(query);
        const accountId = data.get('account_id') as string;
        const transactionHashesParam: string = data.get('transactionHashes') as string;
        if (result === "login") {
            storage.setUserMeta(chatId, {
                accountId: accountId,
                key: PublicKey.fromString(data.get('all_keys') as string),
                chatId,
            });

            await tgBot.sendMessage(
                chatId,
                `Hello [${accountId}](${EXPLORER_URL}/accounts/${accountId})`,
                { parse_mode: "Markdown" }
            );

            await bot.viewBalances(accountId, chatId);
        } else if (result === "transaction") {
            const transactionHashes = transactionHashesParam.split(",");
            for (const hash of transactionHashes) {
                await tgBot.sendMessage(
                    chatId,
                    `Success [${hash}](${EXPLORER_URL}/transactions/${hash})`,
                    { parse_mode: "Markdown" }
                );
            }
        } else {
            await tgBot.sendMessage(chatId, `Something went wrong`);
        }

        response.writeHead(302, {
            Location: CALLBACK_URL,
        });

        response.end();
    })
    .listen(PORT);
