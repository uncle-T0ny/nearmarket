import { PublicKey } from "near-api-js/lib/utils";


export interface TokenMeta {
    decimals: number;
    symbol: string;
    name: string;
}

export interface UserMeta {
    accountId: string;
    key: PublicKey;
    chatId: string;
}

export interface SellMeta {
    tokenAccId: string;
    balance: string;
    symbol: string;
}

export class DataStorage {
    private tokensMetaMap: { [tokenAccId: string]: TokenMeta } = {};
    private userMetaMap: { [chatId: string]: UserMeta } = {};
    private sellMeta: { [chatId: string]: { [metaId: string]: SellMeta } } = {};
    private answerCallbacks: { [chatId: number]: Function } = {};
    public static pairMap: { [hash: string]: string } = {};

    getSellMeta(chatId: string, metaId: string): SellMeta {
        return this.sellMeta[chatId][metaId];
    }

    setSellMeta(chatId: string, metaId: string, meta: SellMeta) {
        this.sellMeta[chatId] = {
            ...this.sellMeta[chatId],
            ...{ [metaId]: meta },
        };
    }

    getUserMeta(chatId: string): UserMeta {
        return this.userMetaMap[chatId];
    }

    setUserMeta(chatId: string, meta: UserMeta) {
        this.userMetaMap[chatId] = meta;
    }

    getTokenMeta(tokenAccId: string): TokenMeta {
        return this.tokensMetaMap[tokenAccId];
    }

    setTokenMeta(tokenAccId: string, meta: TokenMeta) {
        this.tokensMetaMap[tokenAccId] = meta;
    }

    getAnswerCallback(chatId: number): Function {
        return this.answerCallbacks[chatId];
    }

    removeAnswerCallback(chatId: number) {
        delete this.answerCallbacks[chatId];
    }

    setAnswerCallback(chatId: number, fn: Function) {
        this.answerCallbacks[chatId] = fn;
    }
}