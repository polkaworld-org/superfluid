// Copyright 2017-2019 @polkadot/app-storage authors & contributors
// This software may be modified and distributed under the terms
// of the Apache-2.0 license. See the LICENSE file for details.

import {AppProps, I18nProps} from '@polkadot/react-components/types';
import {ApiProps} from '@polkadot/react-api/types';
import TxModal, {TxModalState} from '@polkadot/react-components/TxModal';

import {withCalls, withMulti} from '@polkadot/react-api';

import '../index.css';

import React from 'react';
import Selection from './Selection';
import translate from '../translate';

interface Props extends AppProps, I18nProps, ApiProps {
  accountId?: string;
  basePath: string;
  assetNum: number;
}

interface State extends TxModalState {
}

class Container extends TxModal<Props, State> {
  public constructor(props: Props) {
    super(props);

    this.defaultState = {
      ...this.defaultState,
    };
    this.state = this.defaultState;
  }

  async componentDidMount() {
  }

  public render(): React.ReactNode {
    const {basePath, accountId} = this.props;
    return (
      <div className='container-content'>
        <Selection
          basePath={basePath}
          accountId={accountId}
        />
      </div>
    );
  }
}

export default withMulti(
  Container,
  translate,
  withCalls<Props>(
    ['query.superfluid.nextAssetId', {propName: 'assetNum'}],
  ),
);

