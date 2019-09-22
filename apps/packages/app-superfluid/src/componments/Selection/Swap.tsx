// Copyright 2017-2019 @polkadot/app-123code authors & contributors
// This software may be modified and distributed under the terms
// of the Apache-2.0 license. See the LICENSE file for details.

import React from 'react';
import {withApi} from '@polkadot/react-api';
import {ApiProps} from '@polkadot/react-api/types';
import {Button, Dropdown, TxButton, TxComponent} from '@polkadot/react-components';
import Summary from '../Summary';
import Decimal from 'decimal.js';
import _ from 'lodash';
import './style.css';
import options from './config';

interface Props extends ApiProps {
  accountId?: string;
}

interface State {
  inputAmount?: number;
  outputAmount?: number;
  inputAsset?: number;
  outputAsset?: number;
  inputBalance?: any;
  outputBalance?: any;
  rate?: number;
  autoFill: boolean;
  inherentAssetId?: number;
  btnDis: boolean
}


const defaultState = {
  inputAmount: 0,
  inputAsset: undefined,
  outputAmount: 0,
  outputAsset: undefined,
  inputBalance: undefined,
  outputBalance: undefined,
  rate: undefined,
  inherentAssetId: undefined,
  autoFill: false,
  btnDis: true
};

class Swap extends TxComponent<Props, State> {
  public state: State = defaultState

  async componentDidMount() {
    const {api} = this.props
    let res = await api.query.superfluid.inherentAsset()
    const inherentAssetId: number = Number(res.toString())
    this.setState({inherentAssetId})
    // this.setState({ outputAmount: new BN(0) })
  }

  componentWillReceiveProps(nextProps: Props) {
    const {accountId} = this.props
    if (nextProps && nextProps.accountId !== accountId) {
      // this.selectInputAsset()
      // this.selectOutputAsset()
      this.setState({
        inputAmount: 0,
        inputAsset: undefined,
        outputAmount: 0,
        outputAsset: undefined,
        inputBalance: undefined,
        outputBalance: undefined,
      })
    }
  }


  public render(): React.ReactNode {
    const {accountId} = this.props;
    const {
      outputAmount, inputAmount,
      inputAsset, outputAsset,
      inputBalance, outputBalance,
      inherentAssetId
    } = this.state;
    return (
      <section>
        <div className='ui--row'>
          <div className='large'>

            <div className='ui--row' style={{
              background: 'white',
              borderRadius: '3rem',
              height: '55px'
            }}>
              <div className='input-row'>
                <label>
                  Input
                </label>
                <input className="number-input" type="number" value={inputAmount} onChange={this.onChangeInputAmount}/>
              </div>

              <Dropdown
                dropdownClassName='asset-dropdown'
                value={inputAsset}
                defaultValue={0}
                onChange={this.selectInputAsset}
                options={options}
              />
            </div>

            <div className='ui--row'>
              <Summary className='small'>Balance : {inputBalance ? inputBalance.toString() : ''}</Summary>
            </div>

            <div className='ui--row' style={{
              background: 'white',
              borderRadius: '3rem',
              height: '55px'
            }}>
              <div className='input-row'>
                <label>
                  Output
                </label>
                <input className="number-input" type="number" value={outputAmount}
                       onChange={this.onChangeOutputAmount}/>
              </div>

              <Dropdown
                dropdownClassName='asset-dropdown'
                value={outputAsset}
                defaultValue={null}
                onChange={this.selectOutputAsset}
                options={options}
              />
            </div>

            <div className='ui--row'>
              <Summary className='small'> Balance : {outputBalance ? outputBalance.toString() : ''}</Summary>
            </div>
            <Button.Group isCentered>
              <TxButton
                accountId={accountId}
                className={'tx-button'}
                label='Swap'
                params={[accountId, inputAsset, outputAsset, inputAmount, outputAmount]}
                tx='superfluid.swapAssetsWithExactInput'
                ref={this.button}
              />
            </Button.Group>
          </div>
        </div>
      </section>
    );
  }

  // 获得费率
  private calcRate = async (inputAssetId?: number, outputAssetId?: number, type?: string) => {
    let {inherentAssetId, inputAmount = -1, outputAmount = -1} = this.state;
    const {api} = this.props
    // input inherent asset
    if (_.eq(inputAmount, '')) {
      inputAmount = 0
    } else {
      inputAmount = Number(inputAmount)
    }
    if (_.eq(outputAmount, '')) {
      outputAmount = 0
    } else {
      outputAmount = Number(outputAmount)
    }

    if (inputAssetId === inherentAssetId) {
      const addRes = await api.query.superfluid.exchangeAccounts(outputAssetId)
      const address = addRes.toString()
      if (!address) return
      let inherentB = await api.query.superfluid.balances([inherentAssetId, address])
      let inherentBalance = Number(inherentB.toString())
      let assetB = await api.query.superfluid.balances([outputAssetId, address])
      let assetBalance = Number(assetB.toString())
      let k = new Decimal(inherentBalance).times(assetBalance)
      // if (inputAmount >= 0 && outputAmount >= 0) return
      // if (!inputAmount && !outputAmount) return

      if (type === 'input' && inputAmount > 0 && _.isNumber(inputAmount)) {
        const outputNum = new Decimal(assetBalance).sub(k.div(new Decimal(inputAmount).add(inherentBalance)))
        console.log(outputNum.toString(), '===============')
        this.setState({
          outputAmount: Number(outputNum.toFixed(8))
        })
      } else if (type === 'output' && outputAmount > 0 && _.isNumber(outputAmount)) {
        const inputNum = new Decimal(inherentBalance).sub(k.div(new Decimal(outputAmount).add(assetBalance)))
        this.setState({
          inputAmount: Number(inputNum.toFixed(8))
        })
      }

      return 0
    }
    // out put inherent asset
    if (outputAssetId === inherentAssetId) {
      console.log('output = inherent')
      const addRes = await api.query.superfluid.exchangeAccounts(inputAssetId)
      const address = addRes.toString()
      if (!address) return
      let inherentB = await api.query.superfluid.balances([inherentAssetId, address])
      let inherentBalance = Number(inherentB.toString())
      let assetB = await api.query.superfluid.balances([inputAssetId, address])
      let assetBalance = Number(assetB.toString())
      let k = new Decimal(inherentBalance).times(assetBalance)
      // if (inputAmount >= 0 && outputAmount >= 0) return
      // if (!inputAmount && !outputAmount) return

      if (type === 'input' && inputAmount > 0 && _.isNumber(inputAmount)) {
        const outputNum = new Decimal(inherentBalance).sub(k.div(new Decimal(inputAmount).add(assetBalance)))
        console.log(outputNum.toString(), '===============')
        this.setState({
          outputAmount: Number(outputNum.toFixed(8))
        })
      } else if (type === 'output' && outputAmount > 0 && _.isNumber(outputAmount)) {
        const inputNum = new Decimal(assetBalance).sub(k.div(new Decimal(outputAmount).add(inherentBalance)))
        this.setState({
          inputAmount: Number(inputNum.toFixed(8))
        })
      }

      return 0
    }

    // none of above
    if (inputAssetId !== inherentAssetId && outputAssetId !== inherentAssetId) {
      const inputAddRes = await api.query.superfluid.exchangeAccounts(inputAssetId)
      const inputAddress = inputAddRes.toString()
      const outputAddRes = await api.query.superfluid.exchangeAccounts(outputAssetId)
      const outputAddress = outputAddRes.toString()


      if (!inputAddress && !outputAddress) return


      let inputInherentB = await api.query.superfluid.balances([inherentAssetId, inputAddress])
      let inputInherentBalance = Number(inputInherentB.toString())
      let inputAssetB = await api.query.superfluid.balances([inputAssetId, inputAddress])
      let inputAssetBalance = Number(inputAssetB.toString())
      let inputK = new Decimal(inputInherentBalance).times(inputAssetBalance)

      let outputInherentB = await api.query.superfluid.balances([inherentAssetId, outputAddress])
      let outputInherentBalance = Number(outputInherentB.toString())
      let outputAssetB = await api.query.superfluid.balances([outputAssetId, outputAddress])
      let outputAssetBalance = Number(outputAssetB.toString())
      let outputK = new Decimal(outputInherentBalance).times(outputAssetBalance)
      if (inputK.toNumber() === 0 || outputK.toNumber() === 0) {
        console.log('error ===============')
        return
      }
      if (type === 'input' && inputAmount > 0 && _.isNumber(inputAmount)) {
        const outputInherentAmount = new Decimal(inputInherentBalance).sub(inputK.div(new Decimal(inputAmount).add(inputAssetBalance)))
        console.log(outputInherentAmount.toString(), '===============')
        const outputNum = new Decimal(outputAssetBalance).sub(outputK.div(new Decimal(outputInherentAmount).add(outputInherentBalance)))
        this.setState({
          outputAmount: Number(outputNum.toFixed(8))
        })
      } else if (type === 'output' && outputAmount > 0 && _.isNumber(outputAmount)) {
        const outputInherentAmount = new Decimal(outputInherentBalance).sub(outputK.div(new Decimal(outputAmount).add(outputAssetBalance)))
        console.log(outputInherentAmount.toString(), '===============')
        const inputNum = new Decimal(inputAssetBalance).sub(inputK.div(new Decimal(outputInherentAmount).add(inputInherentBalance)))
        this.setState({
          inputAmount: Number(inputNum.toFixed(8))
        })
      }

    }

  }
  private onChangeInputAmount = async (e: any) => {
    const inputAmount = e.target.value;
    const {outputAsset, inputAsset} = this.state
    if (Number(inputAsset) >= 0 && Number(outputAsset) >= 0) {
      // todo 计算费率 并填充
      this.setState({inputAmount})
      // await this.calcRate(inputAsset, outputAsset, 'input')
      _.delay(() => this.calcRate(inputAsset, outputAsset, 'input'), 100)
    } else {
      this.setState({inputAmount});
    }
  }

  private onChangeOutputAmount = async (e: any) => {
    const outputAmount = e.target.value
    const {outputAsset, inputAsset} = this.state
    if (Number(inputAsset) >= 0 && Number(outputAsset) >= 0) {
      // todo 计算费率 并填充
      this.setState({outputAmount})
      _.delay(() => this.calcRate(inputAsset, outputAsset, 'output'), 100)
    } else {
      this.setState({outputAmount});
    }
  }

  private selectInputAsset = async (val?: number) => {
    if (val === undefined || val === null) {
      this.setState({inputAsset: undefined})
      return
    }
    const {accountId, api} = this.props
    const {outputAsset} = this.state
    let balance = await api.query.superfluid.balances([val, accountId])

    if (val === outputAsset) {
      this.setState({
        outputAsset: undefined,
        outputAmount: 0,
        inputAsset: val,
        inputBalance: balance
      })
    } else {
      this.setState({
        inputAsset: val,
        inputBalance: balance
      })
    }
  }

  private valid = async () => {
    const {
      outputAmount, inputAmount,
      inputAsset, outputAsset,
    } = this.state;

    const outAmountFlag = outputAmount && Number(outputAmount.toString()) > 0
    const inAmountFlag = inputAmount && Number(inputAmount.toString()) > 0
    const inputAssetFlag = inputAsset !== undefined && inputAsset >= 0
    const outputAssetFlag = outputAsset !== undefined && outputAsset >= 0
    const flag = outAmountFlag && inAmountFlag && inputAssetFlag && outputAssetFlag

    // this.setState({
    //   btnDis:flag
    // })
    console.log(!flag, '=============')
    return !flag
  }
  private selectOutputAsset = async (val?: number) => {
    if (val === undefined || val === null) {
      this.setState({outputAsset: undefined})
      return
    }
    const {accountId, api} = this.props
    const {inputAsset} = this.state
    let balance = await api.query.superfluid.balances([val, accountId])
    // balance = balance.toString()
    if (val === inputAsset) {
      this.setState({
        inputAsset: undefined,
        inputAmount: 0,
        outputAsset: val,
        outputBalance: balance
      })
    } else {
      this.setState({
        outputAsset: val,
        outputBalance: balance
      })
    }
  }

}

export default withApi(Swap);
