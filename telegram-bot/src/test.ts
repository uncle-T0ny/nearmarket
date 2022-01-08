import { PROVIDER } from "./config";
import { DataStorage } from "./DataStorage";
import { NearApi } from "./NearApi";

const api = new NearApi(PROVIDER, new DataStorage());
(async () => {
    const balances = await api.getFTBalances('rant.near');
    console.log('balances', balances);
})();