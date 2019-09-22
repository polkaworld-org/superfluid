// Copyright 2017-2019 @polkadot/app-123code authors & contributors
// This software may be modified and distributed under the terms
// of the Apache-2.0 license. See the LICENSE file for details.

import BN from 'bn.js';
import React from 'react';
import {withApi} from '@polkadot/react-api';
import {Button, Dropdown, TxButton, TxComponent} from '@polkadot/react-components';
import {ApiProps} from '@polkadot/react-api/types';
import _ from 'lodash';
import Decimal from 'decimal.js';
import options from './config';
import './style.css';
import Summary from '../Summary';

let optionsAsset = options.filter(asset => {
  return asset.value > 0
})

const typeOptions = [
  {
    text: 'Add Liquidity',
    value: 1
  },
  {
    text: 'Remove Liquidity',
    value: 2
  },
]


interface Props extends ApiProps {
  accountId?: string;
}

interface State {
  assetAmount?: number;
  inherentAmount?: number;
  minLiquidity?: BN;
  recipientId?: string;
  formType?: number;
  assetId?: number;
  accountLiquidity?: string;
  poolInherentBalance?: string;
  poolAssetBalance?: string;
  poolAddress?: string;
  inherentAssetId?: number;
  rate?: string;
}

class Pool extends TxComponent<Props, State> {
  public state: State = {
    formType: 1,
    assetId: undefined,
    assetAmount: 0,
    inherentAmount: 0,
    minLiquidity: new BN(0),
    accountLiquidity: '',
    poolInherentBalance: '',
    poolAssetBalance: '',
    poolAddress: '',
    inherentAssetId: undefined,
    rate: ''
  };

  async componentDidMount() {
    await this.selectAsset(1)
    const {api} = this.props
    let res = await api.query.superfluid.inherentAsset()
    const inherentAssetId: number = Number(res.toString())

    this.setState({inherentAssetId})
  }

  public render(): React.ReactNode {
    const {accountId} = this.props;
    const {
      assetAmount, inherentAmount, minLiquidity, formType, assetId, inherentAssetId,
      accountLiquidity, poolInherentBalance, poolAddress, poolAssetBalance, rate
    } = this.state;

    return (
      <section>
        <div className='ui--row'>
          <div className='large'>

            <div style={{
              background: 'white',
              borderRadius: '2rem',
              height: '55px'
            }}>
              <Dropdown
                dropdownClassName='asset-dropdown'
                value={formType}
                onChange={this.selectType}
                options={typeOptions}
              />
            </div>


            <div style={{
              height: '55px',
              marginTop: '1.25rem',
              background: 'white',
              borderRadius: '2rem',
              verticalAlign: 'middle'
            }}>
              <div className='input-row' style={{
                paddingLeft: '15px',
                height: '55px',
                display: 'inline-block',
                verticalAlign: 'middle',
                width: '60%'
              }}>
                <label>
                  {/*{formType === 1 ? 'inherent asset amount' : 'min inherent asset amount'}*/}
                  Deposit
                </label>
                <input className="number-input" type="number" value={inherentAmount}
                       onChange={this.onChangeInherentAmount}/>
              </div>

              <Dropdown
                style={{
                  display: 'inline-block',
                  height: '55px',
                  verticalAlign: 'middle'
                }}
                dropdownClassName='asset-dropdown'
                value={0}
                isDisabled={true}
                // onChange={this.selectAsset}
                options={options}
              />
            </div>


            <div style={{
              height: '55px',
              marginTop: '1.25rem',
              background: 'white',
              borderRadius: '2rem',
              verticalAlign: 'middle'
            }}>
              <div className='input-row' style={{
                paddingLeft: '15px',
                height: '55px',
                display: 'inline-block',
                verticalAlign: 'middle',
                width: '60%'
              }}>
                <label>
                  {/*{formType === 1 ? 'asset amount' : 'min asset amount'}*/}
                  Deposit
                </label>
                <input className="number-input" type="number" value={assetAmount} onChange={this.onChangeAssetAmount}/>
              </div>

              <Dropdown
                style={{
                  display: 'inline-block',
                  height: '55px',
                  verticalAlign: 'middle'
                }}
                dropdownClassName='asset-dropdown'
                value={assetId}
                onChange={this.selectAsset}
                options={optionsAsset}
              />
            </div>

            {/* Remove Min Liquidity and Liquidity logic */}
            {/*<div style={{*/}
            {/*  height: '55px',*/}
            {/*  marginTop: '1.25rem',*/}
            {/*  background: 'white',*/}
            {/*  borderRadius: '2rem',*/}
            {/*  verticalAlign: 'middle'*/}
            {/*}}>*/}
            {/*  <div className='input-row' style={{*/}
            {/*    paddingLeft: '15px',*/}
            {/*    height: '55px',*/}
            {/*    display: 'inline-block',*/}
            {/*    verticalAlign: 'middle'*/}
            {/*  }}>*/}
            {/*    <label>*/}
            {/*      {formType === 1 ? 'Min Liquidity' : 'Liquidity'}*/}
            {/*    </label>*/}
            {/*    <input className="number-input" type="number" value={minLiquidity}*/}
            {/*           onChange={this.onChangeMinLiquidity}/>*/}
            {/*  </div>*/}
            {/*</div>*/}


            <Summary style={{
              background: 'rgba(41,44,47,.2)',
              marginTop: '1.25rem',
              boxShadow: '0 0 5px #ccc',
              borderRadius: '2rem',
              color: 'white',
              opacity: '1'
            }}>
              {accountLiquidity > 0 && <p>
                  Account Liquidity : {accountLiquidity}
              </p>}
              <p>
                {this.getAssetName(inherentAssetId)} Balance : {poolInherentBalance}
              </p>
              {assetId > 0 && <p>
                {this.getAssetName(assetId)} Balance : {poolAssetBalance}
              </p>}
              {rate && <p>
                  1 {this.getAssetName(inherentAssetId)} ≈ {rate} {this.getAssetName(assetId)}
              </p>}
            </Summary>

            <Button.Group>
              {formType === 1 ?
                <TxButton
                  accountId={accountId}
                  className={'pool-button'}
                  label='Add'
                  params={[assetId, inherentAmount, assetAmount, minLiquidity]}
                  tx='superfluid.addLiquidity'
                  ref={this.button}
                  onSuccess={this.onSuccess}

                /> :
                <TxButton
                  accountId={accountId}
                  className={'pool-button'}
                  label='Remove'
                  params={[assetId, minLiquidity, inherentAmount, assetAmount]}
                  tx='superfluid.removeLiquidity'
                  ref={this.button}
                  onSuccess={this.onSuccess}
                />}
            </Button.Group>


          </div>
        </div>
      </section>
    );
  }

  private getAssetName = (id?: number) => {
    const asset = options.find(asset => {
      return asset.value == id
    })
    return asset && asset.text ? asset.text : ''
  }

  private selectType = (formType?: number): void => {
    this.setState({formType});
  }

  private selectAsset = async (assetId: number) => {
    const {accountId, api} = this.props
    const {inherentAssetId} = this.state
    const res = await api.query.superfluid.accountLiquidities([assetId, accountId])
    
    const addRes = await api.query.superfluid.exchangeAccounts(assetId)
    const address = addRes.toString()

    const poolAssetBalance = await api.query.superfluid.balances([assetId, address])
    const poolInherentBalance = await api.query.superfluid.balances([inherentAssetId, address])

    this.setState({
      assetId,
      accountLiquidity: res.toString(),
      poolAddress: address,
      poolAssetBalance: poolAssetBalance.toString(),
      poolInherentBalance: poolInherentBalance.toString()
    })
    _.delay(() => this.calcRate('input'), 100)
  }

  private onSuccess = async (result: any) => {
    console.log(result, '===============1===============')
    const {api} = this.props
    const {inherentAssetId, assetId} = this.state
    const addRes = await api.query.superfluid.exchangeAccounts(assetId)
    const address = addRes.toString()

    const poolAssetBalance = await api.query.superfluid.balances([assetId, address])
    const poolInherentBalance = await api.query.superfluid.balances([inherentAssetId, address])
    this.setState({
      poolAssetBalance: poolAssetBalance.toString(),
      poolInherentBalance: poolInherentBalance.toString()
    })
  }


  private onChangeInherentAmount = (e: any): void => {
    let inherentAmount = e.target.value
    this.setState({inherentAmount});

    const poolInherentBalance = this.state.poolInherentBalance;
    const poolAssetBalance = this.state.poolAssetBalance;
    if (poolInherentBalance != 0 && poolAssetBalance != 0) {
      _.delay(() => this.calcRate('input'), 100)
    }
  }

  private onChangeAssetAmount = (e: any): void => {
    let assetAmount = e.target.value
    this.setState({assetAmount});

    const poolInherentBalance = this.state.poolInherentBalance;
    const poolAssetBalance = this.state.poolAssetBalance;
    if (poolInherentBalance != 0 && poolAssetBalance != 0) {
      _.delay(() => this.calcRate('output'), 100)
    }
  }

  private onChangeMinLiquidity = (minLiquidity?: BN): void => {
    this.setState({minLiquidity});
  }

  // 获得费率
  private calcRate = async (type?: string) => {
    let {inherentAssetId, inherentAmount = -1, assetAmount = -1, assetId} = this.state;
    if (!assetId) {
      console.log('no asset selected');
      return
    }
    const {api} = this.props
    // input inherent asset
    if (_.eq(inherentAmount, '')) {
      inherentAmount = 0
    } else {
      inherentAmount = Number(inherentAmount)
    }
    if (_.eq(assetAmount, '')) {
      assetAmount = 0
    } else {
      assetAmount = Number(assetAmount)
    }


    const addRes = await api.query.superfluid.exchangeAccounts(assetId)
    const address = addRes.toString()
    if (!address) return
    let inherentB = await api.query.superfluid.balances([inherentAssetId, address])
    let inherentBalance = Number(inherentB.toString())
    let assetB = await api.query.superfluid.balances([assetId, address])
    let assetBalance = Number(assetB.toString())
    let k = new Decimal(inherentBalance).times(assetBalance)
    // if (inputAmount >= 0 && outputAmount >= 0) return
    // if (!inputAmount && !outputAmount) return
    const rate = new Decimal(assetBalance).sub(k.div(new Decimal(1).add(inherentBalance))).toFixed(8)
    this.setState({
      rate
    })

    if (type === 'input' && inherentAmount > 0 && _.isNumber(inherentAmount)) {
      const outputNum = new Decimal(assetBalance).sub(k.div(new Decimal(inherentAmount).add(inherentBalance)))
      console.log(outputNum.toString(), '===============')
      this.setState({
        assetAmount: Number(outputNum.toFixed(8))
      })
    } else if (type === 'output' && assetAmount > 0 && _.isNumber(assetAmount)) {
      const inputNum = new Decimal(inherentBalance).sub(k.div(new Decimal(assetAmount).add(assetBalance)))
      this.setState({
        inherentAmount: Number(inputNum.toFixed(8))
      })
    }

  }


}

export default withApi(Pool);
