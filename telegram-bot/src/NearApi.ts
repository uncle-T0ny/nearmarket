import axios from 'axios';
import { JsonRpcProvider } from 'near-api-js/lib/providers';
import { NETWORK, SERVER_URL } from './config';
import * as nearApi from 'near-api-js';
import Big from 'big.js';
import { CodeResult } from 'near-api-js/lib/providers/provider';
import BN from 'bn.js';
import { URL } from 'url';
import { DataStorage, TokenMeta, UserMeta } from './DataStorage';

export interface TokenBalance {
    symbol: string;
    balance: string;
    tokenAccId: string;
}

export class NearApi {
    constructor(private provider: JsonRpcProvider, private storage: DataStorage) { }

    // todo check empty object args, before it was ""
    async contractQuery(contract: string, method: string, args = {}): Promise<any> {
        const rawResult = await this.provider.query<CodeResult>({
            request_type: "call_function",
            account_id: contract,
            method_name: method,
            args_base64: Buffer.from(JSON.stringify(args)).toString("base64"),
            finality: "optimistic",
        });
        return JSON.parse(Buffer.from(rawResult.result).toString());
    }

    async getFTBalances(accountId: string): Promise<TokenBalance[]> {
        const tokens = await this.getFTTokens(accountId);
        const res: TokenBalance[] = [];
        for (const token of tokens) {
            try {
                const balanceRaw = await this.contractQuery(token, "ft_balance_of", {
                    account_id: accountId,
                });
                const balance = await this.toPrecision(balanceRaw, token, 2);
                const symbol = (await this.getTokenMeta(token)).symbol;
                if (balance != '0') {
                    res.push({ symbol, balance, tokenAccId: token });
                }
            } catch (err) {
                // todo if we can't receive balance for token, we shouldn't try to do it every time

                // console.log('failed to get ft_balance_of', err);
            }
        }

        return res;
    }

    async signURL(
        user: UserMeta,
        contract: string,
        method: string,
        args = {},
        depositAddresses = [],
        attachedDeposit = "1",
        gas = 300000000000000,
        meta = null,
    ) {
        const actions = [];
        actions.push(
            nearApi.transactions.functionCall(
                method,
                Buffer.from(JSON.stringify(args)),
                new BN(gas),
                new BN(attachedDeposit)
            )
        );
        const block = await this.provider.block({ finality: "final" });

        const txs = [];
        let nonce = 1;
        for (const { depositContract, depositAddress } of depositAddresses) {
            if (await this.needToDeposit(depositContract, depositAddress)) {
                const depositAmount = await this.getDepostiAmount(depositContract);
                const depositActions = [
                    nearApi.transactions.functionCall(
                        "storage_deposit",
                        Buffer.from(
                            JSON.stringify({
                                registration_only: true,
                                account_id: depositAddress,
                            })
                        ),
                        new BN(gas),
                        depositAmount
                    ),
                ];
                txs.push(
                    nearApi.transactions.createTransaction(
                        user.accountId,
                        user.key,
                        depositContract,
                        nonce++,
                        depositActions,
                        nearApi.utils.serialize.base_decode(block.header.hash)
                    )
                );
            }
        }
        txs.push(
            nearApi.transactions.createTransaction(
                user.accountId,
                user.key,
                contract,
                nonce,
                actions,
                nearApi.utils.serialize.base_decode(block.header.hash)
            )
        );
        const newUrl = new URL("sign", `https://wallet.${NETWORK}.near.org/`);
        newUrl.searchParams.set(
            "transactions",
            txs
                .map((transaction) =>
                    nearApi.utils.serialize.serialize(
                        nearApi.transactions.SCHEMA,
                        transaction
                    )
                )
                .map((serialized) => Buffer.from(serialized).toString("base64"))
                .join(",")
        );
        newUrl.searchParams.set(
            "callbackUrl",
            `${SERVER_URL}/${user.chatId}/transaction`
        );
        if (meta) newUrl.searchParams.set("meta", meta);
        return newUrl.href;
    }

    async getFTTokens(accountId: string) {
        const res = await axios.get(
            `https://helper.mainnet.near.org/account/${accountId}/likelyTokens`
        );
        return res.data;
    }

    async needToDeposit(contract: string, account: string) {
        const balance = await this.contractQuery(contract, "storage_balance_of", {
            account_id: account,
        });
        return !balance;
    }

    async getDepostiAmount(contract: string) {
        const result = await this.contractQuery(contract, "storage_balance_bounds", {});
        return result.min;
    }

    async toPrecision(value: string, tokenAddress: string, fixed = 6): Promise<string> {
        const precision = (await this.getTokenMeta(tokenAddress)).decimals;
        return Big(value).div(Big(10).pow(precision)).round(fixed).toFixed();
    }

    async fromPrecision(value: string, tokenAddress: string): Promise<string> {
        const precision = (await this.getTokenMeta(tokenAddress)).decimals;
        return Big(value).mul(Big(10).pow(precision)).toFixed();
    }

    async getTokenMeta(tokenAddress: string): Promise<TokenMeta> {
        let tokenMeta: TokenMeta = this.storage.getTokenMeta(tokenAddress);

        if (!tokenMeta) {
            const result = await this.contractQuery(tokenAddress, "ft_metadata");
            tokenMeta = {
                decimals: result.decimals,
                symbol: result.symbol,
                name: result.name,
            }
            this.storage.setTokenMeta(tokenAddress, tokenMeta);
        }

        return tokenMeta;
    }
}