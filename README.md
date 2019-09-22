# Superfluid Platform

![](https://github.com/polkaworld-org/superfluid/raw/master/apps/packages/app-superfluid/src/images/logo.jpg)
By Team Pacman
## Descriptions
- Superfluid platform is a next generation DeFi platform developed using Substrate.  
- It currently provides a uniswap like token swap exchange, and will support a Compound like lending pool soon.  
- Tools for arbitrager and liquidity provider are also available.

## Contents
- **apps** UI interface  
- **pwc-node** Superfluid node  
- **pwc-node/market-making-tool** Automatic tools for arbitrager and liquidity provider  

## How to build and run

```
$ cd pwc-node
$ cargo build --release
$ cargo build -p market-making-tool --release
$ ./target/release/pwc-node--dev
$ ./target/release/market-making-tool run --addr 127.0.0.1:9944
$ 
$ cd apps
$ yarn
$ yarn run start
```
