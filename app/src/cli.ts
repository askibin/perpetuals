//@ts-nocheck
import { BN } from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { PerpetualsClient } from "./client";

(async function main() {
  // read args
  if (process.argv.length < 5) {
    throw new Error(
      "Usage: npx ts-node src/cli.ts CLUSTER_URL ADMIN_KEY_PATH COMMAND PARAM"
    );
  }
  let clusterUrl = process.argv[2];
  let adminKey = process.argv[3];
  let command = process.argv[4];
  let param = process.argv[5];

  // constants and params, to be loaded from config files
  let perpetualsConfig = {
    minSignatures: 1,
    allowSwap: true,
    allowAddLiquidity: true,
    allowRemoveLiquidity: true,
    allowOpenPosition: true,
    allowClosePosition: true,
    allowPnlWithdrawal: true,
    allowCollateralWithdrawal: true,
    allowSizeChange: true,
    protocolFeeShareBps: new BN(100),
  };
  let poolNames = ["GLP"];

  // init client
  let client = new PerpetualsClient(clusterUrl, [adminKey]);
  client.log("Client Initialized");

  client.log("Processing command: " + command);
  switch (command) {
    case "init":
      await client.init(perpetualsConfig);
      client.prettyPrint(await client.getPerpetuals());
    case "addPool":
      await client.addPool(param);
      client.prettyPrint(await client.getPool(param));
  }
})();
