const nearApi = require("near-api-js");
const axios = require("axios");

const Big = require("big.js");
const { SERVER_URL } = require("./config");
const { NETWORK, CONTRACT, PROVIDER } = require("./config");
const crypto = require("crypto");
const tokenMap = {};

async function signURL(
  user,
  contract,
  method,
  args = {},
  depositAddresses = [],
  attachedDeposit = "1",
  gas = 300000000000000,
  meta
) {
  const deposit =
    typeof attachedDeposit == "string"
      ? attachedDeposit
      : nearApi.utils.format.parseNearAmount("" + attachedDeposit);
  const actions = [];
  actions.push(
    method === "!transfer"
      ? nearApi.transactions.transfer(deposit)
      : nearApi.transactions.functionCall(
          method,
          Buffer.from(JSON.stringify(args)),
          gas,
          deposit
        )
  );
  const block = await PROVIDER.block({ finality: "final" });

  const txs = [];
  let nonce = 1;
  for (const { depositContract, depositAddress } of depositAddresses) {
    if (await needToDeposit(depositContract, depositAddress)) {
      const depositAmount = await getDepostiAmount(depositContract);
      const depositActions = [
        nearApi.transactions.functionCall(
          "storage_deposit",
          Buffer.from(
            JSON.stringify({
              registration_only: true,
              account_id: depositAddress,
            })
          ),
          gas,
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

function loginUrl(chatId) {
  const newUrl = new URL("login", `https://wallet.${NETWORK}.near.org`);
  newUrl.searchParams.set("success_url", `${SERVER_URL}/${chatId}/login`);
  newUrl.searchParams.set("failure_url", `${SERVER_URL}/${chatId}/fail`);
  return newUrl.href;
}

async function contractQuery(contract, method, args = "") {
  const rawResult = await PROVIDER.query({
    request_type: "call_function",
    account_id: contract,
    method_name: method,
    args_base64: Buffer.from(JSON.stringify(args)).toString("base64"),
    finality: "optimistic",
  });
  return JSON.parse(Buffer.from(rawResult.result).toString());
}

async function getFTTokens(accountId) {
  const res = await axios.get(
    `https://helper.mainnet.near.org/account/${accountId}/likelyTokens`
  );
  return res.data;
}

async function getFTBalances(accountId) {
  const tokens = await getFTTokens(accountId);
  const res = [];
  for (const token of tokens) {
    try {
      const balanceRaw = await contractQuery(token, "ft_balance_of", {
        account_id: accountId,
      });
      const balance = await toPrecision(balanceRaw, token, 2);
      const symbol = await getTokenSymbol(token);
      if (balance != '0') {
          res.push({ token: symbol, balance });
      } 
    } catch (err) {
        // console.log('failed to get ft_balance_of', err);
    }
  }
  
  return res;
}

async function getTokenPrecision(tokenAddress) {
  if (!tokenMap[tokenAddress]) {
    const result = await contractQuery(tokenAddress, "ft_metadata");
    tokenMap[tokenAddress] = {
      decimals: result.decimals,
      symbol: result.symbol,
      name: result.name,
    };
  }

  return tokenMap[tokenAddress].decimals;
}

async function getTokenSymbol(tokenAddress) {
  if (!tokenMap[tokenAddress]) {
    const result = await contractQuery(tokenAddress, "ft_metadata");
    tokenMap[tokenAddress] = {
      decimals: result.decimals,
      symbol: result.symbol,
      name: result.name,
    };
  }

  return tokenMap[tokenAddress].symbol;
}

async function getOrder(orderId) {
  return contractQuery(CONTRACT, "get_order", { order_id: orderId });
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
  for (const {
    order: { sell_amount, sell_token, buy_amount, buy_token },
    order_id,
  } of orderList) {
    inline_keyboard.push([
      {
        text:
          `Buy ${await toPrecision(
            sell_amount,
            sell_token
          )} ${await getTokenSymbol(sell_token)}` +
          ` for ${await toPrecision(
            buy_amount,
            buy_token
          )} ${await getTokenSymbol(buy_token)}`,
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

async function formatPairList(pairs) {
  const inline_keyboard = [];
  for (const pair of pairs) {
    const hash = setPairMap(pair);
    const text = await pairToString(pair);
    inline_keyboard.push([
      {
        text,
        callback_data: "orders " + hash,
      },
    ]);
  }
  return {
    reply_markup: {
      inline_keyboard,
    },
  };
}

async function pairToString(pair) {
  const [sell, buy] = pair.split("#");
  const sellSymbol = await getTokenSymbol(sell);
  const buySymbol = await getTokenSymbol(buy);
  return `${buySymbol} -> ${sellSymbol}`;
}

async function needToDeposit(contract, account) {
  const balance = await contractQuery(contract, "storage_balance_of", {
    account_id: account,
  });
  return !balance;
}

async function getDepostiAmount(contract) {
  const result = await contractQuery(contract, "storage_balance_bounds", {});
  return result.min;
}

const pairMap = {};

function setPairMap(pair) {
  const hash = crypto.createHash("md5").update(pair).digest("hex");
  pairMap[hash] = pair;
  return hash;
}

function getPair(hash) {
  return pairMap[hash];
}

module.exports = {
  signURL,
  contractQuery,
  getTokenPrecision,
  getOrder,
  toPrecision,
  fromPrecision,
  formatOrderList,
  loginUrl,
  getTokenSymbol,
  formatPairList,
  setPairMap,
  getPair,
  getFTTokens,
  getFTBalances,
};
