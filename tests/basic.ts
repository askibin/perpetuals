import * as anchor from "@project-serum/anchor";
import { TestClient } from "./test_client";
import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import * as spl from "@solana/spl-token";
import { expect, assert } from "chai";
import { BN } from "bn.js";

describe("perpetuals", () => {
  let tc = new TestClient();
  tc.printErrors = true;
  let oracleConfig;
  let permissions;
  let fees;
  let ratios;
  let perpetualsExpected;
  let multisigExpected;
  let tokenExpected;

  it("init", async () => {
    await tc.initFixture();
    await tc.init();

    let err = await tc.ensureFails(tc.init());
    assert(err.logs[3].includes("already in use"));

    perpetualsExpected = {
      permissions: {
        allowSwap: true,
        allowAddLiquidity: true,
        allowRemoveLiquidity: true,
        allowOpenPosition: true,
        allowClosePosition: true,
        allowPnlWithdrawal: true,
        allowCollateralWithdrawal: true,
        allowSizeChange: true,
      },
      pools: [],
      protocolFeeShareBps: new BN(100),
      transferAuthorityBump: tc.authority.bump,
      perpetualsBump: tc.perpetuals.bump,
      inceptionTime: new BN(0),
    };

    multisigExpected = {
      numSigners: 2,
      numSigned: 0,
      minSignatures: 2,
      instructionAccountsLen: 0,
      instructionDataLen: 0,
      instructionHash: new anchor.BN(0),
      signers: [
        tc.admins[0].publicKey,
        tc.admins[1].publicKey,
        PublicKey.default,
        PublicKey.default,
        PublicKey.default,
        PublicKey.default,
      ],
      signed: [false, false, false, false, false, false],
      bump: tc.multisig.bump,
    };

    let multisig = await tc.program.account.multisig.fetch(
      tc.multisig.publicKey
    );
    expect(JSON.stringify(multisig)).to.equal(JSON.stringify(multisigExpected));

    let perpetuals = await tc.program.account.perpetuals.fetch(
      tc.perpetuals.publicKey
    );
    expect(JSON.stringify(perpetuals)).to.equal(
      JSON.stringify(perpetualsExpected)
    );
  });

  it("setAdminSigners", async () => {
    await tc.setAdminSigners(1);

    let multisig = await tc.program.account.multisig.fetch(
      tc.multisig.publicKey
    );
    multisigExpected.minSignatures = 1;
    expect(JSON.stringify(multisig)).to.equal(JSON.stringify(multisigExpected));
  });

  it("setPermissions", async () => {
    perpetualsExpected.permissions = {
      allowSwap: true,
      allowAddLiquidity: true,
      allowRemoveLiquidity: true,
      allowOpenPosition: true,
      allowClosePosition: true,
      allowPnlWithdrawal: true,
      allowCollateralWithdrawal: true,
      allowSizeChange: true,
    };
    await tc.setPermissions(perpetualsExpected.permissions);

    let perpetuals = await tc.program.account.perpetuals.fetch(
      tc.perpetuals.publicKey
    );
    expect(JSON.stringify(perpetuals)).to.equal(
      JSON.stringify(perpetualsExpected)
    );
  });

  it("addAndRemovePool", async () => {
    await tc.addPool("test pool");

    let pool = await tc.program.account.pool.fetch(tc.pool.publicKey);
    let poolExpected = {
      name: "test pool",
      tokens: [],
      bump: tc.pool.bump,
      lpTokenBump: pool.lpTokenBump,
      inceptionTime: new BN(0),
    };
    expect(JSON.stringify(pool)).to.equal(JSON.stringify(poolExpected));

    await tc.removePool();
    tc.ensureFails(tc.program.account.pool.fetch(tc.pool.publicKey));

    await tc.addPool("test pool");
  });

  it("addAndRemoveToken", async () => {
    oracleConfig = {
      maxPriceError: new BN(10000),
      maxPriceAgeSec: 60,
      oracleType: { test: {} },
      oracleAccount: tc.custodies[0].oracleAccount,
    };
    permissions = {
      allowSwap: true,
      allowAddLiquidity: true,
      allowRemoveLiquidity: true,
      allowOpenPosition: true,
      allowClosePosition: true,
      allowPnlWithdrawal: true,
      allowCollateralWithdrawal: true,
      allowSizeChange: true,
    };
    fees = {
      mode: { test: {} },
      maxChange: new BN(20000),
      swap: new BN(100),
      addLiquidity: new BN(100),
      removeLiquidity: new BN(100),
      openPosition: new BN(100),
      closePosition: new BN(100),
      liquidation: new BN(100),
    };
    ratios = {
      target: new BN(100),
      min: new BN(10),
      max: new BN(1000),
    };
    await tc.addToken(tc.custodies[0], oracleConfig, permissions, fees, ratios);

    let token = await tc.program.account.custody.fetch(tc.custodies[0].custody);
    tokenExpected = {
      tokenAccount: tc.custodies[0].tokenAccount,
      mint: tc.custodies[0].mint.publicKey,
      decimals: 9,
      oracle: {
        oracleAccount: tc.custodies[0].oracleAccount,
        oracleType: { test: {} },
        maxPriceError: "10000",
        maxPriceAgeSec: 60,
      },
      permissions: {
        allowSwap: true,
        allowAddLiquidity: true,
        allowRemoveLiquidity: true,
        allowOpenPosition: true,
        allowClosePosition: true,
        allowPnlWithdrawal: true,
        allowCollateralWithdrawal: true,
        allowSizeChange: true,
      },
      fees: {
        mode: { linear: {} },
        maxChange: "20000",
        swap: "100",
        addLiquidity: "100",
        removeLiquidity: "100",
        openPosition: "100",
        closePosition: "100",
        liquidation: "100",
      },
      assets: { collateral: "0", protocolFees: "0", owned: "0", locked: "0" },
      collectedFees: {
        swap: "0",
        addLiquidity: "0",
        removeLiquidity: "0",
        openPosition: "0",
        closePosition: "0",
        liquidation: "0",
      },
      volumeStats: {
        swap: "0",
        addLiquidity: "0",
        removeLiquidity: "0",
        openPosition: "0",
        closePosition: "0",
        liquidation: "0",
      },
      tradeStats: { profit: "0", loss: "0", oiLong: "0", oiShort: "0" },
      bump: token.bump,
      tokenAccountBump: token.tokenAccountBump,
    };
    expect(JSON.stringify(token)).to.equal(JSON.stringify(tokenExpected));

    let oracleConfig2 = Object.assign({}, oracleConfig);
    oracleConfig2.oracleAccount = tc.custodies[1].oracleAccount;
    await tc.addToken(
      tc.custodies[1],
      oracleConfig2,
      permissions,
      fees,
      ratios
    );

    await tc.removeToken(tc.custodies[0]);
    tc.ensureFails(tc.program.account.custody.fetch(tc.custodies[0].custody));

    await tc.addToken(tc.custodies[0], oracleConfig, permissions, fees, ratios);
  });

  it("setTokenConfig", async () => {
    oracleConfig.maxPriceAgeSec = 90;
    permissions.allowPnlWithdrawal = false;
    fees.liquidation = new BN(200);
    ratios.target = new BN(90);
    await tc.setTokenConfig(
      tc.custodies[0],
      oracleConfig,
      permissions,
      fees,
      ratios
    );

    let token = await tc.program.account.custody.fetch(tc.custodies[0].custody);
    tokenExpected.oracle.maxPriceAgeSec = 90;
    tokenExpected.permissions.allowPnlWithdrawal = false;
    tokenExpected.fees.liquidation = "200";
    expect(JSON.stringify(token)).to.equal(JSON.stringify(tokenExpected));
  });

  it("setTestOraclePrice", async () => {
    await tc.setTestOraclePrice(123, tc.custodies[0]);
    await tc.setTestOraclePrice(200, tc.custodies[1]);

    let oracle = await tc.program.account.testOracle.fetch(
      tc.custodies[0].oracleAccount
    );
    let oracleExpected = {
      price: new BN(123000),
      expo: -3,
      conf: new BN(0),
      publishTime: oracle.publishTime,
    };
    expect(JSON.stringify(oracle)).to.equal(JSON.stringify(oracleExpected));
  });

  it("setTestTime", async () => {
    await tc.setTestTime(111);

    let perpetuals = await tc.program.account.perpetuals.fetch(
      tc.perpetuals.publicKey
    );
    expect(JSON.stringify(perpetuals.inceptionTime)).to.equal(
      JSON.stringify(new BN(111))
    );
  });

  it("addLiquidity", async () => {
    await tc.addLiquidity(
      tc.toTokenAmount(10, tc.custodies[0].decimals),
      tc.users[0],
      tc.users[0].tokenAccounts[0],
      tc.custodies[0],
      [tc.custodies[1].custody],
      [tc.custodies[1].oracleAccount]
    );
    await tc.addLiquidity(
      tc.toTokenAmount(10, tc.custodies[1].decimals),
      tc.users[1],
      tc.users[1].tokenAccounts[1],
      tc.custodies[1],
      [tc.custodies[0].custody],
      [tc.custodies[0].oracleAccount]
    );
  });

  it("swap", async () => {
    await tc.swap(
      tc.toTokenAmount(1, tc.custodies[0].decimals),
      new BN(1),
      tc.users[0],
      tc.users[0].tokenAccounts[0],
      tc.users[0].tokenAccounts[1],
      tc.custodies[0],
      tc.custodies[1]
    );
  });

  it("removeLiquidity", async () => {
    await tc.removeLiquidity(
      tc.toTokenAmount(1, 6),
      tc.users[0],
      tc.users[0].tokenAccounts[0],
      tc.custodies[0],
      [tc.custodies[1].custody],
      [tc.custodies[1].oracleAccount]
    );
    await tc.removeLiquidity(
      tc.toTokenAmount(1, 6),
      tc.users[1],
      tc.users[1].tokenAccounts[1],
      tc.custodies[1],
      [tc.custodies[0].custody],
      [tc.custodies[0].oracleAccount]
    );
  });

  /*it("openPosition", async () => {
    await tc.openPosition(
      1123,
      tc.toTokenAmount(1, tc.custodies[0].decimals),
      tc.toTokenAmount(10, tc.custodies[0].decimals),
      "long",
      tc.users[0],
      tc.users[0].tokenAccounts[0],
      tc.users[0].positionAccountsLong[0],
      tc.custodies[0]
    );
  });

  it("closePosition", async () => {
    await tc.closePosition(
      0,
      tc.toTokenAmount(1, tc.custodies[0].decimals),
      new BN(0),
      new BN(0),
      new BN(0),
      tc.users[0],
      tc.users[0].tokenAccounts[0],
      tc.users[0].positionAccountsLong[0],
      tc.custodies[0]
    );
  });*/

  /*  it("liquidate", async () => {
    await tc.liquidate(
      tc.users[0],
      tc.users[0].tokenAccounts[0],
      tc.users[0].positionAccountsLong[0],
      tc.custodies[0]
    );
  });*/
});
