# Sandblizzard notes 

The perpetuals program is basically GMX on Solana. For more info please check out these resources
- [GMX deep dive](https://mirror.xyz/0x1e35A719f1d68da02DEf39Bde510c9cc4efDC84B/1WbTXj5CjB4CU0W083p8MDj_wfkwzDVzbCZmLHcDxr4)
- [GMX docs](https://gmxio.gitbook.io/gmx/)

## About 
The basic aim is to increase capital efficiency by using liquidity for AMM, lending and market making in general. While some projects have purused Uniswap CLMM to increase capital effiency others have found new use cases for idle capital. In the case for the GMX this is lending and perps MM. 

It also have at least two other pros, namely
- liquidity can be deployed more inactive than having to concentrate and adjust the position
- LPs receive LP tokens when they deposit to a pool, thus they are exposed to a basket of assets and thus the portfolio risk is reduced

## Dynamic Borrow rate
User can take out positions against tokens in any pool. However, to disentivize users to drain the token pool for liqudidity a dynamic borrow fee is introduced. This is very similar to how any lending protocol like AAVE and Sollend do it. 

## Dynamic swap fee
One challenge with multiple uses of the same liquidity is co-effects. Meaning if a user takes out a large long position against pool1-SOL and SOL increases much of the liquidity will be locked. Thus this will hurt users that wishes to use the AMM to swap from any token to SOL. It is therefore necessary to (dis)incentivice users to drain pool positions. This is done in GMX by increasing the swap fees in the direction of the drained liqudity and decrease it in the other direction. 

Another big issue with multiple pool risks is that users can end up being liquiditated PnL extreme cases. It makes sense that a user gets liquidated when then margin falls below the maintance margin. However, if a user has a large long against a pool token it might happen that the pool token itself is liquiditated. Hopefully, there will be enough short positons to balance out the extreme long. Perpetual dexes like Mango and Drift usually solves this by incentivizing position equilibrium. There are ways that an GMX clone can mitigate this problem, namely
- Liquidate the pool itself 
- Rebalance the pool to meet weights requirements. This can be done through other dexes and would naturally expose the protocol to large slippage. 

## Current implementation
### Pools
The current implementation supports multiple pools rather than one main pool like GMX has. On a chain like Solana where liquidity absolutely dried up in the fall of 22 it makes it hard to argue for multiple pools. Although a cool feature this can end up fragmenting liquidity and make the protocol more exposed to be liquidated. 

### Deposit to pool
Any user can deposit accepted tokens to a pool in exchange for pool liquidity position tokens. The LP token share is calculated from the USD value of the deposited asset. 

### Collateral
Users can post whatever collateral they want but it will be turned into hard money like USDC immidiately to reduce risks. 

### Liquidation bot
There is a liq bot that can be run to check positions and liquidiate for a small fee. 

# Potential Features
This list is just based on Sandblizzards ideas plus input from third parties

- Simple UI frontend for degens 
- Bring liquidity over to DEX using wormhole
- Power Options rather than X leverage 
- Liquidity mining through trading stats per user
- Referral program 
- Incentivice LPs to deploy capital 

# TODO
