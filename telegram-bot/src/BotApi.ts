import TelegramBot from "node-telegram-bot-api";
import { NearApi, TokenBalance } from "./NearApi";
import { v4 as uuidv4 } from 'uuid';
import { DataStorage } from "./DataStorage";
import Big from "big.js";
import { URL } from 'url';
import * as crypto from 'crypto';

import { USDT_TOKEN_ADDRESS, CONTRACT, NETWORK, SERVER_URL } from "./config";

export class BotApi {
    constructor(private api: NearApi, private tgBot: TelegramBot, private storage: DataStorage) { }

    async viewBalances(accountId: string, chatId: string) {
        const balances: TokenBalance[] = await this.api.getFTBalances(accountId);

        console.log("balances", balances);

        const items = balances.map((b) => {
            const { tokenAccId, balance, symbol } = b;
            const metaId = uuidv4();

            this.storage.setSellMeta(chatId, metaId, {
                tokenAccId, balance, symbol
            });

            return [
                { text: symbol, callback_data: `cancel` },
                { text: balance, callback_data: "cancel" },
                { text: "BUY \b\r  🔼", callback_data: "cancel" },
                {
                    text: "SELL \b\r 🔽",
                    callback_data: `sell ${metaId}`,
                },
            ];
        });

        this.tgBot.sendMessage(chatId, "Wallet balances", {
            reply_markup: {
                inline_keyboard: items,
            },
        });
    }

    async onTokenSale(chatId: string, sellToken: string, sellTokenSymbol: string, sellAmount: string) {
        this.tgBot.sendMessage(chatId, `Type price in USDT`).then(() => {
            const cb = async (answer: any) => {
                const typedPrice = answer.text; // todo validate amount

                const buyAmount = Big(sellAmount).mul(typedPrice).toFixed();

                const sell_amount = await this.api.fromPrecision(sellAmount, sellToken);
                const sell_token = sellToken;
                const buy_amount = await this.api.fromPrecision(buyAmount, USDT_TOKEN_ADDRESS);
                const buy_token = USDT_TOKEN_ADDRESS;

                await this.sendTransaction(
                    Number(chatId),
                    sell_token,
                    "ft_transfer_call",
                    {
                        receiver_id: CONTRACT,
                        amount: await this.api.fromPrecision(sell_amount, sell_token),
                        msg: JSON.stringify({
                            buy_amount: await this.api.fromPrecision(buy_amount, buy_token),
                            buy_token,
                            sell_token,
                            sell_amount: await this.api.fromPrecision(sell_amount, sell_token),
                        }),
                    },
                    // @ts-ignore
                    [{ depositContract: sell_token, depositAddress: CONTRACT }],
                    undefined,
                    `
      
              You are selling *${sellAmount}* ${sellTokenSymbol} for *${typedPrice}* USDT per *1* token
      
              you will receive *${buyAmount}* USDT
              `.replace(/[\\$."]/g, "\\$&")
                );
            };

            this.storage.setAnswerCallback(Number(chatId), cb);
        });
    }

    async sendTransaction(
        chatId: number,
        contract: string,
        method: string,
        args = {},
        depositAddresses = [],
        deposit = "1",
        msg = ""
    ) {
        const user = await this.getUser(String(chatId));
        const url = await this.api.signURL(
            user,
            contract,
            method,
            args,
            depositAddresses,
            deposit
        );
        await this.tgBot.sendMessage(
            chatId,
            `
          ${msg}
          
          ["Click to send transaction"](${url})
          
          `,
            {
                parse_mode: "MarkdownV2",
            }
        );
    }

    async getUser(chatId: string) {
        const user = this.storage.getUserMeta(chatId);
        if (!user) {
            await this.tgBot.sendMessage(chatId, `Please [login](${this.loginUrl(Number(chatId))}) first`, {
                parse_mode: "MarkdownV2",
            });
            throw new Error("User not found");
        }
        return user;
    }

    async getOrder(orderId: string) {
        return this.api.contractQuery(CONTRACT, "get_order", { order_id: orderId });
    }

    async formatOrderList(orderList: any) {
        const inline_keyboard = [];
        for (const {
            order: { sell_amount, sell_token, buy_amount, buy_token },
            order_id,
        } of orderList) {
            inline_keyboard.push([
                {
                    text:
                        `Buy ${await this.api.toPrecision(
                            sell_amount,
                            sell_token
                        )} ${(await this.api.getTokenMeta(sell_token)).symbol}` +
                        ` for ${await this.api.toPrecision(
                            buy_amount,
                            buy_token
                        )} ${(await this.api.getTokenMeta(buy_token)).symbol}`,
                    callback_data: `match ${order_id}`,
                },
            ]);
        }
        return {
            reply_markup: {
                inline_keyboard,
            },
        };
    }

    async viewTokensToBuy(chatId: number) {
        const result = await this.api.contractQuery(CONTRACT, "get_pairs", {});
        const keyboard = await this.getSellTokensInlineKeyboard(
            result,
            USDT_TOKEN_ADDRESS
        );
        await this.tgBot.sendMessage(chatId, "Available tokens", {
            reply_markup: {
                inline_keyboard: keyboard,
            },
        });
    }

    async getSellTokensInlineKeyboard(pairs: string[], buyTokenAddress: string) {
        const inline_keyboard = [];

        for (let pair of pairs) {
            const [sellAccId, buyAccId] = pair.split("#");

            if (buyAccId == buyTokenAddress) {
                const symbol = (await this.api.getTokenMeta(sellAccId)).symbol;
                const hash = this.setPairMap(pair);
                inline_keyboard.push([
                    {
                        text: symbol,
                        callback_data: `orders ${hash}`,
                    },
                ]);
            }
        }

        return inline_keyboard;
    }

    setPairMap(pair: string) {
        const hash = crypto.createHash("md5").update(pair).digest("hex");
        DataStorage.pairMap[hash] = pair;
        return hash;
    }

    loginUrl(chatId: number) {
        const newUrl = new URL("login", `https://wallet.${NETWORK}.near.org`);
        newUrl.searchParams.set("success_url", `${SERVER_URL}/${chatId}/login`);
        newUrl.searchParams.set("failure_url", `${SERVER_URL}/${chatId}/fail`);
        return newUrl.href;
    }
}